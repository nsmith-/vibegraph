//! Squared-order constraints against MadGraph's split-order standalone output, per
//! coupling-order class, per helicity and per colour flow.
//!
//! Each table under `validation/madgraph/squared_orders/` (written by
//! `validation/madgraph/gen_squared_order_jamps.py`) carries, at a few RAMBO points
//! and for every helicity of MadGraph's NHEL table, the split JAMPs
//! `JAMP(flow, M)` of every amplitude split-order class `M` and the per-squared-order
//! contributions `RES(I)` of MadGraph's `MATRIX`, with the class and squared-order
//! tables, the `CHOSEN_SO_CONFIGS` mask and the param card. This side enumerates the
//! same process line, binds the same card, and compares:
//!
//! * **the diagram count**, against MadGraph's `NGRAPHS` (one graph per diagram on
//!   these processes), which pins the generation-time filter;
//! * **the classes**, as order vectors in the constrained names, both ways;
//! * **every `(class, flow)` JAMP**, under one complex constant fitted over every
//!   entry of the table, which must have modulus one (the two programs differ by a
//!   process-wide phase convention). A JAMP this side does not compile — a class
//!   that reaches no diagram of a flow — must be zero in MadGraph;
//! * **every `RES(I)`**, rebuilt from this side's JAMPs and colour matrix over the
//!   class pairs whose orders sum to `I`'s;
//! * **the per-helicity |M|²** this side's evaluator returns, against the sum of
//!   MadGraph's chosen `RES(I)`, which is the selection itself.
//!
//! What it provably cannot see: the process-wide phase; a split of one class's JAMP
//! into diagrams that compensate one another at every banked point; helicities and
//! regions the banked points do not visit. What it does see that a full-|M|² row
//! cannot: an assignment of diagrams to the wrong class that preserves the sum, and a
//! wrong class pair selected or dropped, since every class and every pair is
//! compared on its own.

mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use common::{generate_with, sm_model};
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::repr::C;
use vibegraph::helas::LorentzVector;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::{EvaluatedModel, UFOModel};

/// Relative deviation allowed per entry, against the table's largest entry of the
/// same kind. Both sides sum the same few dozen diagrams in double precision.
const REL_TOL: f64 = 1e-12;

/// Every banked table.
const TABLES: &[&str] = &["ee_to_ttx_np1", "ee_to_ttx_np2", "uux_to_uux_qcd2"];

fn table(key: &str) -> serde_json::Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/madgraph/squared_orders")
        .join(format!("{key}.json"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

fn complex(v: &serde_json::Value) -> C<f64> {
    C::new(v[0].as_f64().unwrap(), v[1].as_f64().unwrap())
}

fn ints(v: &serde_json::Value) -> Vec<i64> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_i64().unwrap())
        .collect()
}

fn model(key: &str, t: &serde_json::Value) -> Arc<UFOModel> {
    match t["model"].as_str().unwrap() {
        "sm" => sm_model(),
        _ => {
            let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
            let dir = repo.join(t["ufo"]["dir"].as_str().unwrap());
            let card = repo.join(t["ufo"]["restrict_card"].as_str().unwrap());
            UFOModel::load(&dir, Some(&card)).unwrap_or_else(|e| panic!("{key}: {e}"))
        }
    }
}

/// The worst deviations of one table, each relative to the table's largest entry
/// of its kind.
struct Measured {
    g: C<f64>,
    worst_jamp: (f64, String),
    worst_res: (f64, String),
    worst_m2: (f64, String),
}

fn measure(key: &str) -> Measured {
    measure_with(key, false)
}

