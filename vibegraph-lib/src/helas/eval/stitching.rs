//! Decay-chain stitching against the undecayed final state.
//!
//! A stitched decay-chain set must be exactly the diagrams of the undecayed final state
//! in which every chain resonance is an s-channel line whose final-state side is its
//! stated products, recursively ([`match_resonances`]). The full final state is
//! enumerated at the stitched diagrams' own `WEIGHTED` order: a diagram holding every
//! resonance has the core's order plus each decay's, each at least its own lowest, so
//! nothing below that bound is missed and nothing above it can belong.
//!
//! Two oracles run on every case:
//!
//! - **Container equality**: the two sets of [`Diagram::canonical`] forms are equal, the
//!   filtered diagrams' matched resonances flagged forced on shell. It sees every graph
//!   field, the sign and the symmetry factor, and the on-shell flag on exactly the chain
//!   resonances; it is blind to a sign or orientation convention that is the wrong
//!   function of the graph, which both sides would share.
//! - **Amplitudes**: each stitched diagram and its filtered counterpart, compiled alone,
//!   give the same per-helicity, per-flow amplitudes at a fixed point. The two differ
//!   only in internal numbering (the stitched one numbers the core first, then each
//!   decay), so this sees any convention `helas` still reads off the numbering rather
//!   than the graph; it cannot see what the graph determines for both.

use std::collections::{HashMap, HashSet};

use super::renumbering::{generate, point, single_diagram_amplitudes};
use crate::diagrams::check::check_supported;
use crate::diagrams::diagram::{CanonicalDiagram, Diagram, OnShell};
use crate::diagrams::parse::parse_proc_card_ast;
use crate::diagrams::schannel::{match_resonances, Resonance};
use crate::diagrams::{DiagramSet, Unsupported};
use crate::ufo::sm::{sm_model, SMRestrict};
use crate::ufo::{EvaluatedModel, UFOModel};

/// Relative tolerance on a stitched diagram's amplitudes against its filtered
/// counterpart: the two are one graph under two numberings, so only the rounding of a
/// re-associated momentum sum separates them (as in the renumbering test).
const AMP_REL_TOL: f64 = 1e-10;

fn r(particle: i64, daughters: &[&str], decays: Vec<Resonance>) -> Resonance {
    Resonance {
        particle,
        daughters: daughters.iter().map(|s| s.to_string()).collect(),
        decays,
    }
}

/// A decay-chain card and, per stitched final state, the resonances it describes.
struct Case {
    card: &'static str,
    chain: fn(&[String]) -> Vec<Resonance>,
}

const CASES: &[Case] = &[
    // Identical particles across two identical decays.
    Case {
        card: "e+ e- > z z, z > e+ e-",
        chain: |_| vec![r(23, &["e+", "e-"], vec![]), r(23, &["e+", "e-"], vec![])],
    },
    Case {
        card: "e+ e- > z z, z > e+ e-, z > mu+ mu-",
        chain: |_| vec![r(23, &["e+", "e-"], vec![]), r(23, &["mu+", "mu-"], vec![])],
    },
    Case {
        card: "e+ e- > t t~, t > w+ b, t~ > w- b~",
        chain: |_| vec![r(6, &["W+", "b"], vec![]), r(-6, &["W-", "b~"], vec![])],
    },
    Case {
        card: "e+ e- > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~",
        chain: |_| {
            vec![
                r(6, &["b"], vec![r(24, &["e+", "ve"], vec![])]),
                r(-6, &["W-", "b~"], vec![]),
            ]
        },
    },
    // Colour through a coloured decay, with a gluon in the core.
    Case {
        card: "u u~ > t t~ g, t > w+ b",
        chain: |_| vec![r(6, &["W+", "b"], vec![])],
    },
    // An initial-state fermion line that runs on through the decay: the stitched line
    // carries one more propagator than the core's, which moves its line sign, so the sign
    // is not the product of the parts' signs.
    Case {
        card: "g b > w- t, t > w+ b",
        chain: |_| vec![r(6, &["W+", "b"], vec![])],
    },
    // Hadronic labels: every core subprocess, each lepton flavour.
    Case {
        card: "p p > z j, z > l+ l-",
        chain: |out| {
            let l = if out.iter().any(|p| p == "e+") {
                "e"
            } else {
                "mu"
            };
            vec![r(23, &[&format!("{l}+"), &format!("{l}-")], vec![])]
        },
    },
];

