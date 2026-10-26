//! The LHAPDF set a `pdlabel = lhapdf` run card names, and its `AlphaS_*` block.
//!
//! Shared by `validate_alphas.rs` (the printed `SCALUP` as the scale) and
//! `validate_scales.rs` (the computed `μR` as the scale): both need the same
//! grid `αs` reading, and a `lhaid` → set-name table transcribed in two places
//! is exactly the kind of thing that rots silently.

use std::path::Path;

use vibegraph::pdf::grid::AlphaSInfo;
use vibegraph::pdf::PdfSet;
use vibegraph::runcard::{PdfChoice, RunCard};

/// The LHAPDF set each `lhaid` names. A run card carries only the id, and the
/// grid `αs` has to come from the set the *densities* come from, so the
/// mapping is stated here rather than inferred from whatever set happens to
/// be unpacked.
const PDF_SET_BY_LHAID: &[(i64, &str)] = &[(247000, "NNPDF23_lo_as_0130_qed")];

/// The `AlphaS_*` metadata of the set a run's beams read, or `None` for a run
/// that reads no LHAPDF set.
///
/// Which set that is comes from [`RunCard::pdf_choice`], the same rule the
/// densities follow, and not from `pdlabel`/`lhaid` read off the card: a
/// fixed-energy card MadGraph writes spells `none` per beam, carries
/// `lhaid = 230000` and no `pdlabel` line, so its resolved `pdlabel` is the
/// default while it reads no density at all. A card whose label is refused
/// gives `None` here, and the refusal surfaces where the coupling is built.
pub fn set_alpha_s_info(card: &RunCard) -> Option<AlphaSInfo> {
    let lhaid = match card.pdf_choice() {
        Ok(PdfChoice::Lhapdf { lhaid }) => lhaid,
        Ok(PdfChoice::None) | Err(_) => return None,
    };
    let name = PDF_SET_BY_LHAID
        .iter()
        .find(|(id, _)| *id == lhaid)
        .map(|(_, name)| *name)
        .unwrap_or_else(|| panic!("no PDF set registered for lhaid {lhaid}"));
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
