//! Coupling orders with a negative power, against MadGraph's diagram counts, on
//! the authored model `validation/ufo/vibegraph_toy_negorder_UFO`.
//!
//! The toy writes its scalar's cubic self-coupling twice, as SMEFTsim writes the
//! Higgs's: `V_xw3` at `QED=1` and `V_xw3np` at `{NP: 1, QED: -1}`, a vev
//! insertion whose WEIGHTED is `1 - 2 = -1`. MadGraph sums a diagram's powers with
//! their sign (`Diagram.calculate_orders`, `base_objects.py:2638-2657`), and spends
//! a `<=` bound vertex by vertex while it builds the diagram (`reduce_orders`,
//! `diagram_generation.py:1048-1083`), so a diagram whose total is within the bound
//! is dropped when its negative vertex is added after a prefix that is not.
//!
//! Every count below is the pinned MadGraph checkout's (3.7.1, `generate` and
//! nothing else); a refusal is MadGraph's `NoDiagramException`.

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, DiagramError, ParsingOptions};
use vibegraph::ufo::UFOModel;

fn toy() -> Arc<UFOModel> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/ufo/vibegraph_toy_negorder_UFO");
    UFOModel::load(&dir, None).expect("the negative-order toy loads")
}

/// The diagram count, or `None` for MadGraph's refusal.
fn count(process: &str, model: &UFOModel) -> Option<usize> {
    let card = parse_proc_card(&format!("generate {process}"), &ParsingOptions::default())
        .expect("the process card parses");
    match generate_from_proc_card(&card, model) {
        Ok(sets) => Some(sets.iter().map(|s| s.diagrams.len()).sum()),
        Err(DiagramError::NoDiagrams { .. }) => None,
        Err(e) => panic!("{process}: {e}"),
    }
}

/// The toy's coupling carries the negative power as written.
#[test]
fn a_negative_coupling_order_loads_signed() {
    let model = toy();
    let gc = &model.couplings["GC_X3NP"];
    assert_eq!(gc.orders.get("QED"), Some(&-1));
    assert_eq!(gc.orders.get("NP"), Some(&1));
}

/// `fw~ fw > xw xw`: the t- and u-channel fermion exchanges (QED=2), the
/// s-channel scalar through `V_xw3` (QED=2) and through `V_xw3np` (QED=0, NP=1).
/// Under `QED=0` the last one's total is 0, but MadGraph adds the `fw~ fw xw`
/// vertex (QED=1) first, and refuses; under `WEIGHTED=1` the same vertex's 2
/// exceeds the bound before the vev insertion's -1 brings it back to 1.
#[test]
fn a_bound_is_spent_vertex_by_vertex_in_madgraph_order() {
    let model = toy();
    let cases: &[(&str, Option<usize>)] = &[
        ("fw~ fw > xw xw", Some(4)),
        ("fw~ fw > xw xw NP=0", Some(3)),
        ("fw~ fw > xw xw QED=0", None),
        ("fw~ fw > xw xw QED=1", Some(1)),
        ("fw~ fw > xw xw WEIGHTED=1", None),
        ("fw~ fw > xw xw WEIGHTED=2", Some(1)),
        ("fw~ fw > xw xw xw", Some(31)),
        ("fw~ fw > xw xw xw NP=1", Some(28)),
        ("fw~ fw > xw xw xw QED=1", Some(12)),
        ("fw~ fw > xw xw xw QED=2 NP=1", Some(12)),
        ("fw~ fw > xw xw xw QED=0", None),
        ("xw xw > xw xw", Some(13)),
        ("xw xw > xw xw QED=0", Some(6)),
        ("xw xw > xw xw QED=1", Some(9)),
        ("xw xw > xw xw xw QED=1", Some(94)),
        ("fw~ fw > vw xw xw", Some(18)),
        ("fw~ fw > vw xw xw QED=2", Some(3)),
        ("fw~ fw > vw xw xw QED=1", Some(3)),
        ("vw vw > xw xw QED=1", Some(1)),
        ("vw vw > xw xw xw QED=1", Some(12)),
        ("fw~ fw > vw vw xw xw QED=3 NP=1", Some(28)),
        ("fw~ fw > fw~ fw xw xw QED=2", Some(38)),
    ];
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|&(process, want)| {
            let got = count(process, &model);
            (got != want).then(|| format!("{process}: {got:?}, MadGraph {want:?}"))
        })
        .collect();
    assert!(failures.is_empty(), "{failures:#?}");
}
