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
use crate::diagrams::DiagramSet;
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
    // Yang-Mills vertices: three-gluon, four-gluon and electroweak quartic vertices.
    Case {
        card: "g g > t t~ g, t > w+ b",
        chain: |_| vec![r(6, &["W+", "b"], vec![])],
    },
    Case {
        card: "u u~ > z g g g, z > e+ e-",
        chain: |_| vec![r(23, &["e+", "e-"], vec![])],
    },
    Case {
        card: "e+ e- > w+ w- z, z > mu+ mu-",
        chain: |_| vec![r(23, &["mu+", "mu-"], vec![])],
    },
    // Two identical decays on the interchangeable Z slots of the h Z Z vertex, nested
    // under a core decay: permuting the electrons between the decays reaches each graph a
    // second time with the two Z lines exchanged between those slots.
    Case {
        card: "e+ e- > z h, (h > z z, z > e+ e-, z > e+ e-)",
        chain: |_| {
            vec![r(
                25,
                &[],
                vec![r(23, &["e+", "e-"], vec![]), r(23, &["e+", "e-"], vec![])],
            )]
        },
    },
    // Identical products from decays on the Z and h slots of one vertex, which are not
    // interchangeable: every permutation is a diagram of its own.
    Case {
        card: "e+ e- > z h, z > b b~, h > b b~",
        chain: |_| vec![r(23, &["b", "b~"], vec![]), r(25, &["b", "b~"], vec![])],
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
fn weighted(d: &Diagram, model: &UFOModel) -> i64 {
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
                .map(|(order, &n)| model.order_hierarchy[order] as i64 * n)
                .sum::<i64>()
        })
        .sum()
}

/// The undecayed final state's diagrams that hold `chain`, the resonances flagged forced.
fn filtered(set: &DiagramSet, w: i64, chain: &[Resonance], model: &UFOModel) -> Vec<Diagram> {
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
    let (mut n_contact, mut n_slot_order) = (0, 0);
    for set in stitched.iter().filter(|s| !s.diagrams.is_empty()) {
        n_sets += 1;
        let sub = format!(
            "{} | {} > {}",
            case.card,
            set.particles_in.join(" "),
            set.particles_out.join(" ")
        );
        let orders: HashSet<i64> = set.diagrams.iter().map(|d| weighted(d, model)).collect();
        assert_eq!(orders.len(), 1, "{sub}: one WEIGHTED order");
        let w = *orders.iter().next().unwrap();
        let chain = (case.chain)(&set.particles_out);
        let reference = filtered(set, w, &chain, model);

        let ours: Vec<CanonicalDiagram> = set.diagrams.iter().map(|d| d.canonical(model)).collect();
        let theirs: HashMap<CanonicalDiagram, &Diagram> =
            reference.iter().map(|d| (d.canonical(model), d)).collect();
        let ours_set: HashSet<&CanonicalDiagram> = ours.iter().collect();
        n_diagrams += ours.len();
        n_contact += set
            .diagrams
            .iter()
            .filter(|d| d.vertices.iter().any(|v| v.rays.len() == 4))
            .count();
        if ours_set.len() != ours.len() {
            failures.push(format!(
                "{sub}: {} stitched diagrams repeat",
                ours.len() - ours_set.len()
            ));
        }
        if theirs.len() != reference.len() {
            failures.push(format!("{sub}: filtered diagrams repeat"));
        }
        // Container equality is blind to which of two identical-particle slots of a
        // vertex a line binds to (a false inequality): pair what is left by a
        // slot-order-free description of the graph, and let the amplitudes confirm.
        let mut partner: HashMap<&CanonicalDiagram, &Diagram> = HashMap::new();
        let mut unmatched_theirs: Vec<(&CanonicalDiagram, &Diagram)> = theirs
            .iter()
            .filter(|(c, _)| !ours_set.contains(c))
            .map(|(c, d)| (c, *d))
            .collect();
        let mut missing = 0;
        for c in &ours {
            if let Some(d) = theirs.get(c) {
                partner.insert(c, d);
                continue;
            }
            let key = slot_free(c.diagram());
            match unmatched_theirs
                .iter()
                .position(|(t, _)| slot_free(t.diagram()) == key)
            {
                Some(i) => {
                    partner.insert(c, unmatched_theirs.swap_remove(i).1);
                    n_slot_order += 1;
                }
                None => missing += 1,
            }
        }
        let extra = unmatched_theirs.len();
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
            let b = single_diagram_amplitudes(set, partner[c], model, evaluated, &momenta);
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
        "{}: {n_sets} sets, {n_diagrams} diagrams ({n_contact} with a four-point vertex), \
         {} equal as containers and {n_slot_order} up to identical-slot order, {n_amps} \
         amplitude comparisons, worst relative {worst_amp:.1e}",
        case.card,
        n_diagrams - n_slot_order
    );
    if n_sets == 0 {
        failures.push(format!("{}: nothing stitched", case.card));
    }
    failures
}

