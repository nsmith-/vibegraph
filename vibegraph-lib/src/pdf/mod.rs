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
//!
//! # Reading a point
//!
//! [`PdfMember::xfx_all`] is the form a luminosity sum wants: one `(x, Q²)`,
//! every flavor, into a [`FlavorRow`] indexed by [`flavor_slot`]. A hadronic
//! phase-space point has exactly two distinct evaluation points — one per beam —
//! however many subprocesses are summed over it, so the guards, the band
//! selection, the two logarithms and the two knot searches happen twice per
//! point rather than once per (subprocess, beam, ordering).
//! [`PdfMember::xfx_q2`] reads one flavor and is the oracle and test form; the
//! two agree bit for bit.

pub mod alphas;
pub mod extrap;
pub mod grid;
pub mod interp;

use std::path::{Path, PathBuf};

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

/// Slots a [`FlavorRow`] carries. Fourteen are used — the six quarks, their
/// antiquarks, the gluon and the photon, which is every code an `lhagrid1`
/// flavor list holds; the array is rounded up to a power of two.
pub const FLAVOR_SLOTS: usize = 16;

/// Every tabulated flavor's `x·f` at one `(x, Q²)`, indexed by [`flavor_slot`].
/// A slot the member does not carry holds exactly zero.
pub type FlavorRow = [f64; FLAVOR_SLOTS];

