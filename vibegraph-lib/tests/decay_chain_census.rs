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
//!   lines lead to exactly the final-state blocks MadGraph put the decays on, each line
//!   the particle MadGraph decays there. MadGraph
//!   does not permute identical particles between decays (it divides by
//!   `identical_decay_chain_factor` instead), so those diagrams are the ones it has; the
//!   rest are the permutations, whose count is reported;
//! - per subprocess, the symmetry factor: here every arrangement of identical particles
//!   between decays is a diagram and the final state's `Π n!` divides, there each
//!   diagram stays on its blocks and MadGraph's `identical_particle_factor` divides. The
//!   two agree when each of MadGraph's diagrams stands for `Π n! / factor` of the
//!   stitched ones: `stitched · factor = on_blocks · Π n!`. A graph stitched twice (one
//!   diagram counted twice) breaks it, as does a missing or doubled `1/n!`;
//! - per card, how many subprocesses two decay assignments reach (`g g > u u~ c c~` from
//!   `g g > z j j, z > j j`, or from two lines whose resonances differ): each is two
//!   entries on both sides, one per assignment;
//! - where MadGraph refuses, stitching refuses too; where MadGraph drops a decay with a
//!   warning (its particle is in no core subprocess), stitching refuses by design.
//!
//! Each card is stitched under the model it imports, the built-in SM at its restriction;
//! a card needing any other model cannot be banked here.
//!
//! What it cannot see: which diagram is which beyond the forced lines' final states, and
//! anything past the diagram list. The stitched diagrams themselves are checked against
//! the undecayed final state in the library's stitching tests. Nor *why* a card is
//! refused: where MadGraph refuses or drops a decay, a refusal here for any reason
//! passes, so a card refused for the wrong reason is indistinguishable from one refused
//! for MadGraph's.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
/// the `(start, length, decaying particle)` block of final-state positions each decay's
/// products take, nested decays included.
fn expand(entry: &Value) -> (Vec<i64>, Vec<Block>) {
    let mut decays: Vec<&Value> = entry["decays"].as_array().expect("decays").iter().collect();
    let mut out = Vec::new();
    let mut blocks = Vec::new();
    for id in ids(&entry["final"]) {
        match decays.iter().position(|d| ids(&d["initial"])[0] == id) {
            Some(k) => {
                let (products, inner) = expand(decays.remove(k));
                let start = out.len();
                blocks.push((start, products.len(), id));
                blocks.extend(inner.into_iter().map(|(s, n, p)| (s + start, n, p)));
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
type Banked = (usize, Vec<Block>, u64);

/// A decay's products in the final state: first position, count, and the PDG code of
/// the particle decaying into them.
type Block = (usize, usize, i64);

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

/// The stitched diagrams of `set` whose forced lines lead to exactly `blocks`, each line
/// the block's particle up to charge conjugation (a line's orientation is not read here).
fn unpermuted(set: &DiagramSet, blocks: &[Block], model: &UFOModel) -> usize {
    let mut blocks: Vec<Block> = blocks.iter().map(|&(s, n, p)| (s, n, p.abs())).collect();
    blocks.sort_unstable();
    set.diagrams
        .iter()
        .filter(|d| {
            let mut sides: Vec<Block> = (0..d.props.len())
                .filter(|&p| d.props[p].onshell == OnShell::Forced)
                .filter_map(|p| {
                    let side = d.final_state_side(PropIdx(p))?;
                    let start = side[0].0 - d.n_in;
                    let contiguous = side.windows(2).all(|w| w[1].0 == w[0].0 + 1);
                    let pdg = model.particle(d.props[p].particle).pdg_code.abs();
                    contiguous.then_some((start, side.len(), pdg))
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

/// The model a card imports: the built-in SM at the card's restriction (the default
/// when it imports none). The census has no other model.
fn model_for(card: &str) -> Arc<UFOModel> {
    let ast = parse_proc_card_ast(card).expect("a census card parses");
    let import = check_supported(&ast).ok().and_then(|c| c.model);
    let Some(import) = import else {
        return sm_model(SMRestrict::Default);
    };
    assert_eq!(import.name, "sm", "the census reads only the built-in SM");
    let restrict = SMRestrict::from_suffix(import.restrict_variant.as_deref())
        .unwrap_or_else(|| panic!("no SM restriction {:?}", import.restrict_variant));
    sm_model(restrict)
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

/// How many particle contents (sorted initial, sorted final) more than one of `keys`
/// has.
fn shared_contents<'a>(keys: impl Iterator<Item = &'a Key>) -> usize {
    let mut counts: BTreeMap<Key, usize> = BTreeMap::new();
    for (initial, final_state) in keys {
        let mut sorted = final_state.clone();
        sorted.sort_unstable();
        *counts.entry((initial.clone(), sorted)).or_default() += 1;
    }
    counts.values().filter(|&&n| n > 1).count()
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
    let doc = reference();
    let cases = doc["cases"].as_array().expect("cases");
    assert!(!cases.is_empty(), "the census banks no cards");
    let mut failures = Vec::new();
    let (mut matched, mut refused, mut subprocesses, mut shared) = (0, 0, 0, 0);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let card = case["card"].as_str().unwrap();
        let model = model_for(card);
        let pdg = |name: &str| model.particles[name].pdg_code;
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
                    let n = unpermuted(set, blocks, &model);
                    if n != *count {
                        ok = false;
                        failures.push(format!(
                            "{name} {key:?}: MadGraph {count} diagrams, here {n} on its blocks \
                             ({} in all)",
                            set.diagrams.len()
                        ));
                    }
                    let here = identical_factor(&key.1);
                    if set.diagrams.len() as u64 * factor != n as u64 * here {
                        ok = false;
                        failures.push(format!(
                            "{name} {key:?}: {} stitched diagrams over Π n! = {here}, but \
                             MadGraph's {count} over its factor {factor} with {n} here on its \
                             blocks: the symmetry factors differ by {}/{}",
                            set.diagrams.len(),
                            set.diagrams.len() as u64 * factor,
                            n as u64 * here
                        ));
                    }
                    report.push(format!(
                        "{:?}: {count} (+{} permuted), factor MadGraph {factor} / here {}",
                        key.1,
                        set.diagrams.len() - n,
                        identical_factor(&key.1)
                    ));
                }
                // A subprocess two decay assignments reach is two entries on each side,
                // their final states in different orders (each decay's products at its
                // particle's position).
                let (here, there) = (shared_contents(ours.keys()), shared_contents(theirs.keys()));
                if here != there {
                    ok = false;
                    failures.push(format!(
                        "{name}: {here} subprocesses reached by two decay assignments here, \
                         {there} in MadGraph"
                    ));
                }
                if ok {
                    matched += 1;
                }
                shared += there;
                eprintln!(
                    "{name}: {there} subprocesses from two decay assignments; {}",
                    report.join("; ")
                );
            }
        }
    }
    eprintln!(
        "{} cards: {matched} stitched and matched ({subprocesses} subprocesses, {shared} of \
         them reached by two decay assignments), {refused} refused as MadGraph refuses or drops",
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
