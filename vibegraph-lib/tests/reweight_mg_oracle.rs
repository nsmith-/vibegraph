//! Reweighting against MadGraph's own reweight module, weight by weight.
//!
//! `validation/madgraph/reweight_mg_reference.json` banks, per row, events
//! vibegraph generated and the weights MadGraph's `ReweightInterface` gave each of
//! them under each hypothesis of a reweight card (`gen_reweight_oracle.py`, which
//! also prints the same comparison over the whole generated file). This test
//! reweights the banked events through [`ReweightPlan`] — the row's own path,
//! polynomial or joint where the row names couplings, and the exact path besides —
//! and compares each weight with MadGraph's.
//!
//! **Informational.** The comparison is measured and printed, not enforced, until
//! it has run against more processes than these rows; what is asserted is that it
//! ran: every banked event found its subprocess, every hypothesis was compared, and
//! the parameters both sides started from are the same (below).
//!
//! # What it holds fixed, and what it is blind to
//!
//! Parameter provenance first: the reference carries the param card MadGraph
//! computed with, and this side binds that same card — MadGraph writes it at seven
//! significant digits (`WH` 6.382339e-03 against the model's 6.38233934e-03) and
//! computes at what it wrote. Its gap to the model's own defaults, which
//! `generate` runs at, is printed and bounded far below anything a hypothesis moves.
//!
//! MadGraph resolves the subprocess from the event's PDG codes, boosts the written
//! momenta itself, compiles its own matrix elements and moves its own parameters
//! through its own param-card machinery, so this sees an error in vibegraph's
//! parameter propagation (a coupling left at its old value), in the hypothesis
//! amplitude, or in the polynomial path's reconstruction of it. It is blind to:
//!
//! * the strong coupling: both read the event's `AQCDUP`, and at fixed QCD order it
//!   cancels in the ratio anyway — the per-event audit in `generate` is what checks
//!   the coupling the denominator was taken at;
//! * `generate`'s subprocess indexing: the subprocess here is recovered from the
//!   banked flavours by this test, not by `generate` (the same audit covers that);
//! * anything below the files' eight printed digits, which bound the comparison at
//!   about `1e-7` relative.

use std::path::Path;
use std::sync::Arc;

use serde_json::Value;
use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
use vibegraph::helas::repr::lorentz::LorentzVector;
use vibegraph::reweight::card::ReweightCard;
use vibegraph::reweight::engine::{ReweightOptions, ReweightPlan};
use vibegraph::reweight::{resolve, resolve_couplings};
use vibegraph::ufo::parameters::ParamNature;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::sm::{sm_model, SMRestrict};
use vibegraph::ufo::{EvaluatedModel, UFOModel};

type V = LorentzVector<f64>;

/// Twice the files' printing: each weight carries eight significant digits.
const PRINTED: f64 = 2e-7;

fn reference() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/madgraph/reweight_mg_reference.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("the banked reference")).unwrap()
}

/// The row's model: the interned Standard Model, or a vendored UFO under its
/// restrict card.
fn model_of(row: &Value) -> Arc<UFOModel> {
    match row["model"].as_str().unwrap() {
        "sm" => sm_model(SMRestrict::Default),
        name => {
            let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../validation/ufo")
                .join(name);
            let restrict = row["restrict"].as_str().expect("a restrict card");
            UFOModel::load(&dir, Some(&dir.join(format!("restrict_{restrict}.dat"))))
                .unwrap_or_else(|e| panic!("load {name}-{restrict}: {e}"))
        }
    }
}

/// MadGraph's param card, as banked, for [`EvaluatedModel::from_model_card`].
fn madgraph_card(card: &Value) -> ParamCard {
    let mut text = String::new();
    let mut block = String::new();
    for entry in card.as_array().unwrap() {
        let name = entry[0].as_str().unwrap();
        let code: Vec<String> = entry[1]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_i64().unwrap().to_string())
            .collect();
        let value = entry[2].as_f64().unwrap();
        if name == "decay" {
            text += &format!("DECAY {} {value:e}\n", code.join(" "));
            block.clear();
        } else {
            if name != block {
                text += &format!("BLOCK {name}\n");
                block = name.to_string();
            }
            text += &format!(" {} {value:e}\n", code.join(" "));
        }
    }
    text.parse().expect("the banked param card parses")
}

