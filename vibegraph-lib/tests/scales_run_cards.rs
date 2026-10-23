//! The run cards the banked cross-section references were produced with, read
//! back through [`vibegraph::coupling::scales`].
//!
//! Nothing here touches an event: it asserts what the committed cards compile
//! to, which is the premise every banked sigma depends on. The per-event replay
//! against MadGraph's own scale fields is `validate_scales.rs`.

use std::path::{Path, PathBuf};

use vibegraph::coupling::scales::{ScaleChoice, ScaleEvent};
use vibegraph::proton::ProtonBeams;
use vibegraph::runcard::RunCard;

fn validation_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation")
}

/// The two Drell–Yan cards behind `validation/madgraph/hadronic_sigma_reference.json`
/// fix both scales at `M_Z`. That is what makes those two numbers reproducible
/// without any of the dynamic machinery — and what makes any movement in them a
/// bug in the fixed branch rather than a re-derivation. The other banked runs'
/// cards live with their runs and are `validate_scales.rs`'s subject.
#[test]
fn the_banked_cross_section_cards_are_fixed_scale() {
    let mut checked = 0;
    for name in [
        "dy13_default_run_card.dat",
        "dy13_mmll_run_card.dat",
        "dy_beams_6500_4000_run_card.dat",
    ] {
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
    assert_eq!(checked, 3);
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

/// Every run card committed under `validation/madgraph/`, `tests/data/` and
/// `research/upstream/*/` is accepted by the parser's field enforcement.
///
/// The enforcement refuses a card that moves a recognized-but-unread parameter
/// off MadGraph's default where that would change what this generator produces,
/// so it can only be sound if it rejects nothing a real card carries. The cards
/// are found by name (`*run_card*.dat`), so a card committed there is read
/// without being listed here. The first two directories must each hold a card.
/// A folder under `research/upstream/` holds whatever its upstream report needs,
/// often no run card at all, so those folders are scanned without that demand.
/// Their total must still be non-zero, so the scan cannot silently find nothing.
/// This is the bare-clone half of that check; the banked runs' own cards are
/// read by `validate_scales.rs`, which needs the reference data.
#[test]
fn the_committed_run_cards_are_accepted() {
    fn run_cards(dir: &Path) -> Vec<PathBuf> {
        let mut cards: Vec<PathBuf> = std::fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .map(|e| e.expect("directory entry").path())
            .filter(|p| {
                p.is_file()
                    && p.extension().is_some_and(|x| x == "dat")
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.contains("run_card"))
            })
            .collect();
        cards.sort();
        cards
    }

    let mut committed: Vec<PathBuf> = Vec::new();
    for dir in [
        validation_dir().join("madgraph"),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data"),
    ] {
        let cards = run_cards(&dir);
        assert!(!cards.is_empty(), "{} holds no run card", dir.display());
        committed.extend(cards);
    }

    let upstream = Path::new(env!("CARGO_MANIFEST_DIR")).join("../research/upstream");
    let mut folders: Vec<PathBuf> = std::fs::read_dir(&upstream)
        .expect("research/upstream is committed")
        .map(|e| e.expect("directory entry").path())
        .filter(|p| p.is_dir())
        .collect();
    folders.sort();
    let upstream_cards: Vec<PathBuf> = folders.iter().flat_map(|d| run_cards(d)).collect();
    assert!(
        !upstream_cards.is_empty(),
        "{} holds no run card in any folder",
        upstream.display()
    );
    committed.extend(upstream_cards);

    let mut refused = Vec::new();
    for path in &committed {
        if let Err(e) = RunCard::parse_file(path) {
            refused.push(format!("{}: {e}", path.display()));
        }
    }
    assert!(
        refused.is_empty(),
        "{} of {} committed run cards refused:\n{}",
        refused.len(),
        committed.len(),
        refused.join("\n")
    );
    println!("{} committed run cards accepted", committed.len());
}

/// The unequal-beam Drell-Yan card is the default Drell-Yan card with the second
/// beam at 4000 GeV and nothing else: the same lines apart from `ebeam2` and the
/// two header lines that name the beams. A difference anywhere else would make a
/// disagreement with MadGraph about more than the beams.
///
/// Cannot see: a change that preserves the line count and the lines compared
/// here, as the comparison is line by line and says nothing about the order
/// MadGraph reads them in.
#[test]
fn the_unequal_beam_card_differs_from_the_default_card_in_the_beams_alone() {
    let dir = validation_dir().join("madgraph");
    let read = |name: &str| std::fs::read_to_string(dir.join(name)).expect("run card");
    let default = read("dy13_default_run_card.dat");
    let unequal = read("dy_beams_6500_4000_run_card.dat");
    let (a, b): (Vec<&str>, Vec<&str>) = (default.lines().collect(), unequal.lines().collect());
    assert_eq!(a.len(), b.len(), "the cards differ in length");
    let differing: Vec<(&str, &str)> = a
        .iter()
        .zip(&b)
        .filter(|(x, y)| x != y)
        .map(|(x, y)| (*x, *y))
        .collect();
    let is_header = |line: &str| line.starts_with('#');
    let headers: Vec<_> = differing.iter().filter(|(x, _)| is_header(x)).collect();
    let values: Vec<_> = differing.iter().filter(|(x, _)| !is_header(x)).collect();
    assert_eq!(headers.len(), 2, "header lines that differ: {headers:?}");
    assert_eq!(
        values,
        [&("  6500.0  = ebeam2", "  4000.0  = ebeam2")],
        "the only value that differs is ebeam2"
    );
    assert!(default.ends_with('\n') == unequal.ends_with('\n'));
}

/// Both Drell-Yan cards give the proton integrand the beams it was built with
/// before beams were read from the card: `ProtonBeams::from_run_card` on the
/// equal-beam cards is bit-identical to `ProtonBeams::new(13000/2, 13000/2)`, so
/// a gate that takes its beams from the card moves nothing on them. The
/// unequal-beam card collides at `2√(E₁E₂)`.
///
/// Cannot see: any card not listed, which is why the banked layer's own cells
/// are the evidence for the rest.
#[test]
fn run_card_beams_reproduce_the_constant_the_equal_beam_gates_used() {
    let constant = ProtonBeams::new(13000.0 / 2.0, 13000.0 / 2.0);
    let beams_of = |name: &str| {
        let path = validation_dir().join("madgraph").join(name);
        ProtonBeams::from_run_card(&RunCard::parse_file(&path).expect("run card"))
    };
    for name in ["dy13_default_run_card.dat", "dy13_mmll_run_card.dat"] {
        let beams = beams_of(name);
        for (got, want) in beams.energies().into_iter().zip(constant.energies()) {
            assert_eq!(got.to_bits(), want.to_bits(), "{name}");
        }
        assert_eq!(
            beams.sqrt_s().to_bits(),
            constant.sqrt_s().to_bits(),
            "{name}"
        );
    }
    let unequal = beams_of("dy_beams_6500_4000_run_card.dat");
    assert_eq!(unequal.energies(), [6500.0, 4000.0]);
    assert_eq!(unequal.sqrt_s(), 2.0 * (6500.0f64 * 4000.0).sqrt());
}