/// [`measure`], with MadGraph's classes paired to ours rotated by one when
/// `rotate` is set: the comparison's own control, a class assignment that is
/// wrong while every flow's sum over the classes is unchanged.
fn measure_with(key: &str, rotate: bool) -> Measured {
    let t = table(key);
    let model = model(key, &t);
    let card: ParamCard = t["param_card"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        .parse()
        .unwrap();
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &card).unwrap();
    let process = t["process"].as_str().unwrap();
    let set = generate_with(process, model.as_ref()).remove(0);
    assert_eq!(
        set.diagrams.len() as u64,
        t["n_graphs"].as_u64().unwrap(),
        "{key}: diagrams against MadGraph's NGRAPHS"
    );
    let evaluator = AmplitudeEvaluator::compile(&set, model.as_ref()).unwrap();
    let n_flows = t["n_flows"].as_u64().unwrap() as usize;
    assert_eq!(evaluator.n_flows(), n_flows, "{key}: NCOLOR");

    // MadGraph's classes against ours, both ways.
    let (selection, classes) = evaluator
        .order_classes()
        .unwrap_or_else(|| panic!("{key}: the evaluator is not split"));
    let split_orders: Vec<String> = t["split_orders"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_owned())
        .collect();
    let names: Vec<String> = selection
        .constraints
        .iter()
        .map(|c| c.order.clone())
        .collect();
    assert_eq!(names, split_orders, "{key}: the split orders");
    let amp_orders: Vec<Vec<i64>> = t["amp_orders"]
        .as_array()
        .unwrap()
        .iter()
        .map(ints)
        .collect();
    let mut ours_by_orders: BTreeMap<Vec<i64>, usize> = BTreeMap::new();
    for (c, k) in classes.iter().enumerate() {
        ours_by_orders.insert(k.clone(), c);
    }
    let theirs: std::collections::BTreeSet<&Vec<i64>> = amp_orders.iter().collect();
    let ours: std::collections::BTreeSet<&Vec<i64>> = classes.iter().collect();
    assert_eq!(ours, theirs, "{key}: the amplitude split-order classes");
    let mut class_of_m: Vec<usize> = amp_orders.iter().map(|o| ours_by_orders[o]).collect();
    if rotate {
        class_of_m.rotate_left(1);
    }

    let sq_orders: Vec<Vec<i64>> = t["sq_orders"]
        .as_array()
        .unwrap()
        .iter()
        .map(ints)
        .collect();
    let chosen: Vec<bool> = t["chosen"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_bool().unwrap())
        .collect();

    let helicities: Vec<Vec<i32>> = t["helicities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| ints(h).into_iter().map(|x| x as i32).collect())
        .collect();
    let ours_h: std::collections::BTreeSet<&Vec<i32>> = evaluator.helicities().iter().collect();
    let theirs_h: std::collections::BTreeSet<&Vec<i32>> = helicities.iter().collect();
    assert_eq!(
        ours_h, theirs_h,
        "{key}: the helicity sums run over different sets"
    );

    let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);
    let mut scratch = bound.scratch_space();
    let slots = evaluator.jamp_slots();
    let jamp_cf: Vec<f64> = evaluator
        .jamp_cf()
        .iter()
        .map(|r| *r.numer() as f64 / *r.denom() as f64)
        .collect();
    let cf: Vec<f64> = evaluator
        .cf_matrix()
        .iter()
        .map(|r| *r.numer() as f64 / *r.denom() as f64)
        .collect();

    let mut jamps: Vec<(C<f64>, C<f64>, String)> = Vec::new();
    let mut res: Vec<(f64, f64, String)> = Vec::new();
    let mut m2: Vec<(f64, f64, String)> = Vec::new();
    for (pi, point) in t["points"].as_array().unwrap().iter().enumerate() {
        let momenta: Vec<LorentzVector<f64>> = point["momenta"]
            .as_array()
            .unwrap()
            .iter()
            .map(|k| {
                let c: Vec<f64> = k
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_f64().unwrap())
                    .collect();
                LorentzVector::new(c[0], c[1], c[2], c[3])
            })
            .collect();
        let total = bound.eval_m2(&momenta, &mut scratch);
        let mut selected_total = 0.0;
        for (hi, entry) in point["entries"].as_array().unwrap().iter().enumerate() {
            let hel = &helicities[hi];
            let what = format!("point {pi}, helicity {hel:?}");
            let ours_j = bound.run_flows(&momenta, hel, &mut scratch);
            assert_eq!(ours_j.len(), slots.len(), "{key}: one JAMP per slot");
            // Ours per (class, flow), zero where no JAMP is compiled.
            let n_classes = classes.len();
            let mut grid = vec![C::new(0.0, 0.0); n_classes * n_flows];
            for (&(c, f), &j) in slots.iter().zip(&ours_j) {
                grid[c * n_flows + f] = j;
            }
            let mg_jamps = entry["jamps"].as_array().unwrap();
            for (m, per_flow) in mg_jamps.iter().enumerate() {
                let c = class_of_m[m];
                for (f, v) in per_flow.as_array().unwrap().iter().enumerate() {
                    jamps.push((
                        complex(v),
                        grid[c * n_flows + f],
                        format!("{what}, class {:?}, flow {f}", amp_orders[m]),
                    ));
                }
            }
            // RES(I) from our JAMPs, MadGraph's contraction over the class pairs whose
            // orders sum to I's.
            let mg_res: Vec<f64> = entry["res"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap())
                .collect();
            for (i, sq) in sq_orders.iter().enumerate() {
                let mut sum = 0.0;
                for (a, ka) in classes.iter().enumerate() {
                    for (b, kb) in classes.iter().enumerate() {
                        let total: Vec<i64> = ka.iter().zip(kb).map(|(x, y)| x + y).collect();
                        if &total != sq {
                            continue;
                        }
                        for fi in 0..n_flows {
                            let mut z = C::new(0.0, 0.0);
                            for fj in 0..n_flows {
                                z += grid[a * n_flows + fj] * cf[fj * n_flows + fi];
                            }
                            sum += (z * grid[b * n_flows + fi].conj()).re;
                        }
                    }
                }
                res.push((mg_res[i], sum, format!("{what}, RES for orders {sq:?}")));
            }
            let selected: f64 = mg_res
                .iter()
                .zip(&chosen)
                .filter(|(_, &c)| c)
                .map(|(r, _)| r)
                .sum();
            selected_total += selected;
            // This helicity's |M|² through the evaluator's own contraction weights.
            let n = slots.len();
            let mut contracted = 0.0;
            for i in 0..n {
                let mut z = C::new(0.0, 0.0);
                for (j, jj) in ours_j.iter().enumerate() {
                    z += jj * jamp_cf[j * n + i];
                }
                contracted += (z * ours_j[i].conj()).re;
            }
            m2.push((selected, contracted, format!("{what}, selected |M|²")));
        }
        m2.push((
            selected_total,
            total,
            format!("point {pi}, helicity-summed |M|²"),
        ));
    }

    // One complex constant over every JAMP entry.
    let (num, den) = jamps
        .iter()
        .fold((C::new(0.0, 0.0), 0.0), |(n, d), (mg, vg, _)| {
            (n + mg.conj() * vg, d + mg.norm_sqr())
        });
    let g = num / den;
    let worst = |entries: &[(f64, f64, String)]| {
        let scale = entries.iter().map(|e| e.0.abs()).fold(0.0, f64::max);
        entries
            .iter()
            .map(|(a, b, w)| ((a - b).abs() / scale, w.clone()))
            .fold(
                (0.0, String::new()),
                |acc, x| if x.0 > acc.0 { x } else { acc },
            )
    };
    let scale = jamps.iter().map(|e| e.0.norm()).fold(0.0, f64::max);
    let worst_jamp = jamps
        .iter()
        .map(|(mg, vg, w)| ((vg - g * mg).norm() / scale, w.clone()))
        .fold(
            (0.0, String::new()),
            |acc, x| if x.0 > acc.0 { x } else { acc },
        );
    Measured {
        g,
        worst_jamp,
        worst_res: worst(&res),
        worst_m2: worst(&m2),
    }
}