/// The hierarchy-weighted coupling order of a diagram.
fn weighted(d: &Diagram, model: &UFOModel) -> u32 {
    d.vertices
        .iter()
        .map(|v| {
            let def = model.vertex_def(v.interaction);
            let coupling = def
                .couplings
                .values()
                .next()
                .expect("a vertex has a coupling");
            model
                .coupling_def(*coupling)
                .orders
                .iter()
                .map(|(order, &n)| model.order_hierarchy[order] * n as u32)
                .sum::<u32>()
        })
        .sum()
}

/// The undecayed final state's diagrams that hold `chain`, the resonances flagged forced.
fn filtered(set: &DiagramSet, w: u32, chain: &[Resonance], model: &UFOModel) -> Vec<Diagram> {
    let process = format!(
        "{} > {} WEIGHTED<={w}",
        set.particles_in.join(" "),
        set.particles_out.join(" ")
    );
    let full = generate(&process, model);
    let full: Vec<&DiagramSet> = full.iter().filter(|s| !s.diagrams.is_empty()).collect();
    assert_eq!(full.len(), 1, "{process}: one subprocess");
    assert_eq!(
        full[0].particles_out, set.particles_out,
        "{process}: leg order"
    );
    let names: Vec<String> = set
        .particles_in
        .iter()
        .chain(&set.particles_out)
        .cloned()
        .collect();
    full[0]
        .diagrams
        .iter()
        .filter_map(|d| {
            let props = match_resonances(d, model, &names, chain)?;
            let mut d = d.clone();
            for p in props {
                d.props[p.0].onshell = OnShell::Forced;
            }
            Some(d)
        })
        .collect()
}

/// Container equality and per-diagram amplitudes, stitched against filtered, on every
/// case; returns the failures.
fn compare(case: &Case, model: &UFOModel, evaluated: &EvaluatedModel) -> Vec<String> {
    let mut failures = Vec::new();
    let stitched = generate(case.card, model);
    let (mut n_sets, mut n_diagrams, mut n_amps, mut worst_amp) = (0, 0, 0, 0.0f64);
    for set in stitched.iter().filter(|s| !s.diagrams.is_empty()) {
        n_sets += 1;
        let sub = format!(
            "{} | {} > {}",
            case.card,
            set.particles_in.join(" "),
            set.particles_out.join(" ")
        );
        let orders: HashSet<u32> = set.diagrams.iter().map(|d| weighted(d, model)).collect();
        assert_eq!(orders.len(), 1, "{sub}: one WEIGHTED order");
        let w = *orders.iter().next().unwrap();
        let chain = (case.chain)(&set.particles_out);
        let reference = filtered(set, w, &chain, model);

        let ours: Vec<CanonicalDiagram> = set.diagrams.iter().map(|d| d.canonical(model)).collect();
        let theirs: HashMap<CanonicalDiagram, &Diagram> =
            reference.iter().map(|d| (d.canonical(model), d)).collect();
        let ours_set: HashSet<&CanonicalDiagram> = ours.iter().collect();
        n_diagrams += ours.len();
        if ours_set.len() != ours.len() {
            failures.push(format!(
                "{sub}: {} stitched diagrams repeat",
                ours.len() - ours_set.len()
            ));
        }
        if theirs.len() != reference.len() {
            failures.push(format!("{sub}: filtered diagrams repeat"));
        }
        let missing = ours.iter().filter(|c| !theirs.contains_key(*c)).count();
        let extra = theirs.keys().filter(|c| !ours_set.contains(c)).count();
        if missing > 0 || extra > 0 {
            failures.push(format!(
                "{sub}: {} stitched, {} filtered; {missing} stitched not among the filtered, \
                 {extra} filtered not stitched",
                ours.len(),
                reference.len()
            ));
            continue;
        }
        // Every forced line is a decay the provenance records, and every recorded decay
        // is a forced line.
        for d in &set.diagrams {
            let forced: HashSet<usize> = (0..d.props.len())
                .filter(|&p| d.props[p].onshell == OnShell::Forced)
                .collect();
            let recorded: HashSet<usize> = d.provenance.decays.iter().map(|o| o.prop.0).collect();
            if forced != recorded || forced.len() != chain_size(&chain) {
                failures.push(format!("{sub}: forced {forced:?}, recorded {recorded:?}"));
            }
        }

        let momenta = point(set, evaluated);
        for (d, c) in set.diagrams.iter().zip(&ours) {
            let a = single_diagram_amplitudes(set, d, model, evaluated, &momenta);
            let b = single_diagram_amplitudes(set, theirs[c], model, evaluated, &momenta);
            n_amps += 1;
            let scale = a.iter().map(|z| z.norm()).fold(0.0, f64::max);
            let diff = a
                .iter()
                .zip(&b)
                .map(|(x, y)| (x - y).norm())
                .fold(0.0, f64::max);
            if scale == 0.0 || a.len() != b.len() || diff > AMP_REL_TOL * scale {
                failures.push(format!(
                    "{sub}: amplitudes differ by {diff:.3e} at scale {scale:.3e} ({} vs {} values)",
                    a.len(),
                    b.len()
                ));
            }
            worst_amp = worst_amp.max(diff / scale);
        }
    }
    println!(
        "{}: {n_sets} sets, {n_diagrams} diagrams equal as containers, {n_amps} amplitude \
         comparisons, worst relative {worst_amp:.1e}",
        case.card
    );
    if n_sets == 0 {
        failures.push(format!("{}: nothing stitched", case.card));
    }
    failures
}

