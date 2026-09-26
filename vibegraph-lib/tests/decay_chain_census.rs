//! The decay-chain census against MadGraph's own generation.
//!
//! `validation/madgraph/decay_chain_census.json` banks, per card, what MadGraph
//! generates and combines (`validation/madgraph/dump_decay_chain_census.py`): either the
//! error it raised or every combined matrix element with its diagram count and every
//! (process, decays) it holds, the final state with the decays in place. This test
//! stitches the same cards and compares:
//!
//! - the (process, decays) list: the same subprocesses, each with the same final state in
//!   the same order, decay products in place of their particle;
//! - per subprocess, MadGraph's diagram count against the stitched diagrams whose forced
//!   lines lead to exactly the final-state blocks MadGraph put the decays on. MadGraph
//!   does not permute identical particles between decays (it divides by
//!   `identical_decay_chain_factor` instead), so those diagrams are the ones it has; the
//!   rest are the permutations, whose count is reported;
//! - where MadGraph refuses, stitching refuses too; where MadGraph drops a decay with a
//!   warning (its particle is in no core subprocess), stitching refuses by design.
//!
//! What it cannot see: which diagram is which beyond the forced lines' final states, and
//! anything past the diagram list. The stitched diagrams themselves are checked against
//! the undecayed final state in the library's stitching tests.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;
use vibegraph::diagrams::check::check_supported;
use vibegraph::diagrams::diagram::{OnShell, PropIdx};
use vibegraph::diagrams::parse::parse_proc_card_ast;
use vibegraph::diagrams::{generate_from_proc_card, DiagramError, DiagramSet};
use vibegraph::ufo::sm::{sm_model, SMRestrict};
use vibegraph::ufo::UFOModel;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn reference() -> Value {
    let path = repo().join("validation/madgraph/decay_chain_census.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    serde_json::from_str(&text).expect("decay_chain_census.json is JSON")
}

fn ids(values: &Value) -> Vec<i64> {
    values
        .as_array()
        .expect("an id list")
        .iter()
        .map(|v| v.as_i64().expect("an id"))
        .collect()
}

/// A MadGraph (process, decays) entry's final state with every decay in place
/// (`get_legs_with_decays`: each decay to the first remaining leg of its particle), and
/// the `(start, length)` block of final-state positions each decay's products take,
/// nested decays included.
fn expand(entry: &Value) -> (Vec<i64>, Vec<(usize, usize)>) {
    let mut decays: Vec<&Value> = entry["decays"].as_array().expect("decays").iter().collect();
    let mut out = Vec::new();
    let mut blocks = Vec::new();
    for id in ids(&entry["final"]) {
        match decays.iter().position(|d| ids(&d["initial"])[0] == id) {
            Some(k) => {
                let (products, inner) = expand(decays.remove(k));
                let start = out.len();
                blocks.push((start, products.len()));
                blocks.extend(inner.into_iter().map(|(s, n)| (s + start, n)));
                out.extend(products);
            }
            None => out.push(id),
        }
    }
    assert!(decays.is_empty(), "every decay applies to a leg");
    (out, blocks)
}

/// Subprocess key: sorted initial ids and the in-place final ids.
type Key = (Vec<i64>, Vec<i64>);

/// MadGraph's diagram count, decay blocks and identical-particle factor for one
/// subprocess.
type Banked = (usize, Vec<(usize, usize)>, u64);

/// MadGraph's side: per subprocess, its diagram count, decay blocks and identical
/// particle factor.
fn banked(case: &Value) -> BTreeMap<Key, Banked> {
    let mut out = BTreeMap::new();
    for me in case["matrix_elements"].as_array().expect("matrix elements") {
        let diagrams = me["diagrams"].as_u64().unwrap() as usize;
        let factor = me["identical_particle_factor"].as_u64().unwrap();
        for process in me["processes"].as_array().expect("processes") {
            let mut initial = ids(&process["with_decays"]["initial"]);
            initial.sort_unstable();
            let (final_state, mut blocks) = expand(process);
            assert_eq!(final_state, ids(&process["with_decays"]["final"]));
            blocks.sort_unstable();
            let previous = out.insert((initial, final_state), (diagrams, blocks, factor));
            assert!(previous.is_none(), "a subprocess banked twice");
        }
    }
    out
}

/// The stitched diagrams of `set` whose forced lines lead to exactly `blocks`.
fn unpermuted(set: &DiagramSet, blocks: &[(usize, usize)]) -> usize {
    set.diagrams
        .iter()
        .filter(|d| {
            let mut sides: Vec<(usize, usize)> = (0..d.props.len())
                .filter(|&p| d.props[p].onshell == OnShell::Forced)
                .filter_map(|p| {
                    let side = d.final_state_side(PropIdx(p))?;
                    let start = side[0].0 - d.n_in;
                    let contiguous = side.windows(2).all(|w| w[1].0 == w[0].0 + 1);
                    contiguous.then_some((start, side.len()))
                })
                .collect();
            sides.sort_unstable();
            sides == blocks
        })
        .count()
}

enum Outcome {
    Sets(Vec<DiagramSet>),
    Refused(String),
}

fn stitch(card: &str, model: &UFOModel) -> Outcome {
    let ast = match parse_proc_card_ast(card) {
        Ok(ast) => ast,
        Err(e) => return Outcome::Refused(e.to_string()),
    };
    let card = match check_supported(&ast) {
        Ok(c) => c,
        Err(e) => return Outcome::Refused(e.to_string()),
    };
    match generate_from_proc_card(&card, model) {
        Ok(sets) => Outcome::Sets(sets),
        Err(
            e @ (DiagramError::DecayChain { .. }
            | DiagramError::NotADecay { .. }
            | DiagramError::NoDiagrams { .. }),
        ) => Outcome::Refused(e.to_string()),
        Err(e) => panic!("unexpected enumeration error: {e}"),
    }
}

/// The identical-particle factor of a final state.
fn identical_factor(ids: &[i64]) -> u64 {
    let mut counts: BTreeMap<i64, u64> = BTreeMap::new();
    for &id in ids {
        *counts.entry(id).or_default() += 1;
    }
    counts.values().map(|&n| (1..=n).product::<u64>()).product()
}

#[test]
fn decay_chain_census_matches_madgraph() {
    let model = sm_model(SMRestrict::Default);
    let pdg = |name: &str| model.particles[name].pdg_code;
    let doc = reference();
    let cases = doc["cases"].as_array().expect("cases");
    let mut failures = Vec::new();
    let (mut matched, mut refused, mut subprocesses) = (0, 0, 0);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let card = case["card"].as_str().unwrap();
        let dropped = case["warnings"].as_array().is_some_and(|w| {
            w.iter()
                .any(|m| m.as_str().unwrap().starts_with("Decay without"))
        });
        match (case.get("error"), stitch(card, &model)) {
            (Some(_), Outcome::Refused(_)) => refused += 1,
            (None, Outcome::Refused(_)) if dropped => refused += 1,
            (None, Outcome::Sets(_)) if dropped => failures.push(format!(
                "{name}: MadGraph drops a decay with no core particle; here it must be refused"
            )),
            (Some(err), Outcome::Sets(_)) => failures.push(format!(
                "{name}: MadGraph refuses ({err}), here it stitches"
            )),
            (None, Outcome::Refused(e)) => {
                failures.push(format!("{name}: MadGraph generates, here refused: {e}"))
            }
            (None, Outcome::Sets(sets)) => {
                let theirs = banked(case);
                let mut ours: BTreeMap<Key, &DiagramSet> = BTreeMap::new();
                for set in sets.iter().filter(|s| !s.diagrams.is_empty()) {
                    let mut initial: Vec<i64> = set.particles_in.iter().map(|n| pdg(n)).collect();
                    initial.sort_unstable();
                    let final_state = set.particles_out.iter().map(|n| pdg(n)).collect();
                    assert!(ours.insert((initial, final_state), set).is_none());
                }
                let mut ok = true;
                if ours.keys().ne(theirs.keys()) {
                    ok = false;
                    failures.push(format!(
                        "{name}: subprocesses differ:\n    here     {:?}\n    MadGraph {:?}",
                        ours.keys().collect::<Vec<_>>(),
                        theirs.keys().collect::<Vec<_>>()
                    ));
                }
                let mut report = Vec::new();
                for (key, (count, blocks, factor)) in &theirs {
                    let Some(set) = ours.get(key) else { continue };
                    subprocesses += 1;
                    let n = unpermuted(set, blocks);
                    if n != *count {
                        ok = false;
                        failures.push(format!(
                            "{name} {key:?}: MadGraph {count} diagrams, here {n} on its blocks \
                             ({} in all)",
                            set.diagrams.len()
                        ));
                    }
                    report.push(format!(
                        "{:?}: {count} (+{} permuted), factor MadGraph {factor} / here {}",
                        key.1,
                        set.diagrams.len() - n,
                        identical_factor(&key.1)
                    ));
                }
                if ok {
                    matched += 1;
                }
                eprintln!("{name}: {}", report.join("; "));
            }
        }
    }
    eprintln!(
        "{} cards: {matched} stitched and matched ({subprocesses} subprocesses), {refused} \
         refused as MadGraph refuses or drops",
        cases.len()
    );
    assert!(
        failures.is_empty(),
        "{} of {} cards disagree with MadGraph:\n  {}",
        failures.len(),
        cases.len(),
        failures.join("\n  ")
    );
}