#[test]
fn squared_order_classes_match_madgraph_per_helicity_and_flow() {
    let mut failures = Vec::new();
    for key in TABLES {
        let m = measure(key);
        eprintln!(
            "{key}: G = {:.15} {:+.15}i, worst JAMP {:.3e} ({}), worst RES {:.3e} ({}), worst \
             |M|² {:.3e} ({})",
            m.g.re,
            m.g.im,
            m.worst_jamp.0,
            m.worst_jamp.1,
            m.worst_res.0,
            m.worst_res.1,
            m.worst_m2.0,
            m.worst_m2.1
        );
        if (m.g.norm() - 1.0).abs() >= REL_TOL {
            failures.push(format!("{key}: |G| = {}", m.g.norm()));
        }
        for (what, (dev, at)) in [
            ("JAMP", &m.worst_jamp),
            ("RES", &m.worst_res),
            ("|M|²", &m.worst_m2),
        ] {
            if *dev >= REL_TOL {
                failures.push(format!("{key}: {what} {dev:.3e} at {at}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The comparison sees a wrong class assignment that a full-|M|² comparison
/// cannot: with MadGraph's classes paired to ours rotated by one, every flow's
/// JAMP summed over the classes — and so the unsplit amplitude — is the same, and
/// the per-class comparison must fail.
#[test]
fn a_rotated_class_assignment_is_seen() {
    for key in TABLES {
        let m = measure_with(key, true);
        assert!(
            m.worst_jamp.0 > 1e-3,
            "{key}: a rotated class pairing passed at {:.3e}",
            m.worst_jamp.0
        );
    }
}
