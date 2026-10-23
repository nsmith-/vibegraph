//! The automatic WEIGHTED search against MadGraph's own
//! (`MultiProcess.find_optimal_process_orders`, `diagram_generation.py`), on
//! the authored model `validation/ufo/vibegraph_toy_weighted_UFO` and on the SM.
//!
//! The toy has hierarchies QED=2 and NP=1 and five vertices, every one listed
//! with its WEIGHTED per equivalent three-point vertex:
//!
//! | vertex            | orders           | WEIGHTED |
//! |-------------------|------------------|----------|
//! | `fw~ fw vw`       | QED=1            | 2        |
//! | `fw~ fw xw`       | QED=1, NP=1      | 3        |
//! | `xw xw xw xw`     | QED=1            | 2/2 = 1  |
//! | `xw xw yw`        | QED=1            | 2        |
//! | `yw yw yw yw`     | QED=2, NP=3      | 7/2 = 3.5|
//!
//! so MadGraph's cap is `(n_ext - 2) * int(3.5) = 3 (n_ext - 2)`, where a cap
//! of `(n_ext - 2)` times the largest hierarchy would be `2 (n_ext - 2)`. Every
//! vertex carries QED, so MadGraph's particle hierarchy classes
//! (`Model.get_particles_hierarchy`) put every field in the class of value 2,
//! and the search starts at `get_minimum_WEIGHTED` = the sum of the `n_ext - 2`
//! largest leg values = `2 (n_ext - 2)`.

mod common;

use std::path::PathBuf;

use vibegraph::diagrams::{
    generate_from_proc_card, parse_proc_card, DiagramError, DiagramSet, ParsingOptions,
};
use vibegraph::ufo::UFOModel;

/// The refusal-carrying counterpart of `common::generate_with`.
fn try_generate(process: &str, model: &UFOModel) -> Result<Vec<DiagramSet>, DiagramError> {
    let card = parse_proc_card(&format!("generate {process}"), &ParsingOptions::default())
        .expect("the process card parses");
    generate_from_proc_card(&card, model)
}

fn count(process: &str, model: &UFOModel) -> usize {
    common::generate_with(process, model)
        .iter()
        .map(|s| s.diagrams.len())
        .sum()
}

/// `fw~ fw > xw xw`: the only diagrams are the t- and u-channel `fw` exchanges,
/// each with two `fw~ fw xw` vertices, WEIGHTED 2 * 3 = 6. No other tree exists:
/// the model has no `xw xw xw`, `vw xw xw` or `fw~ fw yw` vertex, so nothing
/// can sit in an s-channel. That is two diagrams (the two orderings of the
/// identical scalars along the fermion line).
///
/// MadGraph starts at 4, finds nothing at 4 or 5, leaves its loop at the cap
/// (4 - 2) * 3 = 6 and generates there: 2 diagrams. A search capped at
/// (4 - 2) * 2 = 4 finds none and the process is refused.
#[test]
fn a_two_order_vertex_is_found_above_the_largest_hierarchy() {
    let model = common::toy_weighted_model();
    assert_eq!(count("fw~ fw > xw xw", &model), 2);
}

/// `fw~ fw > xw xw vw`: the fermion line emits two `xw` and one `vw`, in any
/// order along it, 3! = 6 diagrams, each WEIGHTED 3 + 3 + 2 = 8; there is no
/// other way to attach `vw` (it couples to `fw` only) and, as above, no
/// s-channel scalar. MadGraph starts at 2 * 3 = 6, finds nothing at 6 or 7,
/// and returns 8 from inside its loop, below the cap (5 - 2) * 3 = 9.
#[test]
fn a_two_order_vertex_is_found_by_the_search_below_the_cap() {
    let model = common::toy_weighted_model();
    assert_eq!(count("fw~ fw > xw xw vw", &model), 6);
}

/// `xw xw > xw xw`: the contact `xw^4` (WEIGHTED 2) and the s-, t- and
/// u-channel `yw` exchanges (two `xw xw yw` vertices, WEIGHTED 4). MadGraph's
/// search starts at 4, not at the lowest WEIGHTED a diagram has, so its first
/// trial already holds all four. A search starting from (n_ext - 2) times the
/// smallest hierarchy (2) stops at the contact term alone.
#[test]
fn the_search_starts_where_madgraph_starts() {
    let model = common::toy_weighted_model();
    assert_eq!(count("xw xw > xw xw", &model), 4);
}

/// A process with no tree at any order is refused, as MadGraph raises
/// `NoDiagramException` after generating at the cap: `vw vw > yw` has no
/// vertex joining `vw` to the scalars.
#[test]
fn a_process_without_diagrams_at_the_cap_is_refused() {
    let model = common::toy_weighted_model();
    match try_generate("vw vw > yw", &model) {
        Err(DiagramError::NoDiagrams { .. }) => {}
        other => panic!("expected NoDiagrams, got {:?}", other.map(|s| s.len())),
    }
}

