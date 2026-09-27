//! The proc-card grammar against MadGraph's own parser.
//!
//! `validation/madgraph/proc_grammar.json` banks, for a corpus of proc cards,
//! what MadGraph's `MasterCmd` holds after reading each one — the
//! `ProcessDefinition` of every process, field by field, or the error MadGraph
//! raised (`validation/madgraph/dump_proc_grammar.py`). This test reads the same
//! cards, resolves them against the same SM, and compares:
//!
//! - where MadGraph read the card, the parse and the resolution here must both
//!   succeed and agree on every field (the check may still refuse the card: a
//!   feature being parsed correctly and being supported are separate claims);
//! - where MadGraph refused it, this side must refuse it too, at the parse, the
//!   resolution or the check.
//!
//! What the comparison cannot see: the *order* of a multiparticle's members
//! (MadGraph sorts them by spin, colour and mass in `optimize_order`; they are
//! compared as sets), and anything downstream of the process definition —
//! which diagrams a restriction keeps is the diagram census's business.

use std::path::Path;

use serde_json::{json, Map, Value};
use vibegraph::diagrams::check_supported;
use vibegraph::diagrams::parse::parse_proc_card_ast;
use vibegraph::diagrams::resolve::{resolve_card, ResolvedProcess};
use vibegraph::ufo::sm::{sm_model, SMRestrict};

fn reference() -> Value {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/proc_grammar.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    serde_json::from_str(&text).expect("proc_grammar.json is JSON")
}

fn sorted(v: &Value) -> Value {
    let mut items: Vec<i64> = v
        .as_array()
        .expect("an id list")
        .iter()
        .map(|x| x.as_i64().expect("an integer id"))
        .collect();
    items.sort_unstable();
    json!(items)
}

/// A process definition with every set-valued list in a canonical order.
fn normalize(p: &Value) -> Value {
    let mut out: Map<String, Value> = p.as_object().expect("a process").clone();
    let legs: Vec<Value> = p["legs"]
        .as_array()
        .expect("legs")
        .iter()
        .map(|l| {
            let mut l = l.as_object().expect("a leg").clone();
            let ids = sorted(&l["ids"]);
            l.insert("ids".into(), ids);
            Value::Object(l)
        })
        .collect();
    out.insert("legs".into(), json!(legs));
    for key in [
        "forbidden_particles",
        "forbidden_s_channels",
        "forbidden_onsh_s_channels",
    ] {
        out.insert(key.into(), sorted(&p[key]));
    }
    let required: Vec<Value> = p["required_s_channels"]
        .as_array()
        .expect("required s-channels")
        .iter()
        .map(sorted)
        .collect();
    out.insert("required_s_channels".into(), json!(required));
    let decays: Vec<Value> = p["decay_chains"]
        .as_array()
        .expect("decay chains")
        .iter()
        .map(normalize)
        .collect();
    out.insert("decay_chains".into(), json!(decays));
    Value::Object(out)
}

/// A resolved process in the reference's JSON shape.
fn to_json(p: &ResolvedProcess) -> Value {
    json!({
        "id": p.id,
        "legs": p.legs.iter().map(|l| json!({
            "ids": l.ids,
            "state": l.state,
            "polarization": l.polarization,
            "tagged": l.tagged,
        })).collect::<Vec<_>>(),
        "required_s_channels": p.required_s_channels,
        "forbidden_particles": p.forbidden_particles,
        "forbidden_s_channels": p.forbidden_s_channels,
        "forbidden_onsh_s_channels": p.forbidden_onsh_s_channels,
        "orders": p.orders,
        "squared_orders": p.squared_orders,
        "sqorders_types": p.sqorders_types,
        "constrained_orders": p.constrained_orders.iter()
            .map(|(k, (v, t))| (k.clone(), json!([v, t])))
            .collect::<Map<String, Value>>(),
        "overall_orders": p.overall_orders,
        "perturbation_couplings": p.perturbation_couplings,
        "NLO_mode": p.nlo_mode,
        "has_born": p.has_born,
        "decay_chains": p.decay_chains.iter().map(to_json).collect::<Vec<_>>(),
    })
}

#[test]
fn proc_card_grammar_matches_madgraph() {
    let reference = reference();
    let model = sm_model(SMRestrict::Default);
    let cases = reference["cases"].as_array().expect("cases");
    assert!(cases.len() > 100, "the corpus is the oracle's reach");

    let mut failures = Vec::new();
    let (mut agreed, mut refused) = (0, 0);
    for case in cases {
        let name = case["name"].as_str().expect("a name");
        let card = case["card"].as_str().expect("a card");
        let ours = parse_proc_card_ast(card)
            .map_err(|e| format!("parse: {e}"))
            .and_then(|ast| {
                let resolved = resolve_card(&ast, &model).map_err(|e| format!("resolve: {e}"))?;
                Ok((ast, resolved))
            });

        if let Some(theirs) = case.get("error") {
            let refusal = match &ours {
                Err(e) => Some(e.clone()),
                Ok((ast, _)) => check_supported(ast).err().map(|e| format!("check: {e}")),
            };
            match refusal {
                Some(_) => refused += 1,
                None => failures.push(format!(
                    "[{name}] MadGraph refuses the card ({theirs}) and this side accepts it"
                )),
            }
            continue;
        }

        let (_, resolved) = match ours {
            Ok(ok) => ok,
            Err(e) => {
                failures.push(format!(
                    "[{name}] MadGraph reads the card, this side fails: {e}"
                ));
                continue;
            }
        };
        let theirs: Vec<Value> = case["processes"]
            .as_array()
            .expect("processes")
            .iter()
            .map(normalize)
            .collect();
        let ours: Vec<Value> = resolved.iter().map(|p| normalize(&to_json(p))).collect();
        if theirs.len() != ours.len() {
            failures.push(format!(
                "[{name}] MadGraph holds {} processes, this side {}",
                theirs.len(),
                ours.len()
            ));
            continue;
        }
        for (i, (t, o)) in theirs.iter().zip(&ours).enumerate() {
            for key in t.as_object().expect("a process").keys() {
                if t[key] != o[key] {
                    failures.push(format!(
                        "[{name}] process {i}, field {key}: MadGraph {} against {}",
                        t[key], o[key]
                    ));
                }
            }
        }
        agreed += 1;
    }
    println!("{agreed} cards agree field by field, {refused} refused by both sides");
    assert!(
        failures.is_empty(),
        "{} disagreements with MadGraph's parser:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
