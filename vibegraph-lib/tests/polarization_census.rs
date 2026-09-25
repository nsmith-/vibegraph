//! Polarized external legs against MadGraph's own generation.
//!
//! `validation/madgraph/polarization_census.json` banks, per card, what
//! MadGraph's `MasterCmd` generates (`validation/madgraph/dump_polarization_census.py`):
//! either the error it raised or, for every subprocess, the `HelasMatrixElement`
//! built from it — its legs with their polarizations after generation, the
//! `NHEL` rows the generated `matrix.f` sums over, and the `IDEN` it divides
//! by. This test enumerates and compiles the same cards and compares:
//!
//! - where MadGraph generated, the same subprocesses, keyed by their legs with
//!   polarizations (so a dropped helicity 0 of a massless boson, or a dropped
//!   subprocess, shows as a different key);
//! - per subprocess, the helicity combinations the evaluator sums over against
//!   MadGraph's `NHEL` table, and `1 / (initial average × symmetry factor)`
//!   against `IDEN` — which is where a polarized incoming leg's average and a
//!   polarization-aware identical-particle factor are pinned;
//! - where MadGraph refused, this side refuses too.
//!
//! Five cards MadGraph accepts are refused here on purpose, each named in
//! [`DELIBERATE_REFUSALS`] with the reason.
//!
//! What the comparison cannot see: the helicity combinations are compared as
//! sets of `(pdg, final, helicity)` multisets, so which of two identical legs
//! carries which helicity is invisible, and nothing here evaluates an
//! amplitude — that is the amplitude gate's business.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;
use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
use vibegraph::hadronic::{initial_spin_color_average, outgoing_symmetry_factor};
use vibegraph::helas::eval::AmplitudeEvaluator;
use vibegraph::ufo::sm::{sm_model, SMRestrict};
use vibegraph::ufo::{EvaluatedModel, UFOModel};

/// Cards MadGraph generates and this side refuses, with why.
const DELIBERATE_REFUSALS: &[(&str, &str)] = &[
    (
        "ee_z2_h",
        "helicity 2 is no state of a spin-one particle; MadGraph hands NHEL = 2 to vxxxxx",
    ),
    (
        "ee_z00_h",
        "helicity 0 listed twice; MadGraph sums the same state twice",
    ),
    (
        "ee_z_hR",
        "a scalar has no helicity +1; MadGraph's sxxxxx ignores NHEL, so the card \
         silently means the unpolarized one",
    ),
    (
        "decay_tL_wp_b",
        "the decaying particle is at rest, where its helicity is a spin projection on the \
         axis the wavefunction routine picks; no amplitude row pins that convention",
    ),
    (
        "add_overlap",
        "the two lines both count z{0} h; MadGraph generates both and the sum \
         double-counts the longitudinal state",
    ),
];

fn reference() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/madgraph/polarization_census.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    serde_json::from_str(&text).expect("polarization_census.json is JSON")
}

/// A leg as the key sees it: PDG code and its polarization list (empty for an
/// unpolarized leg).
type KeyLeg = (i64, Vec<i64>);
/// A subprocess: its sorted initial and sorted final legs.
type Key = (Vec<KeyLeg>, Vec<KeyLeg>);
/// One helicity combination as a multiset of `(pdg, final, helicity)`.
type HelRow = Vec<(i64, bool, i64)>;

struct Summary {
    helicities: BTreeSet<HelRow>,
    iden: f64,
}

fn mg_census(case: &Value) -> BTreeMap<Key, Summary> {
    let mut out = BTreeMap::new();
    for sub in case["subprocesses"].as_array().expect("subprocesses") {
        let legs: Vec<(i64, bool, Vec<i64>)> = sub["legs"]
            .as_array()
            .expect("legs")
            .iter()
            .map(|l| {
                (
                    l[0].as_i64().unwrap(),
                    l[1].as_bool().unwrap(),
                    l[2].as_array()
                        .unwrap()
                        .iter()
                        .map(|h| h.as_i64().unwrap())
                        .collect(),
                )
            })
            .collect();
        let key = key_of(&legs);
        let helicities = sub["helicities"]
            .as_array()
            .expect("helicities")
            .iter()
            .map(|row| {
                let mut r: HelRow = legs
                    .iter()
                    .zip(row.as_array().unwrap())
                    .map(|((pdg, fin, _), h)| (*pdg, *fin, h.as_i64().unwrap()))
                    .collect();
                r.sort_unstable();
                r
            })
            .collect();
        let iden = sub["iden"].as_f64().expect("iden");
        assert!(
            out.insert(key.clone(), Summary { helicities, iden })
                .is_none(),
            "MadGraph lists {key:?} twice"
        );
    }
    out
}