/// A one-particle process that is not part of a decay chain runs no WEIGHTED
/// search in MadGraph: `find_optimal_process_orders` returns the process's
/// (empty) orders unchanged, so every order is generated.
///
/// `z > u u~ d d~` has diagrams at two orders. At QCD^2 QED (WEIGHTED 4) the
/// four gluon exchanges; at QED^3 (WEIGHTED 6) the photon, Z and W exchanges.
/// MadGraph's search, were it run, would start at 4 (legs of value 2, 1, 1, 1,
/// 1; the three largest) and stop there. Unbounded, the count must equal a
/// generation whose explicit orders bound nothing, and exceed the WEIGHTED 4
/// one.
#[test]
fn a_top_level_decay_runs_no_weighted_search() {
    let model = common::sm_model();
    let auto = count("z > u u~ d d~", &model);
    let unbounded = count("z > u u~ d d~ QCD<=99 QED<=99", &model);
    let lowest = count("z > u u~ d d~ WEIGHTED<=4", &model);
    assert_eq!(lowest, 4, "the four gluon exchanges");
    assert_eq!(auto, unbounded);
    assert!(auto > lowest, "{auto} diagrams, {lowest} at WEIGHTED<=4");
}

/// Inside a decay chain the decay does run the search (MadGraph flags it
/// `is_decay_chain`, and `find_optimal_process_orders` skips only a
/// one-particle process without that flag): `z > u u~ d d~` as the decay of
/// `e+ e- > z h` keeps its four WEIGHTED-4 gluon exchanges, and the core's
/// single s-channel Z diagram is stitched to each.
#[test]
fn a_decay_inside_a_chain_runs_the_search() {
    let model = common::sm_model();
    let chained = count("e+ e- > z h, z > u u~ d d~", &model);
    let core = count("e+ e- > z h", &model);
    assert_eq!(core, 1);
    assert_eq!(chained, 4);
}

/// The core of a chain that is itself a one-particle process is top-level, and
/// runs no search: `z > t t~ u u~` has 13 diagrams unbounded and 4 at
/// `WEIGHTED<=4` (the gluon exchanges), and with the two top decays attached
/// (each one diagram) the chain keeps all 13.
#[test]
fn a_chain_whose_core_is_a_decay_runs_no_search_on_the_core() {
    let model = common::sm_model();
    let bounded = count("z > t t~ u u~ WEIGHTED<=4", &model);
    let core = count("z > t t~ u u~", &model);
    let chained = count("z > t t~ u u~, t > b w+, t~ > b~ w-", &model);
    assert_eq!((bounded, core), (4, 13));
    assert_eq!(chained, core);
}

/// MadGraph's search drops the final-state gluons from each trial when the
/// gluon sits in the lowest hierarchy class, and lowers the trial's bound by
/// their QCD weight (`find_optimal_process_orders`, `diagram_generation.py`
/// :2040-2059); the bound it returns is then used for the full process.
///
/// SMEFTsim's SM limit, `e+ e- > z g g`: every leg but the gluons is in the
/// QED class (2), the gluons in the QCD class (1), so the search starts at
/// 2 + 2 + 2 = 6. The trial at 6 is `e+ e- > z` at `WEIGHTED<=4`, which has
/// its one `e+ e- Z` vertex (WEIGHTED 2): MadGraph returns 6, finds no diagram
/// of the full process at `WEIGHTED<=6` (the gluon pair couples only through the
/// loop-induced `h g g` vertex, order `SMHLOOP`, hierarchy 99) and raises
/// `NoDiagramException`. A search that trials the full process instead runs on
/// past 6 to the `Z* -> Z h*, h* -> g g` diagram.
#[test]
#[ignore = "passes once the search trials each subprocess with its final-state gluons removed, as MadGraph does; today it trials the full process"]
fn a_final_state_gluon_pair_without_a_qcd_vertex_is_refused() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO");
    let model = UFOModel::load(&dir, Some(&dir.join("restrict_SMlimit_massless.dat")))
        .expect("SMEFTsim's SM limit loads");
    let reduced = count("e+ e- > z WEIGHTED<=4", &model);
    assert_eq!(reduced, 1, "the trial MadGraph makes at its start, 6");
    assert!(
        matches!(
            try_generate("e+ e- > z g g WEIGHTED<=6", &model),
            Err(DiagramError::NoDiagrams { .. })
        ),
        "nothing at the bound MadGraph returns"
    );
    match try_generate("e+ e- > z g g", &model) {
        Err(DiagramError::NoDiagrams { .. }) => {}
        other => panic!("expected NoDiagrams, got {:?}", other.map(|s| s.len())),
    }
}
