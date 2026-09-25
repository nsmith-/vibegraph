//! Internal-numbering invariance of the compiled amplitude.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use num_complex::Complex;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::compile::AmplitudeEvaluator;
use super::root_diagram::compile_single_diagram;
use super::run::BoundAmplitude;
use crate::diagrams::diagram::{Diagram, VtxIdx};
use crate::diagrams::{
    check_enumerable, generate_decay_chains, generate_from_proc_card, parse_proc_card,
    parse_proc_card_ast, DiagramSet, ParsingOptions,
};
use crate::helas::color::colorize::colorize_process;
use crate::helas::LorentzVector;
use crate::phasespace::rambo_massive;
use crate::ufo::sm::{sm_model, SMRestrict};
use crate::ufo::{EvaluatedModel, UFOModel};

/// Standard-Model processes beyond the amplitude-table rows: the sign classes the
/// table rows reach only partly (several VVV and four-gluon vertices per diagram,
/// identical-fermion lines, three and more fermion lines).
const SM_EXTRAS: &[&str] = &[
    "e+ e- > e+ e-",
    "g g > t t~",
    "u d > e+ e- u d QCD=0",
    "e+ e- > W+ W-",
    "g g > g g",
    "g g > g g g",
    "u u~ > g g g",
    "u u~ > g g",
    "u u~ > d d~",
    "e+ e- > Z Z",
    "u u~ > u u~ g",
    "u d > u d g",
    "u u~ > t t~ g",
    "e+ e- > mu+ mu- ta+ ta-",
    "e+ e- > e+ e- e+ e-",
    "u u~ > u u~ u u~",
    "e+ e- > W+ W- Z",
    "g g > t t~ g",
    // 1 → n decays: the decaying particle is the one incoming leg, every line
    // s-channel.
    "t > b e+ ve a",
    "t > w+ b g",
    "z > e+ e- mu+ mu-",
    "h > e+ e- mu+ mu-",
    // Decay chains: diagrams stitched from separate core and decay enumerations, with
    // identical particles permuted between decays.
    "e+ e- > z z, z > e+ e-",
    "e+ e- > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~",
    "u u~ > t t~ g, t > w+ b",
    "e+ e- > w+ w-, w+ > j j, w- > j j",
    "g g > t t~ g, t > w+ b",
    "u u~ > z g g g, z > e+ e-",
    "e+ e- > w+ w- z, z > mu+ mu-",
];

/// Processes in the manifest rows' own models that the rows themselves do not reach: an
/// emission off a line that leaves a four-fermion contact, which puts the contact at a
/// vertex rooted at one of its fermion legs. `(model directory, restrict card name,
/// process)`.
const MODEL_EXTRAS: &[(&str, &str, &str)] = &[
    (
        "validation/ufo/SMEFTsim_topU3l_MwScheme_UFO",
        "vg_cleQt3",
        "ta+ ta- > t t~ a NP<=1",
    ),
    (
        "validation/ufo/SMEFTsim_topU3l_MwScheme_UFO",
        "vg_c4l",
        "e+ e- > mu+ mu- a NP<=1",
    ),
    (
        "validation/ufo/SMEFTsim_topU3l_MwScheme_UFO",
        "vg_c4q",
        "u u~ > t t~ g NP<=1",
    ),
    (
        "validation/ufo/vibegraph_toy_UFO",
        "tensor",
        "lt~ lt > qt qt~ vt NP<=1 NPGG<=1",
    ),
    (
        "validation/ufo/vibegraph_toy_UFO",
        "dipole",
        "lt~ lt > qt qt~ vt NP<=1",
    ),
];

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// A manifest row's UFO model under a restrict card, looked up the way the row's
/// `.mg5` script imports it (the model's own cards first, then the authored ones), and
/// loaded once.
fn load(
    models: &mut HashMap<(String, Option<String>), Arc<UFOModel>>,
    dir: &str,
    restrict: Option<&str>,
) -> Arc<UFOModel> {
    models
        .entry((dir.to_owned(), restrict.map(str::to_owned)))
        .or_insert_with(|| {
            let dir = repo().join(dir);
            let card = restrict.map(|name| {
                let file = format!("restrict_{name}.dat");
                let shipped = dir.join(&file);
                if shipped.exists() {
                    return shipped;
                }
                let last = dir.file_name().unwrap().to_str().unwrap();
                let family = if last.starts_with("SMEFTsim_") {
                    "smeft"
                } else {
                    last
                };
                repo()
                    .join("validation/madgraph/cards")
                    .join(family)
                    .join(file)
            });
            UFOModel::load(&dir, card.as_deref()).unwrap()
        })
        .clone()
}

