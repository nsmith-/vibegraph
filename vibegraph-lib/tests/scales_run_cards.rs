//! The run cards the banked cross-section references were produced with, read
//! back through [`vibegraph::coupling::scales`].
//!
//! Nothing here touches an event: it asserts what the committed cards compile
//! to, which is the premise every banked sigma depends on. The per-event replay
//! against MadGraph's own scale fields is `validate_scales.rs`.

use std::path::{Path, PathBuf};

use vibegraph::coupling::scales::{ScaleChoice, ScaleEvent};
use vibegraph::runcard::RunCard;

fn validation_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation")
}

/// The cross-section reference in `validation/madgraph/` was produced with cards
/// that fix both scales at `M_Z`. That is what makes those numbers reproducible
/// without any of the dynamic machinery — and what makes any movement in them a
/// bug in the fixed branch rather than a re-derivation.
#[test]
fn the_banked_cross_section_cards_are_fixed_scale() {
    let mut checked = 0;
    for name in ["dy13_default_run_card.dat", "dy13_mmll_run_card.dat"] {
        let path = validation_dir().join("madgraph").join(name);
        let card = RunCard::parse_file(&path).expect("run card");
        assert!(card.fixed_ren_scale, "{name}: fixed_ren_scale");
        assert!(card.fixed_fac_scale, "{name}: fixed_fac_scale");
        let choice = ScaleChoice::from_run_card(&card).expect("compiled");
        assert!(
            choice.is_fully_fixed(),
            "{name}: both scales should be run-card constants"
        );
        assert!(
            !choice.needs_channels(),
            "{name}: a fixed scale needs no channel forests"
        );
        let scales = choice
            .scales(&ScaleEvent {
                incoming: [[10.0, 0.0, 0.0, 10.0], [10.0, 0.0, 0.0, -10.0]],
                outgoing: &[[10.0, 3.0, 0.0, 4.0], [10.0, -3.0, 0.0, -4.0]],
            })
            .expect("scales");
        assert_eq!(scales.mu_r, card.scale);
        assert_eq!(scales.mu_f, [card.dsqrt_q2fact1, card.dsqrt_q2fact2]);
        checked += 1;
    }
    assert_eq!(checked, 2);
}

/// The parser fixture compiles to the *other* branch, and is the only committed
/// card that does: it leaves `fixed_ren_scale` false, so a run driven from it
/// takes the clustering branch for `μR` while keeping `μF` at `91.188`. That is
/// the point of it — it is written for the parser, not copied from a run, so it
/// is free to cover a combination no reference card does. Pinned here so the two
/// branches keep an example each.
#[test]
fn the_parser_fixture_compiles_to_the_free_scale_branch() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/run_card_parser_fixture.dat");
    let card = RunCard::parse_file(&path).expect("run card");
    assert!(!card.fixed_ren_scale, "fixture already fixes mu_R");
    assert!(card.fixed_fac_scale, "fixture already frees mu_F");
    let choice = ScaleChoice::from_run_card(&card).expect("compiled");
    assert!(!choice.is_fully_fixed());
    assert!(choice.needs_channels());
}