/// The slot PDG code `pdg` occupies in a [`FlavorRow`] (0 aliases the gluon 21),
/// or `None` for a code no parton density is tabulated for.
///
/// This is a dense renumbering of the PDG codes, not a position in any
/// particular member's flavor list: it is the same for every set, so a consumer
/// can resolve its own beam flavors to slots once at setup and index the rows
/// directly from then on.
#[inline]
pub fn flavor_slot(pdg: i32) -> Option<usize> {
    match normalize_flavor_pdg(pdg) {
        p @ -6..=-1 => Some((p + 6) as usize),
        p @ 1..=6 => Some((p + 5) as usize),
        21 => Some(12),
        22 => Some(13),
        _ => None,
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
    pub fn try_xfx_q2(&self, pdg: i32, x: f64, q2: f64) -> Result<f64, PdfPointError> {
        // Negated comparisons so a NaN lands in the refusing branch.
        if !(x > 0.0) || !x.is_finite() || !(q2 >= 0.0) || !q2.is_finite() {
            return Err(PdfPointError::Unphysical { x, q2 });
        }
        if !self.interp.has_flavor(pdg) {
            return Ok(0.0);
        }
        let value = if extrap::in_grid_range(&self.interp.edges(), x, q2) {
            self.interp.xfx_q2(pdg, x, q2)?
        } else {
            self.extrap.xfx_q2(&self.interp, pdg, x, q2)?
        };
        Ok(force_positive_clamp(self.force_positive, value))
    }

    /// Like [`PdfMember::try_xfx_q2`] but panics on a point with no reading.
    pub fn xfx_q2(&self, pdg: i32, x: f64, q2: f64) -> f64 {
        self.try_xfx_q2(pdg, x, q2)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    /// Every tabulated flavor's `x·f` at one `(x, Q²)`, written into `out` by
    /// [`flavor_slot`] — the same numbers [`PdfMember::try_xfx_q2`] returns one
    /// at a time, bit for bit, with the guards, band selection, logarithms and
    /// knot searches done once for all of them.
    ///
    /// A slot no band carries stays exactly zero and is not clamped, matching
    /// `PDF::xfxQ2`'s absent-flavor return ahead of its positivity switch.
    /// Outside the tabulated range each present flavor takes the scalar
    /// continuation: that is assembled from readings at the grid's own edge, so
    /// it has no all-flavor form to share, and production factorisation scales
    /// sit inside the grid at all but the hardest points.
    pub fn try_xfx_all(&self, x: f64, q2: f64, out: &mut FlavorRow) -> Result<(), PdfPointError> {
        if !(x > 0.0) || !x.is_finite() || !(q2 >= 0.0) || !q2.is_finite() {
            return Err(PdfPointError::Unphysical { x, q2 });
        }
        if extrap::in_grid_range(&self.interp.edges(), x, q2) {
            self.interp.xfx_all(x, q2, out)?;
        } else {
            *out = [0.0; FLAVOR_SLOTS];
            for &(slot, pdg) in self.interp.present_flavors() {
                out[slot as usize] = self.extrap.xfx_q2(&self.interp, pdg, x, q2)?;
            }
        }
        if self.force_positive != 0 {
            for &(slot, _) in self.interp.present_flavors() {
                out[slot as usize] = force_positive_clamp(self.force_positive, out[slot as usize]);
            }
        }
        Ok(())
    }

    /// Like [`PdfMember::try_xfx_all`] but panics on a point with no reading.
    pub fn xfx_all(&self, x: f64, q2: f64, out: &mut FlavorRow) {
        self.try_xfx_all(x, q2, out)
            .unwrap_or_else(|e| panic!("{e}"))
    }
}

/// LHAPDF's `PDF::xfxQ2` positivity switch (`src/PDF.cc`), applied after
/// interpolation/continuation and after the absent-flavor zero, so it never
/// touches either of those. Level `2`'s floor is not `f64::max`: that
/// returns the non-NaN operand and would silently clamp a NaN, where LHAPDF's
/// `if (xfx < 1e-10)` is false for NaN and passes it through unchanged.
fn force_positive_clamp(level: i32, value: f64) -> f64 {
    match level {
        0 => value,
        1 => {
            if value < 0.0 {
                0.0
            } else {
                value
            }
        }
        2 => {
            if value < 1e-10 {
                1e-10
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
        assert!(force_positive_clamp(0, f64::NAN).is_nan());

        assert_eq!(force_positive_clamp(1, -0.5), 0.0);
        assert_eq!(force_positive_clamp(1, 0.5), 0.5);
        assert!(force_positive_clamp(1, f64::NAN).is_nan());

        assert_eq!(force_positive_clamp(2, 9.4e-11), 1e-10);
        assert_eq!(force_positive_clamp(2, 1.34e-10), 1.34e-10);
        assert!(force_positive_clamp(2, f64::NAN).is_nan());
    }

    /// `PDF.cc`'s `if (!hasFlavor(id2)) return 0.0;` runs ahead of the clamp:
    /// a level-2 member must return exactly zero for a flavor its subgrids do
    /// not carry, in range and out of it, never the `1e-10` floor. The
    /// all-flavor reading has to make the same distinction, since it clamps a
    /// whole row at once.
    #[test]
    fn an_absent_flavour_is_zero_and_not_the_floor() {
        let member = PdfMember::from_subgrids(vec![sample_subgrid()]).with_force_positive(2);
        let in_range: f64 = member.xfx_q2(5, 0.2, 50.0);
        assert_eq!(in_range, 0.0);
        let out_of_range: f64 = member.xfx_q2(5, 0.2, 1000.0);
        assert_eq!(out_of_range, 0.0);

        let mut row = [0.0; FLAVOR_SLOTS];
        member.xfx_all(0.2, 50.0, &mut row);
        assert_eq!(row[flavor_slot(5).unwrap()], 0.0);
        assert_eq!(row[flavor_slot(1).unwrap()], 1e-10);
    }

    /// The all-flavor row is the single-flavor reading flavor by flavor, bit for
    /// bit, on both sides of the grid boundary and with the positivity clamp on:
    /// the cross section reads rows while every oracle reads single flavors, so
    /// a difference between the two would be an ungated production path.
    #[test]
    fn a_flavor_row_is_the_single_flavor_readings() {
        let n = 8;
        let geom = |a: f64, b: f64| -> Vec<f64> {
            let (la, lb) = (a.ln(), b.ln());
            (0..n)
                .map(|i| (la + (lb - la) * i as f64 / (n - 1) as f64).exp())
                .collect()
        };
        let x = geom(1e-5, 1.0);
        let q2 = geom(1.0, 1e6);
        let flavors = vec![-2, -1, 1, 2, 21, 22];
        let mut xf = vec![0.0; n * n * flavors.len()];
        for (ix, &xv) in x.iter().enumerate() {
            for (iq, &q2v) in q2.iter().enumerate() {
                for (ifl, &fl) in flavors.iter().enumerate() {
                    xf[(ix * n + iq) * flavors.len() + ifl] =
                        (0.4 * xv.ln() - 0.13 * q2v.ln() + 0.05 * fl as f64).exp();
                }
            }
        }
        let member = PdfMember::from_subgrids(vec![SubGrid {
            x,
            q2,
            flavors: flavors.clone(),
            xf,
        }])
        .with_force_positive(2);

        let mut row = [0.0; FLAVOR_SLOTS];
        // In the grid, below its x floor, above its Q² ceiling, and below its
        // Q² floor — every branch the reading can take.
        for &(xv, q2v) in &[(0.011, 55.0), (1e-7, 55.0), (0.011, 1e9), (0.011, 0.1)] {
            member.xfx_all(xv, q2v, &mut row);
            for &fl in &flavors {
                let one = member.xfx_q2(fl, xv, q2v);
                let all = row[flavor_slot(fl).unwrap()];
                assert_eq!(
                    one.to_bits(),
                    all.to_bits(),
                    "fl={fl} x={xv} Q²={q2v}: xfx_q2 {one:e} vs row {all:e}"
                );
            }
        }
    }

    /// A point with no reading refuses through both entry points alike.
    #[test]
    fn a_row_is_refused_where_a_single_reading_is() {
        let member = PdfMember::from_subgrids(vec![sample_subgrid()]);
        let mut row = [0.0; FLAVOR_SLOTS];
        assert!(member.try_xfx_all(f64::NAN, 50.0, &mut row).is_err());
        assert!(member.try_xfx_all(-0.1, 50.0, &mut row).is_err());
        assert!(member.try_xfx_all(0.2, -1.0, &mut row).is_err());
        // Above the last x knot the continuation has nothing to continue into,
        // for a whole row exactly as for one flavor.
        assert!(member.try_xfx_all(2.0, 50.0, &mut row).is_err());
        assert!(member.try_xfx_q2(1, 2.0, 50.0).is_err());
    }

    #[test]
    fn gluon_zero_aliases_to_21() {
        assert_eq!(normalize_flavor_pdg(0), 21);
        assert_eq!(normalize_flavor_pdg(21), 21);
        assert_eq!(normalize_flavor_pdg(-1), -1);
    }

    /// The slot map is a bijection on the codes a density is tabulated for, the
    /// gluon alias included — a collision would silently make two flavors read
    /// each other's luminosity.
    #[test]
    fn flavor_slots_are_distinct_and_alias_the_gluon() {
        let codes: Vec<i32> = (-6..=-1).chain(1..=6).chain([21, 22]).collect();
        let mut seen = std::collections::BTreeSet::new();
        for &pdg in &codes {
            let slot = flavor_slot(pdg).unwrap_or_else(|| panic!("no slot for {pdg}"));
            assert!(slot < FLAVOR_SLOTS);
            assert!(seen.insert(slot), "slot {slot} claimed twice, at {pdg}");
        }
        assert_eq!(flavor_slot(0), flavor_slot(21));
        assert_eq!(flavor_slot(23), None);
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