/// Every manifest row's process (and its amplitude-table process where that differs),
/// with the model the row declares, followed by [`MODEL_EXTRAS`] and by [`SM_EXTRAS`] on
/// the interned SM.
fn census_processes() -> Vec<(String, Arc<UFOModel>, String)> {
    let text = std::fs::read_to_string(repo().join("validation/manifest.toml")).unwrap();
    let manifest: toml::Value = toml::from_str(&text).unwrap();
    let mut models: HashMap<(String, Option<String>), Arc<UFOModel>> = HashMap::new();
    let sm = sm_model(SMRestrict::Default);
    let mut out: Vec<(String, Arc<UFOModel>, String)> = Vec::new();
    for row in manifest["process"].as_array().unwrap() {
        let key = row["key"].as_str().unwrap().to_owned();
        let model = match row.get("model").and_then(|m| m.as_str()) {
            None => sm.clone(),
            Some(dir) => {
                let restrict = row
                    .get("restrict")
                    .and_then(|r| r.as_str())
                    .map(str::to_owned);
                load(&mut models, dir, restrict.as_deref())
            }
        };
        let mut processes: Vec<String> = Vec::new();
        processes.extend(
            row.get("process")
                .and_then(|p| p.as_str())
                .map(str::to_owned),
        );
        processes.extend(
            row.get("mg_amplitude")
                .and_then(|m| m.get("process"))
                .and_then(|p| p.as_str())
                .map(str::to_owned),
        );
        processes.dedup();
        for p in processes {
            out.push((key.clone(), model.clone(), p));
        }
    }
    for &(dir, restrict, process) in MODEL_EXTRAS {
        let model = load(&mut models, dir, Some(restrict));
        out.push((format!("{dir}-{restrict}"), model, process.to_owned()));
    }
    for p in SM_EXTRAS {
        out.push(("sm".to_owned(), sm.clone(), (*p).to_owned()));
    }
    out
}

/// The diagram sets of one process line; a decay chain is stitched
/// ([`generate_decay_chains`]).
pub(super) fn generate(process: &str, model: &UFOModel) -> Vec<DiagramSet> {
    let text = format!("generate {process}");
    if process.contains(',') {
        let ast = parse_proc_card_ast(&text).unwrap();
        let card = check_enumerable(&ast).unwrap();
        return generate_decay_chains(&card, model).unwrap();
    }
    let opts = ParsingOptions::default();
    let card = parse_proc_card(&text, &opts).unwrap();
    generate_from_proc_card(&card, model).unwrap()
}

// ───────────────────────────── renumbering invariance ─────────────────────────────

/// Relative tolerance on a renumbered diagram's amplitudes. Renumbering can move the
/// production root (its tie-break reads vertex indices), which re-associates the
/// momentum sums the currents are built from; that is rounding at the 1e-15 level of
/// each value, far from the O(1) a sign or a structure error moves it by.
const AMP_REL_TOL: f64 = 1e-10;

/// Largest subprocess whose every diagram's amplitudes are compared.
const AMP_MAX_DIAGRAMS: usize = 40;