/// A vertex's interaction and, per ray, the external legs beyond it and whether it is a
/// forced line.
type SlotFreeVertex = (usize, Vec<(Vec<usize>, bool)>);
/// Sign, symmetry factor and vertices, sorted.
type SlotFree = (i8, usize, Vec<SlotFreeVertex>);

/// A description of a diagram that ignores which slot of a vertex each line binds to: per
/// vertex its interaction and, per ray, the external legs beyond it and whether the ray
/// is a forced line; with the sign and symmetry factor.
fn slot_free(d: &Diagram) -> SlotFree {
    use crate::diagrams::diagram::{Ray, VtxIdx};
    let beyond = |from: VtxIdx, ray: Ray| -> Vec<usize> {
        let (start, via) = match ray {
            Ray::Leg(l) => return vec![l.0],
            Ray::Prop { prop, end } => (d.props[prop.0].endpoints[1 - end].0, prop),
        };
        let mut legs = Vec::new();
        let mut stack = vec![(start, via)];
        let mut seen = vec![false; d.vertices.len()];
        seen[from.0] = true;
        while let Some((v, came)) = stack.pop() {
            if std::mem::replace(&mut seen[v.0], true) {
                continue;
            }
            for &r in &d.vertices[v.0].rays {
                match r {
                    Ray::Leg(l) => legs.push(l.0),
                    Ray::Prop { prop, end } if prop != came => {
                        stack.push((d.props[prop.0].endpoints[1 - end].0, prop))
                    }
                    Ray::Prop { .. } => {}
                }
            }
        }
        legs.sort_unstable();
        legs
    };
    let mut vertices: Vec<SlotFreeVertex> = d
        .vertices
        .iter()
        .enumerate()
        .map(|(vi, v)| {
            let mut rays: Vec<(Vec<usize>, bool)> = v
                .rays
                .iter()
                .map(|&r| {
                    let forced = matches!(r, Ray::Prop { prop, .. }
                        if d.props[prop.0].onshell == OnShell::Forced);
                    (beyond(VtxIdx(vi), r), forced)
                })
                .collect();
            rays.sort();
            (v.interaction.0, rays)
        })
        .collect();
    vertices.sort();
    (d.sign, d.symmetry_factor, vertices)
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

/// A decay chain passes the one check: its diagrams, windows and integration are all
/// supported.
#[test]
fn decay_chains_pass_the_check() {
    let ast = parse_proc_card_ast("generate e+ e- > t t~, t > w+ b, t~ > w- b~").unwrap();
    let card = check_supported(&ast).unwrap();
    assert_eq!(card.processes[0].decays.len(), 2);
}

/// Identical particles between a decay's products and the core's own final state, where
/// the core also holds the resonance as an internal line: `e+ e- > z e+ e-` has diagrams
/// with an s-channel `Z → e+ e-` of its own, so after permuting the electrons one graph
/// is stitched twice, once with each `Z` forced on shell. As a graph it is one diagram of
/// the undecayed final state; which line is the decay's is not defined, and the stitching
/// refuses rather than choose.
#[test]
fn an_ambiguous_forced_line_is_refused() {
    use crate::diagrams::{generate_from_proc_card, DiagramError};
    let model = sm_model(SMRestrict::Default);
    let ast = parse_proc_card_ast("generate e+ e- > z e+ e-, z > e+ e-").unwrap();
    let card = check_supported(&ast).unwrap();
    match generate_from_proc_card(&card, &model) {
        Err(DiagramError::DecayChain { reason, .. }) => {
            assert!(reason.contains("ambiguous"), "{reason}")
        }
        Err(e) => panic!("unexpected error: {e}"),
        Ok(_) => panic!("an ambiguous forced line must be refused"),
    }
}

/// Every stitched diagram dropped as a second way to reach a graph already kept has the
/// kept diagram's amplitudes, helicity by helicity and flow by flow, at a fixed point: the
/// two are one Feynman diagram. A key that merged two different diagrams fails here
/// rather than silently losing one.
///
/// The cards reach both kinds of merge: graphs equal under `Diagram::canonical`
/// (`e+ e- > z z, z > e+ e-`, whose u-channel core diagram is its t-channel one with the
/// Z decays swapped) and graphs equal only once lines on a vertex's identical-boson slots
/// are ordered (`h > z z` with two identical `Z` decays).
///
/// Blind to a merge that drops a diagram whose amplitude happens to equal the kept one's
/// at the point, which a generic point makes measure-zero.
#[test]
fn merged_stitched_diagrams_have_equal_amplitudes() {
    use crate::diagrams::generate_chain_with_merges;
    let model = sm_model(SMRestrict::Default);
    let evaluated = EvaluatedModel::from_model(model.clone());
    let mut failures = Vec::new();
    for (card, min_slot_merges) in [
        ("e+ e- > z z, z > e+ e-", 0),
        ("e+ e- > z h, (h > z z, z > e+ e-, z > e+ e-)", 1),
        ("e+ e- > z h, z > b b~, h > b b~", 0),
    ] {
        let ast = parse_proc_card_ast(&format!("generate {card}")).unwrap();
        let process = &check_supported(&ast).unwrap().processes[0];
        let (_, merges) = generate_chain_with_merges(process, &model).unwrap();
        let mut slot_merges = 0;
        let mut worst = 0.0f64;
        for m in &merges {
            if m.kept.canonical(&model) != m.merged.canonical(&model) {
                slot_merges += 1;
            }
            let set = DiagramSet {
                particles_in: m.particles_in.clone(),
                particles_out: m.particles_out.clone(),
                polarizations: m.polarizations.clone(),
                diagrams: vec![m.kept.clone()],
                mirrored: false,
                squared_orders: None,
            };
            let momenta = point(&set, &evaluated);
            let a = single_diagram_amplitudes(&set, &m.kept, &model, &evaluated, &momenta);
            let b = single_diagram_amplitudes(&set, &m.merged, &model, &evaluated, &momenta);
            let scale = a.iter().map(|z| z.norm()).fold(0.0, f64::max);
            let diff = a
                .iter()
                .zip(&b)
                .map(|(x, y)| (x - y).norm())
                .fold(0.0, f64::max);
            if scale == 0.0 || a.len() != b.len() || diff > AMP_REL_TOL * scale {
                failures.push(format!(
                    "{card} | {} > {}: a merged diagram differs from the kept one by \
                     {diff:.3e} at scale {scale:.3e}",
                    m.particles_in.join(" "),
                    m.particles_out.join(" ")
                ));
            }
            worst = worst.max(diff / scale);
        }
        println!(
            "{card}: {} merges, {slot_merges} only up to identical-boson slot order, worst \
             relative amplitude difference {worst:.1e}",
            merges.len()
        );
        if slot_merges < min_slot_merges {
            failures.push(format!(
                "{card}: {slot_merges} merges up to slot order, expected at least \
                 {min_slot_merges}"
            ));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// Two decays with identical products on slots of different particles (`Z` and `h` of
/// the `Z Z h` vertex) are not one diagram under exchange: all four arrangements of the
/// `b` and `b~` between the decays are kept.
#[test]
fn decays_on_slots_of_different_particles_are_all_kept() {
    let model = sm_model(SMRestrict::Default);
    let sets = generate("e+ e- > z h, z > b b~, h > b b~", &model);
    let sets: Vec<&DiagramSet> = sets.iter().filter(|s| !s.diagrams.is_empty()).collect();
    assert_eq!(sets.len(), 1);
    let unpermuted = generate("e+ e- > z h", &model)
        .iter()
        .map(|s| s.diagrams.len())
        .sum::<usize>();
    assert_eq!(sets[0].diagrams.len(), 4 * unpermuted);
}

/// One graph per Feynman diagram where two identical decays meet at a vertex's
/// interchangeable slots: `h > z z, z > e+ e-, z > e+ e-` has the two pairings of the
/// electrons with the positrons, not each of them twice.
#[test]
fn identical_decays_on_interchangeable_slots_are_stitched_once() {
    let model = sm_model(SMRestrict::Default);
    let sets = generate("e+ e- > z h, (h > z z, z > e+ e-, z > e+ e-)", &model);
    let sets: Vec<&DiagramSet> = sets.iter().filter(|s| !s.diagrams.is_empty()).collect();
    assert_eq!(sets.len(), 1);
    let core = generate("e+ e- > z h", &model)
        .iter()
        .map(|s| s.diagrams.len())
        .sum::<usize>();
    assert_eq!(sets[0].diagrams.len(), 2 * core);
}
