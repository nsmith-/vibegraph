//! Which LHAPDF set the banked-run gates read `αs` from, on the run cards
//! MadGraph itself writes. Hermetic: a card that reads no density never
//! touches the set on disk.

use vibegraph::runcard::{PdfChoice, RunCard};

#[path = "common/pdfset.rs"]
mod pdfset;

/// The PDF lines of the banked `ee_to_ee` run card, as MadGraph 3.7.1 writes
/// them for fixed-energy beams: the per-beam template (`none` on both), the
/// hidden `lhaid` at MadGraph's own default, and no `pdlabel` line.
const BANKED_EE_TO_EE_PDF_LINES: &str = "  0\t= lpp1 ! beam 1 type
  0\t= lpp2 ! beam 2 type
     none    = pdlabel1     ! PDF type for beam #1
     none    = pdlabel2     ! PDF type for beam #2
  230000\t= lhaid ! if pdlabel=lhapdf, this is the lhapdf number
";

#[test]
fn a_madgraph_fixed_energy_card_reads_no_set() {
    let card = RunCard::parse(BANKED_EE_TO_EE_PDF_LINES).expect("banked card lines parse");
    assert_eq!(card.pdf_choice().expect("not refused"), PdfChoice::None);
    assert!(
        pdfset::set_alpha_s_info(&card).is_none(),
        "a fixed-energy card names no LHAPDF set for alpha_s"
    );
}