fn chain_size(chain: &[Resonance]) -> usize {
    chain.iter().map(|r| 1 + chain_size(&r.decays)).sum()
}

/// Every stitched decay-chain set is the undecayed final state's diagrams that hold the
/// chain, as containers and diagram by diagram in amplitude.
#[test]
fn stitched_chains_are_the_filtered_final_state() {
    let model = sm_model(SMRestrict::Default);
    let evaluated = EvaluatedModel::from_model(model.clone());
    let failures: Vec<String> = CASES
        .iter()
        .flat_map(|case| compare(case, &model, &evaluated))
        .collect();
    for f in failures.iter().take(40) {
        println!("  {f}");
    }
    assert!(failures.is_empty(), "{} stitching failures", failures.len());
}

/// Enumeration accepts a decay chain; integration still refuses it, with the one
/// variant that says what is missing.
#[test]
fn decay_chains_enumerate_but_do_not_integrate() {
    let ast = parse_proc_card_ast("generate e+ e- > t t~, t > w+ b, t~ > w- b~").unwrap();
    let refused = check_supported(&ast).unwrap_err().0;
    assert!(matches!(refused[..], [Unsupported::DecayChain { .. }]));
    assert!(crate::diagrams::check_enumerable(&ast).is_ok());
}

/// Identical particles between a decay's products and the core's own final state, where
/// the core also holds the resonance as an internal line: `e+ e- > z e+ e-` has diagrams
/// with an s-channel `Z → e+ e-` of its own, so after permuting the electrons one graph
/// is stitched twice, once with each `Z` forced on shell. As a graph it is one diagram of
/// the undecayed final state; which line is the decay's is not defined, and the stitching
/// refuses rather than choose.
#[test]
fn an_ambiguous_forced_line_is_refused() {
    use crate::diagrams::{check_enumerable, generate_decay_chains, DiagramError};
    let model = sm_model(SMRestrict::Default);
    let ast = parse_proc_card_ast("generate e+ e- > z e+ e-, z > e+ e-").unwrap();
    let card = check_enumerable(&ast).unwrap();
    match generate_decay_chains(&card, &model) {
        Err(DiagramError::DecayChain { reason, .. }) => {
            assert!(reason.contains("ambiguous"), "{reason}")
        }
        Err(e) => panic!("unexpected error: {e}"),
        Ok(_) => panic!("an ambiguous forced line must be refused"),
    }
}