/// A generic phase-space point for `set`, on shell with `evaluated`'s masses.
pub(super) fn point(set: &DiagramSet, evaluated: &EvaluatedModel) -> Vec<LorentzVector<f64>> {
    let model = evaluated.model();
    let masses: Vec<f64> = set
        .particles_in
        .iter()
        .chain(&set.particles_out)
        .map(|n| evaluated.mass(model.particle_id(n).expect("external particle in model")))
        .collect();
    if set.particles_in.len() == 1 {
        // A decay at rest: the products share the mother's mass.
        let mut rng = ChaCha8Rng::seed_from_u64(0x5151);
        let mut momenta = vec![LorentzVector::new(masses[0], 0.0, 0.0, 0.0)];
        momenta.extend(rambo_massive(masses[0], &masses[1..], &mut rng));
        return momenta;
    }
    let sqrt_s = 1.5 * masses.iter().sum::<f64>() + 300.0;
    let (m1, m2) = (masses[0], masses[1]);
    let e1 = (sqrt_s * sqrt_s + m1 * m1 - m2 * m2) / (2.0 * sqrt_s);
    let p = (e1 * e1 - m1 * m1).sqrt();
    let mut momenta = vec![
        LorentzVector::new(e1, 0.0, 0.0, p),
        LorentzVector::new(sqrt_s - e1, 0.0, 0.0, -p),
    ];
    let mut rng = ChaCha8Rng::seed_from_u64(0x5151);
    momenta.extend(rambo_massive(sqrt_s, &masses[2..], &mut rng));
    momenta
}

/// Every helicity's per-flow amplitudes of a one-diagram set, in the evaluator's
/// helicity and flow order.
pub(super) fn single_diagram_amplitudes(
    set: &DiagramSet,
    diagram: &Diagram,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    momenta: &[LorentzVector<f64>],
) -> Vec<Complex<f64>> {
    let single = DiagramSet {
        particles_in: set.particles_in.clone(),
        particles_out: set.particles_out.clone(),
        polarizations: set.polarizations.clone(),
        diagrams: vec![diagram.clone()],
    };
    let evaluator = AmplitudeEvaluator::compile(&single, model).expect("compile one diagram");
    let bound = BoundAmplitude::<f64>::bind(&evaluator, evaluated);
    let mut scratch = bound.scratch_space();
    let mut out = Vec::new();
    for hel in evaluator.helicities() {
        if evaluator.n_flows() == 1 {
            out.push(bound.eval_amplitude(momenta, hel, &mut scratch));
        } else {
            out.extend(bound.run_flows(momenta, hel, &mut scratch));
        }
    }
    out
}

/// Two renumberings of `d`: a random one, and one that also moves whichever vertex is
/// numbered 0 away from index 0 (when there is more than one vertex), so every
/// convention still keyed to an index rather than to the graph is exercised.
fn renumberings(d: &Diagram, model: &UFOModel, rng: &mut ChaCha8Rng) -> Vec<(Vec<usize>, Diagram)> {
    let n_v = d.vertices.len();
    let n_p = d.props.len();
    let mut out = Vec::new();
    for forced in [false, true] {
        let mut vertex_order: Vec<usize> = (0..n_v).collect();
        vertex_order.shuffle(rng);
        if forced && n_v > 1 && vertex_order[0] == d.anchor().0 {
            vertex_order.swap(0, 1 + (rng.random::<u64>() as usize) % (n_v - 1));
        }
        let mut prop_order: Vec<usize> = (0..n_p).collect();
        prop_order.shuffle(rng);
        let flip: Vec<bool> = (0..n_p).map(|_| rng.random::<bool>()).collect();
        let renumbered = d.renumbered(&vertex_order, &prop_order, &flip, model);
        out.push((vertex_order, renumbered));
    }
    out
}