/// How many of the banked card's entries address a free external parameter of the
/// model, each checked to have been bound at the card's value.
fn bound_entries(model: &UFOModel, bound: &EvaluatedModel, card: &Value) -> usize {
    let mut n = 0;
    for entry in card.as_array().unwrap() {
        let block = entry[0].as_str().unwrap();
        let code: Vec<i32> = entry[1]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_i64().unwrap() as i32)
            .collect();
        let value = entry[2].as_f64().unwrap();
        let Some(p) = model.params.externals.iter().find(|p| {
            !model.params.zeros.contains(&p.name)
                && matches!(&p.nature, ParamNature::External { lha_block, lha_code, .. }
                    if lha_block.eq_ignore_ascii_case(block) && *lha_code == code)
        }) else {
            continue;
        };
        let got = bound.param_values[&p.name].re;
        assert!(
            (got - value).abs() <= 1e-12 * value.abs(),
            "{} bound at {got}, the card says {value}",
            p.name
        );
        n += 1;
    }
    n
}

/// The largest relative difference between the external parameters MadGraph
/// computed at and the model's own defaults, with its parameter; asserts that
/// every one of them was read.
fn provenance(
    model: &Arc<UFOModel>,
    ours: &EvaluatedModel,
    theirs: &EvaluatedModel,
) -> (f64, String) {
    let mut worst = (0.0f64, String::new());
    for p in &model.params.externals {
        if model.params.zeros.contains(&p.name) {
            continue;
        }
        let (a, b) = (
            ours.param_values[&p.name].re,
            theirs.param_values[&p.name].re,
        );
        let rel = (a - b).abs() / a.abs().max(b.abs()).max(f64::MIN_POSITIVE);
        if rel > worst.0 {
            worst = (rel, p.name.clone());
        }
    }
    worst
}

/// An event's subprocess among `pdgs` (each in its own leg order) and its
/// momenta in that order in the partonic centre of mass, beam 1 along +z.
fn locate(pdgs: &[Vec<i32>], event: &Value) -> Option<(usize, Vec<V>)> {
    let status: Vec<i64> = event["status"]
        .as_array()?
        .iter()
        .map(|s| s.as_i64().unwrap())
        .collect();
    let pdg: Vec<i32> = event["pdg"]
        .as_array()?
        .iter()
        .map(|s| s.as_i64().unwrap() as i32)
        .collect();
    let mom: Vec<V> = event["momenta"]
        .as_array()?
        .iter()
        .map(|p| {
            let c: Vec<f64> = p
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap())
                .collect();
            V::new(c[3], c[0], c[1], c[2])
        })
        .collect();
    let incoming: Vec<usize> = (0..pdg.len()).filter(|&i| status[i] == -1).collect();
    let outgoing: Vec<usize> = (0..pdg.len()).filter(|&i| status[i] == 1).collect();
    for (s, own) in pdgs.iter().enumerate() {
        if own.len() != incoming.len() + outgoing.len() {
            continue;
        }
        let beams = if own[..2] == [pdg[incoming[0]], pdg[incoming[1]]] {
            [incoming[0], incoming[1]]
        } else if own[..2] == [pdg[incoming[1]], pdg[incoming[0]]] {
            [incoming[1], incoming[0]]
        } else {
            continue;
        };
        let mut used = vec![false; outgoing.len()];
        let mut order = beams.to_vec();
        for &want in &own[2..] {
            match (0..outgoing.len()).find(|&k| !used[k] && pdg[outgoing[k]] == want) {
                Some(k) => {
                    used[k] = true;
                    order.push(outgoing[k]);
                }
                None => break,
            }
        }
        if order.len() != own.len() {
            continue;
        }
        // Boost along z into the partonic centre of mass, then turn by π about x
        // if the subprocess's first parton arrived on the second beam: a Lorentz
        // transformation, under which the helicity-summed |M|² is unchanged.
        let (a, b) = (mom[beams[0]], mom[beams[1]]);
        let beta = (a.pz() + b.pz()) / (a.e() + b.e());
        let gamma = 1.0 / (1.0 - beta * beta).sqrt();
        let flip = a.pz() < 0.0;
        let p = order
            .iter()
            .map(|&i| {
                let q = mom[i];
                let e = gamma * (q.e() - beta * q.pz());
                let z = gamma * (q.pz() - beta * q.e());
                if flip {
                    V::new(e, q.px(), -q.py(), -z)
                } else {
                    V::new(e, q.px(), q.py(), z)
                }
            })
            .collect();
        return Some((s, p));
    }
    None
}

struct Comparison {
    events: usize,
    largest: f64,
    beyond: usize,
}