fn key_of(legs: &[(i64, bool, Vec<i64>)]) -> Key {
    let side = |fin: bool| {
        let mut v: Vec<KeyLeg> = legs
            .iter()
            .filter(|l| l.1 == fin)
            .map(|l| (l.0, l.2.clone()))
            .collect();
        v.sort();
        v
    };
    (side(false), side(true))
}

fn our_census(
    sets: &[DiagramSet],
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> BTreeMap<Key, Summary> {
    let mut out = BTreeMap::new();
    for set in sets.iter().filter(|s| !s.diagrams.is_empty()) {
        let eval = AmplitudeEvaluator::compile(set, model).expect("the subprocess compiles");
        let n_in = eval.n_in();
        let legs: Vec<(i64, bool, Vec<i64>)> = eval
            .external_particles()
            .iter()
            .zip(eval.polarizations())
            .enumerate()
            .map(|(i, (&id, pol))| {
                (
                    model.particle(id).pdg_code,
                    i >= n_in,
                    pol.iter().flatten().map(|&h| h as i64).collect(),
                )
            })
            .collect();
        let helicities = eval
            .helicities()
            .iter()
            .map(|row| {
                let mut r: HelRow = legs
                    .iter()
                    .zip(row)
                    .map(|((pdg, fin, _), &h)| (*pdg, *fin, h as i64))
                    .collect();
                r.sort_unstable();
                r
            })
            .collect();
        let iden = 1.0
            / (initial_spin_color_average(&eval, model, evaluated)
                * outgoing_symmetry_factor(&eval));
        out.insert(key_of(&legs), Summary { helicities, iden });
    }
    out
}

fn ours(card: &str, model: &UFOModel) -> Result<Vec<DiagramSet>, String> {
    let parsed = parse_proc_card(card, &ParsingOptions::default()).map_err(|e| e.to_string())?;
    generate_from_proc_card(&parsed, model).map_err(|e| e.to_string())
}

#[test]
fn polarized_subprocesses_match_madgraph() {
    let reference = reference();
    let model = sm_model(SMRestrict::Default);
    let evaluated = EvaluatedModel::from_model(model.clone());
    let mut failures = Vec::new();
    let (mut generated, mut refused, mut deliberate) = (0, 0, 0);
    for case in reference["cases"].as_array().expect("cases") {
        let name = case["name"].as_str().unwrap();
        let card = case["card"].as_str().unwrap();
        let result = ours(card, &model);
        if let Some((_, why)) = DELIBERATE_REFUSALS.iter().find(|(n, _)| *n == name) {
            assert!(
                case.get("error").is_none(),
                "[{name}] listed as a deliberate refusal but MadGraph refuses it too"
            );
            match result {
                Err(e) => {
                    println!("  {name}: refused here, generated by MadGraph ({why}): {e}");
                    deliberate += 1;
                }
                Ok(_) => failures.push(format!("[{name}] should be refused: {why}")),
            }
            continue;
        }
        if let Some(error) = case.get("error") {
            match result {
                Err(e) => {
                    println!("  {name}: both refuse — MadGraph {error}; here {e}");
                    refused += 1;
                }
                Ok(_) => failures.push(format!(
                    "[{name}] MadGraph refuses ({error}), this side generates"
                )),
            }
            continue;
        }
        let sets = match result {
            Ok(sets) => sets,
            Err(e) => {
                failures.push(format!(
                    "[{name}] MadGraph generates, this side refuses: {e}"
                ));
                continue;
            }
        };
        let theirs = mg_census(case);
        let mine = our_census(&sets, &model, &evaluated);
        let (tk, mk): (BTreeSet<&Key>, BTreeSet<&Key>) =
            (theirs.keys().collect(), mine.keys().collect());
        if tk != mk {
            failures.push(format!(
                "[{name}] subprocesses differ: MadGraph {tk:?}, here {mk:?}"
            ));
            continue;
        }
        for (key, t) in &theirs {
            let m = &mine[key];
            if m.helicities != t.helicities {
                failures.push(format!(
                    "[{name}] {key:?}: helicity sets differ ({} here, {} in MadGraph)",
                    m.helicities.len(),
                    t.helicities.len()
                ));
            }
            if (m.iden - t.iden).abs() > 1e-12 * t.iden {
                failures.push(format!(
                    "[{name}] {key:?}: IDEN {} here, {} in MadGraph",
                    m.iden, t.iden
                ));
            }
        }
        println!(
            "  {name}: {} subprocess(es) agree on helicities and IDEN",
            theirs.len()
        );
        generated += 1;
    }
    println!("{generated} generated, {refused} refused by both, {deliberate} refused here only");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert_eq!(deliberate, DELIBERATE_REFUSALS.len());
}