/// The compiled amplitude of a diagram does not depend on how its internal vertices and
/// propagators are numbered.
///
/// Every census diagram is renumbered at random — vertices and propagators permuted,
/// propagator orientations reversed — and compiled again. Its `fermi_sign` must be
/// exactly the original's for every colour chain, its anchor must be the same vertex,
/// its canonical form must be the original's, and (for subprocesses of at most
/// [`AMP_MAX_DIAGRAMS`] diagrams) its per-helicity, per-flow amplitudes at a fixed point
/// must agree to [`AMP_REL_TOL`]. A convention sign read off a vertex *index* rather
/// than off the graph fails this wherever it is not symmetric under the move, which the
/// forced renumbering (the anchor moved off index 0) makes sure is reached.
///
/// The canonical form is held to the other direction too: it is idempotent, and the
/// diagrams of one subprocess — which the enumeration produces distinct — keep distinct
/// canonical forms, so an equality that forgot part of the diagram would show here as a
/// collision.
///
/// Blind to anything two numberings share: a convention that is a function of the
/// graph but the wrong function passes here and is the amplitude oracle's to catch.
#[test]
fn renumbering_preserves_signs_and_amplitudes() {
    let mut rng = ChaCha8Rng::seed_from_u64(0x0d1a_9a3e);
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut failures: Vec<String> = Vec::new();
    let (mut n_diagrams, mut n_signs, mut n_amps, mut n_anchor_moved) = (0, 0, 0, 0);
    for (key, model, process) in census_processes() {
        let evaluated = EvaluatedModel::from_model(model.clone());
        for set in generate(&process, &model) {
            let sub = format!(
                "{} > {}",
                set.particles_in.join(" "),
                set.particles_out.join(" ")
            );
            if set.diagrams.is_empty() || !seen.insert((key.clone(), sub.clone())) {
                continue;
            }
            let canonical: HashSet<_> = set.diagrams.iter().map(|d| d.canonical(&model)).collect();
            if canonical.len() != set.diagrams.len() {
                failures.push(format!(
                    "{key} | {sub}: {} diagrams have only {} distinct canonical forms",
                    set.diagrams.len(),
                    canonical.len()
                ));
            }
            let basis = colorize_process(&model, &set.diagrams).expect("colorize");
            let with_amps = set.diagrams.len() <= AMP_MAX_DIAGRAMS && set.particles_out.len() >= 2;
            let momenta = with_amps.then(|| point(&set, &evaluated));
            for (di, d) in set.diagrams.iter().enumerate() {
                n_diagrams += 1;
                let c = d.canonical(&model);
                if c.diagram().canonical(&model) != c {
                    failures.push(format!(
                        "{key} | {sub} | diagram {di}: canonical form not idempotent"
                    ));
                }
                let chains: Vec<&Vec<u8>> = basis
                    .elements
                    .iter()
                    .flat_map(|e| e.contributions.iter())
                    .filter(|c| c.diagram == di)
                    .map(|c| &c.chain)
                    .collect();
                let base_amps = momenta
                    .as_ref()
                    .map(|m| single_diagram_amplitudes(&set, d, &model, &evaluated, m));
                for (vertex_order, r) in renumberings(d, &model, &mut rng) {
                    let tag = format!("{key} | {sub} | diagram {di} | order {vertex_order:?}");
                    n_anchor_moved += (r.anchor() != VtxIdx(0)) as usize;
                    if r.canonical(&model) != d.canonical(&model) {
                        failures.push(format!("{tag}: canonical forms differ"));
                    }
                    if vertex_order[r.anchor().0] != d.anchor().0 {
                        failures.push(format!("{tag}: the anchor moved to another vertex"));
                    }
                    for &chain in &chains {
                        let moved: Vec<u8> = vertex_order.iter().map(|&v| chain[v]).collect();
                        let a = compile_single_diagram(d, &model, chain).unwrap().fermi_sign;
                        let b = compile_single_diagram(&r, &model, &moved)
                            .unwrap()
                            .fermi_sign;
                        n_signs += 1;
                        if a != b {
                            failures.push(format!("{tag} chain {chain:?}: fermi_sign {a} -> {b}"));
                        }
                    }
                    if let (Some(base), Some(m)) = (&base_amps, &momenta) {
                        let amps = single_diagram_amplitudes(&set, &r, &model, &evaluated, m);
                        n_amps += 1;
                        let scale = base.iter().map(|c| c.norm()).fold(0.0, f64::max);
                        let worst = base
                            .iter()
                            .zip(&amps)
                            .map(|(a, b)| (a - b).norm())
                            .fold(0.0, f64::max);
                        if amps.len() != base.len() || worst > AMP_REL_TOL * scale {
                            failures.push(format!(
                                "{tag}: amplitudes moved by {worst:.3e} at scale {scale:.3e}"
                            ));
                        }
                    }
                }
            }
        }
    }
    println!(
        "{n_diagrams} diagrams, {n_signs} (renumbering, chain) signs, {n_amps} amplitude \
         comparisons, {n_anchor_moved} renumberings with the anchor off index 0, {} failures",
        failures.len()
    );
    for f in failures.iter().take(40) {
        println!("  {f}");
    }
    assert!(
        n_anchor_moved > 1000,
        "the renumberings barely move the anchor"
    );
    assert!(
        failures.is_empty(),
        "{} renumbering failures",
        failures.len()
    );
}

