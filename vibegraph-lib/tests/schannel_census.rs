//! The diagram census against MadGraph's own generation, for the cards whose
//! diagrams depend on s-channel restrictions, explicit `WEIGHTED` bounds, or
//! the five-flavour rewrite of `p` and `j` on model import.
//!
//! `validation/madgraph/schannel_census.json` banks, per card, what MadGraph's
//! `MasterCmd` generates (`validation/madgraph/dump_schannel_census.py`):
//! either the error it raised or every subprocess with its diagram count and,
//! per diagram, the multiset of its s-channel propagators' ids as MadGraph
//! orients them (`Vertex.get_s_channel_id`), plus the members of `p` and `j`.
//! This test enumerates the same cards against the same models and compares:
//!
//! - where MadGraph generated, the same subprocesses, each with the same
//!   number of diagrams and the same multiset of per-diagram s-channel
//!   multisets (which pins the orientation: a `W` read the wrong way round is a
//!   different id, and `$$ t` against `$$ t~` keeps different diagrams);
//! - where MadGraph refused, enumeration refuses too;
//! - the members of `p` and `j` after the card, as sets.
//!
//! Cards with one initial particle are banked for when 1→n processes are
//! supported; until then this side must refuse them as a decay process.
//!
//! What the comparison cannot see: which diagram is which beyond its
//! s-channel content (two diagrams with the same s-channels are
//! interchangeable here), and anything past the diagram list.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::Value;
use vibegraph::diagrams::check::{check_supported, Unsupported};
use vibegraph::diagrams::parse::parse_proc_card_ast;
use vibegraph::diagrams::resolve::model_aliases;
use vibegraph::diagrams::schannel::s_channel_ids;
use vibegraph::diagrams::{generate_from_proc_card, DiagramError};
use vibegraph::ufo::sm::{sm_model, SMRestrict};
use vibegraph::ufo::UFOModel;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn reference() -> Value {
    let path = repo().join("validation/madgraph/schannel_census.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    serde_json::from_str(&text).expect("schannel_census.json is JSON")
}

/// The model a card imports: an SM restriction by suffix, or the committed
/// SMEFTsim with a restrict card.
fn model_for(card: &str) -> Arc<UFOModel> {
    let import = card
        .lines()
        .rev()
        .find_map(|l| l.trim().strip_prefix("import model "))
        .map(str::trim);
    match import {
        None | Some("sm") => sm_model(SMRestrict::Default),
        Some(name) if name.starts_with("sm-") => {
            sm_model(SMRestrict::from_suffix(Some(&name[3..])).expect("a known SM restriction"))
        }
        Some(name) => {
            let restrict = name
                .strip_prefix("@SMEFTSIM@-")
                .unwrap_or_else(|| panic!("unexpected model '{name}'"));
            let dir = repo().join("validation/ufo/SMEFTsim_topU3l_MwScheme_UFO");
            UFOModel::load(&dir, Some(&dir.join(format!("restrict_{restrict}.dat"))))
                .unwrap_or_else(|e| panic!("load SMEFTsim {restrict}: {e}"))
        }
    }
}

/// One subprocess: (sorted initial ids, sorted final ids) → (diagrams, sorted
/// per-diagram s-channel id lists).
type Census = BTreeMap<(Vec<i64>, Vec<i64>), (usize, Vec<Vec<i64>>)>;

fn ids(values: &Value) -> Vec<i64> {
    values
        .as_array()
        .expect("an id list")
        .iter()
        .map(|v| v.as_i64().expect("an id"))
        .collect()
}

fn banked(case: &Value) -> Census {
    case["subprocesses"]
        .as_array()
        .expect("subprocesses")
        .iter()
        .map(|s| {
            let s_channels = s["s_channels"]
                .as_array()
                .expect("s_channels")
                .iter()
                .map(ids)
                .collect();
            (
                (ids(&s["initial"]), ids(&s["final"])),
                (s["diagrams"].as_u64().unwrap() as usize, s_channels),
            )
        })
        .collect()
}

enum Outcome {
    Census(Census, BTreeMap<String, Vec<i64>>),
    Refused(String),
    DecayPending,
}

fn enumerate(card: &str) -> Outcome {
    let ast = match parse_proc_card_ast(card) {
        Ok(ast) => ast,
        Err(e) => return Outcome::Refused(e.to_string()),
    };
    let supported = match check_supported(&ast) {
        Ok(c) => c,
        Err(e)
            if e.0
                .iter()
                .any(|u| matches!(u, Unsupported::DecayProcess { .. })) =>
        {
            return Outcome::DecayPending
        }
        Err(e) => return Outcome::Refused(e.to_string()),
    };
    let model = model_for(card);
    let pdg = |name: &str| model.particles[name].pdg_code;
    let sets = match generate_from_proc_card(&supported, &model) {
        Ok(sets) => sets,
        Err(e @ (DiagramError::NoDiagrams { .. } | DiagramError::Resolve(_))) => {
            return Outcome::Refused(e.to_string())
        }
        Err(e) => panic!("unexpected enumeration error: {e}"),
    };
    let mut census = Census::new();
    for set in sets.iter().filter(|s| !s.diagrams.is_empty()) {
        let mut initial: Vec<i64> = set.particles_in.iter().map(|n| pdg(n)).collect();
        let mut final_state: Vec<i64> = set.particles_out.iter().map(|n| pdg(n)).collect();
        initial.sort_unstable();
        final_state.sort_unstable();
        let mut s_channels: Vec<Vec<i64>> = set
            .diagrams
            .iter()
            .map(|d| s_channel_ids(d, &model))
            .collect();
        s_channels.sort();
        let previous = census.insert((initial, final_state), (set.diagrams.len(), s_channels));
        assert!(previous.is_none(), "a subprocess enumerated twice");
    }
    let processes = ast.processes();
    let aliases = model_aliases(&processes.last().expect("a process").aliases, &model);
    let labels = ["p", "j"]
        .into_iter()
        .map(|label| {
            let mut members: Vec<i64> = aliases
                .plain_members(label)
                .expect("p and j are plain labels")
                .iter()
                .map(|m| {
                    vibegraph::diagrams::resolve::particle_by_name(&model, m)
                        .expect("a label member of the model")
                        .pdg_code
                })
                .collect();
            members.sort_unstable();
            (label.to_owned(), members)
        })
        .collect();
    Outcome::Census(census, labels)
}

fn describe(census: &Census) -> String {
    census
        .iter()
        .map(|((i, f), (n, _))| format!("{i:?} > {f:?}: {n}"))
        .collect::<Vec<_>>()
        .join("; ")
}

#[test]
fn schannel_census_matches_madgraph() {
    let doc = reference();
    let cases = doc["cases"].as_array().expect("cases");
    let mut failures = Vec::new();
    let (mut generated, mut refused, mut pending) = (0, 0, 0);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let card = case["card"].as_str().unwrap();
        let outcome = enumerate(card);
        match (case.get("error"), outcome) {
            (_, Outcome::DecayPending) => {
                if !name.starts_with("decay_") {
                    failures.push(format!("{name}: refused as a decay process"));
                }
                pending += 1;
            }
            (Some(_), Outcome::Refused(_)) => refused += 1,
            (Some(err), Outcome::Census(ours, _)) => failures.push(format!(
                "{name}: MadGraph refuses ({err}), here {}",
                describe(&ours)
            )),
            (None, Outcome::Refused(e)) => {
                failures.push(format!("{name}: MadGraph generates, here refused: {e}"))
            }
            (None, Outcome::Census(ours, labels)) => {
                generated += 1;
                let theirs = banked(case);
                if ours != theirs {
                    let counts_only = |c: &Census| describe(c);
                    let detail = if counts_only(&ours) == counts_only(&theirs) {
                        let diff: Vec<String> = ours
                            .iter()
                            .filter(|(k, v)| theirs.get(*k) != Some(*v))
                            .map(|(k, v)| {
                                format!("{k:?}: here {:?}, MadGraph {:?}", v.1, theirs[k].1)
                            })
                            .collect();
                        format!("s-channel content differs: {}", diff.join("; "))
                    } else {
                        format!(
                            "subprocesses differ:\n    here     {}\n    MadGraph {}",
                            describe(&ours),
                            describe(&theirs)
                        )
                    };
                    failures.push(format!("{name}: {detail}"));
                }
                for label in ["p", "j"] {
                    let mut mg = ids(&case["multiparticles"][label]);
                    mg.sort_unstable();
                    if labels[label] != mg {
                        failures.push(format!(
                            "{name}: '{label}' is {:?} here, {mg:?} in MadGraph",
                            labels[label]
                        ));
                    }
                }
            }
        }
    }
    eprintln!(
        "{} cards: {generated} generated and matched, {refused} refused by both, {pending} \
         decay cards pending 1→n support",
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
