//! LHAPDF6 grid access: parsed set metadata, per-member subgrids, PDG flavor
//! indexing, and log-bicubic `x·f(x, Q²)` interpolation.
//!
//! Interpolation lives behind the [`interp`] seam ([`interp::Bicubic2D`]); the
//! backend that matches LHAPDF6 (and hence MadGraph) is [`interp::LogBicubic`].
//! Points past the tabulated range go to the [`extrap`] seam
//! ([`extrap::Extrapolate2D`]), whose LHAPDF-matching backend is
//! [`extrap::Continuation`] — the split, and which one a point takes, is
//! LHAPDF's own (`GridPDF::_xfxQ2`).
//!
//! A set also carries the strong coupling it was fitted at ([`alphas::GridAlphaS`]),
//! which is the coupling a run reading these densities has to use.

pub mod alphas;
pub mod extrap;
pub mod grid;
pub mod interp;

use std::path::{Path, PathBuf};

use crate::helas::repr::Real;
use extrap::{Continuation, Extrapolate2D, PdfPointError};
use grid::{GridError, SetInfo, SubGrid};
use interp::{Bicubic2D, LogBicubic};

/// LHAPDF's flavor alias: PDG code 0 means the gluon (21) in `.dat` flavor lists.
pub fn normalize_flavor_pdg(pdg: i32) -> i32 {
    if pdg == 0 {
        21
    } else {
        pdg
    }
}

impl SubGrid {
    /// Position of `pdg` within this subgrid's flavor list (0 aliases to 21).
    pub fn flavor_index(&self, pdg: i32) -> Option<usize> {
        let pdg = normalize_flavor_pdg(pdg);
        self.flavors.iter().position(|&f| f == pdg)
    }
}

/// A loaded LHAPDF6 PDF set: `.info` metadata plus the directory members are
/// loaded from.
#[derive(Debug)]
pub struct PdfSet {
    pub info: SetInfo,
    dir: PathBuf,
    name: String,
}

impl PdfSet {
    /// Load a PDF set's `.info` metadata from `<dir>/<name>.info`. Individual
    /// members are loaded lazily via [`PdfSet::member`].
    pub fn load(dir: &Path, name: &str) -> Result<Self, GridError> {
        let info_path = dir.join(format!("{name}.info"));
        let info = grid::parse_info_file(&info_path)?;
        Ok(PdfSet {
            info,
            dir: dir.to_path_buf(),
            name: name.to_owned(),
        })
    }

    /// Load member `id`'s subgrids from `<dir>/<name>_{id:04}.dat`.
    pub fn member(&self, id: u32) -> Result<PdfMember, GridError> {
        if id >= self.info.num_members {
            return Err(GridError::MemberOutOfRange {
                set: self.name.clone(),
                member: id,
                num_members: self.info.num_members,
            });
        }
        let dat_path = self.dir.join(format!("{}_{id:04}.dat", self.name));
        let subgrids = grid::parse_member_dat(&dat_path)?;
        Ok(PdfMember::from_subgrids(subgrids).with_force_positive(self.info.force_positive))
    }
}

/// One PDF member's parsed subgrids (in file order), its precomputed log-bicubic
/// interpolator, and the continuation that reads points past the grid.
#[derive(Debug)]
pub struct PdfMember {
    pub subgrids: Vec<SubGrid>,
    interp: LogBicubic,
    extrap: Continuation,
    force_positive: i32,
}

impl PdfMember {
    /// Build a member directly from parsed subgrids (precomputing the
    /// interpolator). Mainly useful for tests with in-memory grids. Carries no
    /// positivity clamp (`ForcePositive: 0`); use
    /// [`PdfMember::with_force_positive`] to set one.
    pub fn from_subgrids(subgrids: Vec<SubGrid>) -> Self {
        let interp = LogBicubic::build(&subgrids);
        PdfMember {
            subgrids,
            interp,
            extrap: Continuation,
            force_positive: 0,
        }
    }

    /// Set the `ForcePositive` level (`0`, `1` or `2`) applied to every value
    /// this member hands out.
    pub fn with_force_positive(mut self, level: i32) -> Self {
        self.force_positive = level;
        self
    }

    /// The `ForcePositive` level this member applies.
    pub fn force_positive(&self) -> i32 {
        self.force_positive
    }

    /// `x·f(x, Q²)` for PDG code `pdg` (0 aliases the gluon 21).
    ///
    /// Inside the tabulated range this is the LHAPDF-matching log-bicubic
    /// interpolation; outside it, the LHAPDF-matching continuation. The split is
    /// `GridPDF::_xfxQ2`'s, and so is the guard in front of it: a point that is
    /// not a point (`x` at or below zero, a negative `Q²`, either non-finite) is
    /// refused, as is an `x` above the grid's last knot, which is the one
    /// direction the continuation does not extend into. An absent flavor
    /// returns exactly zero before either branch runs, matching
    /// `PDF::xfxQ2`'s own ordering ahead of the positivity clamp, which is
    /// applied last, on the way out.
    pub fn try_xfx_q2<F: Real>(&self, pdg: i32, x: F, q2: F) -> Result<F, PdfPointError> {
        let xv = x.to_f64().unwrap();
        let q2v = q2.to_f64().unwrap();
        // Negated comparisons so a NaN lands in the refusing branch.
        if !(xv > 0.0) || !xv.is_finite() || !(q2v >= 0.0) || !q2v.is_finite() {
            return Err(PdfPointError::Unphysical { x: xv, q2: q2v });
        }
        if !self.interp.has_flavor(pdg) {
            return Ok(F::zero());
        }
        let value = if extrap::in_grid_range(&self.interp.edges(), xv, q2v) {
            self.interp.xfx_q2(pdg, x, q2)?
        } else {
            self.extrap.xfx_q2(&self.interp, pdg, x, q2)?
        };
        Ok(force_positive_clamp(self.force_positive, value))
    }