/// What a diagram stitched together from separate enumerations has to rebuild from the
/// graph is what the enumeration hands over: on every census diagram, the sign is the
/// fermion-pairing parity times the line sign, every propagator's momentum is the
/// graph's [`Diagram::tree_momentum`], and every propagator names the particle of the
/// interaction slot at `endpoints[1]` (and its antiparticle at `endpoints[0]`).
///
/// Each fails if the rebuilt rule is a different function of the graph from the one the
/// enumeration applies, on any census diagram that tells them apart.
#[test]
fn graph_rebuilt_fields_reproduce_the_enumeration() {
    use crate::diagrams::diagram::{antiparticle, PropIdx};
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut failures: Vec<String> = Vec::new();
    let (mut n_diagrams, mut n_props, mut n_charged) = (0, 0, 0);
    for (key, model, process) in census_processes() {
        for set in generate(&process, &model) {
            let sub = format!(
                "{} > {}",
                set.particles_in.join(" "),
                set.particles_out.join(" ")
            );
            if set.diagrams.is_empty() || !seen.insert((key.clone(), sub.clone())) {
                continue;
            }
            for (di, d) in set.diagrams.iter().enumerate() {
                n_diagrams += 1;
                let tag = format!("{key} | {sub} | diagram {di}");
                let rebuilt = d.fermion_pairing_sign(&model) * d.fermion_line_sign(&model);
                if rebuilt != d.sign {
                    failures.push(format!("{tag}: sign {} rebuilt as {rebuilt}", d.sign));
                }
                for (pi, p) in d.props.iter().enumerate() {
                    n_props += 1;
                    let momentum = d.tree_momentum(PropIdx(pi));
                    if momentum != p.momentum {
                        failures.push(format!(
                            "{tag} prop {pi}: momentum {:?} rebuilt as {momentum:?}",
                            p.momentum
                        ));
                    }
                    let slot = |end: usize| {
                        let (v, s) = p.endpoints[end];
                        model.vertex_def(d.vertex(v).interaction).particles[s.0]
                    };
                    n_charged += usize::from(antiparticle(&model, p.particle) != p.particle);
                    if slot(1) != p.particle || slot(0) != antiparticle(&model, p.particle) {
                        failures.push(format!(
                            "{tag} prop {pi}: particle {} between slots {} and {}",
                            model.particle(p.particle).name,
                            model.particle(slot(0)).name,
                            model.particle(slot(1)).name
                        ));
                    }
                }
            }
        }
    }
    println!(
        "{n_diagrams} diagrams, {n_props} propagators ({n_charged} not self-conjugate), {} \
         failures",
        failures.len()
    );
    for f in failures.iter().take(40) {
        println!("  {f}");
    }
    assert!(
        n_charged > 100,
        "too few charged propagators to pin the orientation"
    );
    assert!(failures.is_empty(), "{} rebuild failures", failures.len());
}
