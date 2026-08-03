//! The LHAPDF set a `pdlabel = lhapdf` run card names, and its `AlphaS_*` block.
//!
//! Shared by `validate_alphas.rs` (the printed `SCALUP` as the scale) and
//! `validate_scales.rs` (the computed `μR` as the scale): both need the same
//! grid `αs` reading, and a `lhaid` → set-name table transcribed in two places
//! is exactly the kind of thing that rots silently.

use std::path::Path;

use vibegraph::pdf::grid::AlphaSInfo;
use vibegraph::pdf::PdfSet;
use vibegraph::runcard::RunCard;

/// The LHAPDF set each `lhaid` names. A run card carries only the id, and the
/// grid `αs` has to come from the set the *densities* come from, so the
/// mapping is stated here rather than inferred from whatever set happens to
/// be unpacked.
const PDF_SET_BY_LHAID: &[(i64, &str)] = &[(247000, "NNPDF23_lo_as_0130_qed")];

/// The `AlphaS_*` metadata of the set a run's beams read, or `None` for a run
/// that names no LHAPDF set.
pub fn set_alpha_s_info(card: &RunCard) -> Option<AlphaSInfo> {
    if card.pdlabel != "lhapdf" {
        return None;
    }
    let name = PDF_SET_BY_LHAID
        .iter()
        .find(|(id, _)| *id == card.lhaid)
        .map(|(_, name)| *name)
        .unwrap_or_else(|| panic!("no PDF set registered for lhaid {}", card.lhaid));
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/pdf")
        .join(name);
    let set = PdfSet::load(&dir, name).unwrap_or_else(|e| {
        panic!(
            "cannot load PDF set {name} from {}: {e}\n\
             run `pixi run -e madgraph fetch-pdf`",
            dir.display()
        )
    });
    Some(set.info.alpha_s)
}