#[test]
fn reweighting_matches_madgraphs_reweight_module_event_by_event() {
    let reference = reference();
    println!(
        "MadGraph {} ({})",
        reference["madgraph_version"].as_str().unwrap(),
        reference["generator"].as_str().unwrap()
    );
    let rows = reference["rows"].as_object().unwrap();
    assert!(!rows.is_empty());
    for (key, row) in rows {
        let model = model_of(row);
        // MadGraph writes its param card at seven significant digits and computes at
        // what it wrote; both sides start from that card.
        let base =
            EvaluatedModel::from_model_card(model.clone(), &madgraph_card(&row["param_card"]));
        let (gap, which) = provenance(&model, &EvaluatedModel::from_model(model.clone()), &base);
        let bound = bound_entries(&model, &base, &row["param_card"]);
        assert!(bound >= 10, "{key}: only {bound} card entries bound");
        println!(
            "{key}: {bound} card entries bound; against the model's defaults the largest \
             relative gap is {gap:.1e} ({which})"
        );

        assert!(
            gap < 1e-6,
            "{key}: MadGraph computed at another parameter point ({which}, {gap:e})"
        );

        let card = parse_proc_card(
            &format!("{}\n", row["process"].as_str().unwrap()),
            &ParsingOptions::default(),
        )
        .unwrap();
        let sets: Vec<DiagramSet> = generate_from_proc_card(&card, &model)
            .unwrap()
            .into_iter()
            .filter(|s| !s.diagrams.is_empty())
            .collect();
        let refs: Vec<&DiagramSet> = sets.iter().collect();

        let mut rw_card = String::new();
        let ids: Vec<String> = row["hypotheses"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| {
                let id = h["id"].as_str().unwrap().to_string();
                rw_card += &format!("launch --rwgt_name={id}\n");
                for line in h["lines"].as_array().unwrap() {
                    rw_card += &format!(" {}\n", line.as_str().unwrap());
                }
                id
            })
            .collect();
        let rw_card: ReweightCard = rw_card.parse().unwrap();
        let launches = resolve(&rw_card, &model).unwrap();

        let mut paths = vec![(
            "exact",
            ReweightOptions {
                exact: true,
                couplings: None,
            },
        )];
        if let Some(couplings) = row["couplings"].as_array() {
            let names: Vec<String> = couplings
                .iter()
                .map(|c| c.as_str().unwrap().to_string())
                .collect();
            paths.push((
                "joint",
                ReweightOptions {
                    exact: false,
                    couplings: Some(resolve_couplings(&model, &names).unwrap()),
                },
            ));
        } else {
            paths.push(("default", ReweightOptions::default()));
        }

        let events = row["events"].as_array().unwrap();
        assert!(!events.is_empty(), "{key}: no banked events");
        for (name, options) in paths {
            let plan = ReweightPlan::new(&refs, &model, &base, launches.clone(), options).unwrap();
            let mut rw = plan.bind();
            let pdgs: Vec<Vec<i32>> = (0..sets.len()).map(|s| rw.pdgs(s).to_vec()).collect();
            let mut stats: Vec<Comparison> = ids
                .iter()
                .map(|_| Comparison {
                    events: 0,
                    largest: 0.0,
                    beyond: 0,
                })
                .collect();
            let mut out = Vec::new();
            for (k, event) in events.iter().enumerate() {
                let (sub, momenta) = locate(&pdgs, event)
                    .unwrap_or_else(|| panic!("{key}: banked event {k} matches no subprocess"));
                let xwgtup = event["xwgtup"].as_f64().unwrap();
                let alpha_s = event["aqcdup"].as_f64().unwrap();
                rw.ratios(sub, &momenta, Some(alpha_s), &mut out);
                for (h, id) in ids.iter().enumerate() {
                    let mg = event["madgraph"][id].as_f64().unwrap();
                    let ours = xwgtup * out[h];
                    let rel = (ours / mg - 1.0).abs();
                    let s = &mut stats[h];
                    s.events += 1;
                    s.largest = s.largest.max(rel);
                    s.beyond += usize::from(!(rel <= PRINTED));
                }
            }
            for (id, s) in ids.iter().zip(&stats) {
                assert_eq!(s.events, events.len());
                println!(
                    "{key} [{name}] {id:>8}: {} events, largest |w/w_MG - 1| {:.2e}, {} beyond {PRINTED:.0e}",
                    s.events, s.largest, s.beyond
                );
            }
        }
    }
}