    /// Like [`PdfMember::try_xfx_q2`] but panics on a point with no reading.
    pub fn xfx_q2<F: Real>(&self, pdg: i32, x: F, q2: F) -> F {
        self.try_xfx_q2(pdg, x, q2)
            .unwrap_or_else(|e| panic!("{e}"))
    }
}

/// LHAPDF's `PDF::xfxQ2` positivity switch (`src/PDF.cc`), applied after
/// interpolation/continuation and after the absent-flavor zero, so it never
/// touches either of those. Level `2`'s floor is not `Float::max`: that
/// returns the non-NaN operand and would silently clamp a NaN, where LHAPDF's
/// `if (xfx < 1e-10)` is false for NaN and passes it through unchanged.
fn force_positive_clamp<F: Real>(level: i32, value: F) -> F {
    match level {
        0 => value,
        1 => {
            if value < F::zero() {
                F::zero()
            } else {
                value
            }
        }
        2 => {
            let floor = F::from(1e-10).unwrap();
            if value < floor {
                floor
            } else {
                value
            }
        }
        other => panic!(
            "unreachable ForcePositive level {other}: the parser refuses anything outside 0..=2"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grid::SubGrid;

    fn sample_subgrid() -> SubGrid {
        SubGrid {
            x: vec![0.1, 0.5],
            q2: vec![10.0, 100.0],
            flavors: vec![-1, 1, 21, 22],
            xf: vec![0.0; 2 * 2 * 4],
        }
    }

    /// Mirrors LHAPDF's `PDF::xfxQ2` switch (`src/PDF.cc`): level 0 touches
    /// nothing, level 1 floors only negatives, level 2 floors below `1e-10`
    /// (including a value straddling it), and NaN passes through unchanged at
    /// every level (`if (xfx < floor)` is false for NaN).
    #[test]
    fn force_positive_clamp_matches_lhapdfs_switch() {
        assert_eq!(force_positive_clamp(0, 0.5), 0.5);
        assert_eq!(force_positive_clamp(0, -0.5), -0.5);
        assert!(force_positive_clamp::<f64>(0, f64::NAN).is_nan());

        assert_eq!(force_positive_clamp(1, -0.5), 0.0);
        assert_eq!(force_positive_clamp(1, 0.5), 0.5);
        assert!(force_positive_clamp::<f64>(1, f64::NAN).is_nan());

        assert_eq!(force_positive_clamp(2, 9.4e-11), 1e-10);
        assert_eq!(force_positive_clamp(2, 1.34e-10), 1.34e-10);
        assert!(force_positive_clamp::<f64>(2, f64::NAN).is_nan());
    }

    /// `PDF.cc`'s `if (!hasFlavor(id2)) return 0.0;` runs ahead of the clamp:
    /// a level-2 member must return exactly zero for a flavor its subgrids do
    /// not carry, in range and out of it, never the `1e-10` floor.
    #[test]
    fn an_absent_flavour_is_zero_and_not_the_floor() {
        let member = PdfMember::from_subgrids(vec![sample_subgrid()]).with_force_positive(2);
        let in_range: f64 = member.xfx_q2(5, 0.2, 50.0);
        assert_eq!(in_range, 0.0);
        let out_of_range: f64 = member.xfx_q2(5, 0.2, 1000.0);
        assert_eq!(out_of_range, 0.0);
    }

    #[test]
    fn gluon_zero_aliases_to_21() {
        assert_eq!(normalize_flavor_pdg(0), 21);
        assert_eq!(normalize_flavor_pdg(21), 21);
        assert_eq!(normalize_flavor_pdg(-1), -1);
    }

    #[test]
    fn flavor_index_resolves_gluon_alias() {
        let sg = sample_subgrid();
        assert_eq!(sg.flavor_index(21), Some(2));
        assert_eq!(sg.flavor_index(0), Some(2));
        assert_eq!(sg.flavor_index(22), Some(3));
        assert_eq!(sg.flavor_index(5), None);
    }

    #[test]
    fn load_missing_set_is_io_error() {
        let dir = std::env::temp_dir().join("vibegraph_pdf_test_missing_set");
        let err = PdfSet::load(&dir, "NoSuchSet").unwrap_err();
        assert!(matches!(err, GridError::Io { .. }), "{err:?}");
    }

    #[test]
    fn member_out_of_range_is_typed_error() {
        let dir = std::env::temp_dir().join(format!("vibegraph_pdf_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let info_path = dir.join("Tiny.info");
        std::fs::write(
            &info_path,
            "SetDesc: \"test\"\nFormat: lhagrid1\nNumMembers: 1\nParticle: 2212\n\
Flavors: [21]\nOrderQCD: 0\nErrorType: none\nXMin: 0.01\nXMax: 1\nQMin: 1\nQMax: 100\n\
AlphaS_MZ: 0.118\nAlphaS_OrderQCD: 0\nAlphaS_Type: ipol\nAlphaS_Qs: [1.0]\n\
AlphaS_Vals: [0.5]\nAlphaS_Lambda4: 0.3\nAlphaS_Lambda5: 0.2\n",
        )
        .unwrap();

        let set = PdfSet::load(&dir, "Tiny").expect("info should parse");
        let err = set.member(5).unwrap_err();
        assert!(matches!(err, GridError::MemberOutOfRange { .. }), "{err:?}");

        std::fs::remove_dir_all(&dir).ok();
    }
}
