//! LHAPDF6 grid access: parsed set metadata, per-member subgrids, PDG flavor
//! indexing, and log-bicubic `x·f(x, Q²)` interpolation.
//!
//! Interpolation lives behind the [`interp`] seam ([`interp::Bicubic2D`]); the
//! backend that matches LHAPDF6 (and hence MadGraph) is [`interp::LogBicubic`].
//!
//! A set also carries the strong coupling it was fitted at ([`alphas::GridAlphaS`]),
//! which is the coupling a run reading these densities has to use.

pub mod alphas;
pub mod grid;
pub mod interp;

use std::path::{Path, PathBuf};

use crate::helas::repr::Real;
use grid::{GridError, SetInfo, SubGrid};
use interp::{Bicubic2D, LogBicubic, OutOfRange};

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
        let interp = LogBicubic::build(&subgrids);
        Ok(PdfMember { subgrids, interp })
    }
}

/// One PDF member's parsed subgrids (in file order) plus its precomputed
/// log-bicubic interpolator.
#[derive(Debug)]
pub struct PdfMember {
    pub subgrids: Vec<SubGrid>,
    interp: LogBicubic,
}

impl PdfMember {
    /// Build a member directly from parsed subgrids (precomputing the
    /// interpolator). Mainly useful for tests with in-memory grids.
    pub fn from_subgrids(subgrids: Vec<SubGrid>) -> Self {
        let interp = LogBicubic::build(&subgrids);
        PdfMember { subgrids, interp }
    }

    /// `x·f(x, Q²)` for PDG code `pdg` (0 aliases the gluon 21), interpolated
    /// with the LHAPDF-matching log-bicubic scheme. Returns
    /// [`OutOfRange`](interp::OutOfRange) if the point lies outside every
    /// subgrid's support (extrapolation is a deliberate non-goal).
    pub fn try_xfx_q2<F: Real>(&self, pdg: i32, x: F, q2: F) -> Result<F, OutOfRange> {
        self.interp.xfx_q2(pdg, x, q2)
    }

    /// Like [`PdfMember::try_xfx_q2`] but panics on an out-of-grid point.
    pub fn xfx_q2<F: Real>(&self, pdg: i32, x: F, q2: F) -> F {
        self.try_xfx_q2(pdg, x, q2)
            .unwrap_or_else(|e| panic!("{e}"))
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
