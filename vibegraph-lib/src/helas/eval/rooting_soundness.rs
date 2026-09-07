//! Rooting-soundness gate.
//!
//! Every Feynman diagram is a tree; choosing which vertex to root it at orients its
//! internal edges but must not change the physics. The whole amplitude machinery —
//! momentum routing, Lorentz-output rooting, fermion-spine signs — was originally
//! validated only for feyngraph's `VtxIdx(0)` orientation; the convention signs are read
//! off that canonical rooting so the honest currents stay root-invariant. This module
//! drives the [`super::root_diagram`] test hook to re-root diagrams and asserts the |M|²
//! is invariant under the root choice, which is what lets production root each diagram at
//! [`canonical_root`](super::root_diagram) instead.
//!
//! The oracle is the baseline (unoverridden) |M|² itself, not MadGraph: the production
//! rooting is already pinned against MG by `tests/amplitude_oracle.rs`, so any rooting
//! that reproduces the baseline is correct and any that does not is a soundness bug. The
//! comparison uses `REL_TOL`, since re-rooting reassociates momentum sums and is never
//! bit-for-bit even when it is correct.
//!
//! Full sweep (passes — the rooting-dependent convention signs are lifted to the
//! diagram's `fermi_sign` at the canonical rooting; see `research/notes/19` §V5):
//! ```text
//! RUST_MIN_STACK=134217728 cargo test -p vibegraph-lib \
//!     --lib helas::eval::rooting_soundness::all_rootings_preserve_amplitude \
//!     -- --ignored --nocapture --test-threads=1
//! ```

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::compile::{AmplitudeEvaluator, MG_VALIDATED_PROCESSES};
use super::root_diagram::{clear_root_override, set_root_override};
use super::run::BoundAmplitude;

use crate::diagrams::diagram::{Diagram, VtxIdx};
use crate::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
use crate::helas::LorentzVector;
use crate::ufo::slha::ParamCard;
use crate::ufo::sm::{sm_model, SMRestrict};
use crate::ufo::{EvaluatedModel, UFOModel};

/// Relative tolerance for the invariance check.
///
/// A correct re-rooting reassociates the momentum sums that route each propagator (the
/// off-shell current momenta are accumulated in a different order), so agreement against
/// the baseline is never bit-for-bit and the floor is *looser* than the amplitude-level
/// reordering `tests/amplitude_oracle.rs` pins at 1e-12. The rooting-dependent **signs**
/// are all lifted to the diagram's `fermi_sign` (build-convention, spine, reversed-
/// bilinear — all computed at the canonical `VtxIdx(0)` rooting), so a surviving
/// deviation here is pure double-precision reassociation: the observed worst case across
/// the MG-validated suite is 2.2e-11 (`e+e-→τ+τ-H`, an 8-momentum sum). This rides a few×
/// above that floor and still an enormous margin below any sign/structure error (O(1)).
const REL_TOL: f64 = 1e-10;

/// Reference momenta cap per process: enough to expose both failure regimes the study
/// found (gross wrong amplitude at max_rel 1e-2…1e+3, and benign over-tolerance
/// reassociation at ~1e-11), without paying for all 50 CSV points on every re-rooting.
const MAX_POINTS: usize = 6;

/// Diagram-count ceiling for per-diagram root isolation. Above it, a process re-roots
/// whole-process instead (see [`all_rootings_preserve_amplitude`]); this keeps the two
/// 579/615-diagram 8-point processes out of the O(Σ vertices)-compiles regime.
const PER_DIAGRAM_MAX: usize = 40;

// ───────────────────────────── reference-point plumbing ─────────────────────────────

/// A structural fingerprint of a diagram, stable within a process. Used to target the
/// root override at one specific diagram (all others stay at `VtxIdx(0)`), isolating
/// which diagram a soundness failure lives in.
fn diagram_key(d: &Diagram) -> u64 {
    let mut h = DefaultHasher::new();
    format!("{d:?}").hash(&mut h);
    h.finish()
}

fn normalize(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn generate(process: &str) -> Vec<DiagramSet> {
    let opts = ParsingOptions::default();
    let card = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
    generate_from_proc_card(&card, &sm_model(SMRestrict::Default)).unwrap()
}

/// One process's committed amplitude table, reduced to what a re-rooting sweep
/// needs: the first [`MAX_POINTS`] phase-space points and the param card
/// MadGraph evaluated them with. MadGraph's own `|M|²` column is not read — the
/// oracle here is the baseline vibegraph value, not MadGraph's.
struct Reference {
    momenta: Vec<Vec<LorentzVector<f64>>>,
    card: ParamCard,
}

/// Map normalised process string -> its committed table.
///
/// Two tables can carry the same process string (`u u~ > mu+ mu-` is generated
/// both on its own and as the concrete subprocess of a group), so the first in
/// filename order wins and the choice does not depend on directory order.
fn table_index() -> HashMap<String, Reference> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/amplitudes");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();

    let mut map = HashMap::new();
    for path in paths {
        let text = std::fs::read_to_string(&path).expect("amplitude table");
        let json: serde_json::Value = serde_json::from_str(&text).expect("amplitude table");
        let process = normalize(json["process"].as_str().expect("process"));
        if map.contains_key(&process) {
            continue;
        }
        let momenta = json["points"]
            .as_array()
            .expect("points")
            .iter()
            .take(MAX_POINTS)
            .map(|pt| {
                pt["momenta"]
                    .as_array()
                    .expect("momenta")
                    .iter()
                    .map(|m| {
                        let c: Vec<f64> = m
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_f64().unwrap())
                            .collect();
                        LorentzVector::new(c[0], c[1], c[2], c[3])
                    })
                    .collect()
            })
            .collect();
        let card = json["param_card"]
            .as_array()
            .expect("param_card")
            .iter()
            .map(|l| l.as_str().unwrap())
            .collect::<Vec<_>>()
            .join("\n")
            .parse::<ParamCard>()
            .expect("param_card");
        map.insert(process, Reference { momenta, card });
    }
    map
}

/// Compile `set` under whatever root override is currently installed and evaluate |M|² at
/// every reference point. `Err` carries a compile error or a caught panic message.
fn eval_m2_all(
    set: &DiagramSet,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    points: &[Vec<LorentzVector<f64>>],
) -> Result<Vec<f64>, String> {
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let evaluator =
            AmplitudeEvaluator::compile(set, model).map_err(|e| format!("compile: {e}"))?;
        let bound = BoundAmplitude::<f64>::bind(&evaluator, evaluated);
        let mut scratch = bound.scratch_space();
        Ok(points
            .iter()
            .map(|m| bound.eval_m2(m, &mut scratch))
            .collect::<Vec<f64>>())
    }));
    match result {
        Ok(inner) => inner,
        Err(_) => Err("PANIC".to_owned()),
    }
}

/// Largest relative deviation of `cand` from `base` over the point set.
fn max_rel(base: &[f64], cand: &[f64]) -> f64 {
    base.iter()
        .zip(cand)
        .map(|(&b, &c)| (c - b).abs() / b.abs().max(1e-30))
        .fold(0.0f64, f64::max)
}

// ───────────────────────────── the gate ─────────────────────────────

/// One diagram re-rooted at one non-baseline vertex, and how badly it broke.
/// `diagram == usize::MAX` marks a whole-process re-rooting (no single-diagram locus).
struct Failure {
    process: &'static str,
    diagram: usize,
    n_vertices: usize,
    root: usize,
    detail: String,
}

/// Compare one re-rooted evaluation against the baseline and push a [`Failure`] if it
/// panicked or drifted past `REL_TOL`.
fn record(
    failures: &mut Vec<Failure>,
    process: &'static str,
    diagram: usize,
    n_vertices: usize,
    root: usize,
    base: &[f64],
    cand: Result<Vec<f64>, String>,
) {
    let detail = match cand {
        Err(e) => e,
        Ok(cand) => {
            let mr = max_rel(base, &cand);
            if mr <= REL_TOL {
                return;
            }
            format!("max_rel={mr:.2e}")
        }
    };
    failures.push(Failure {
        process,
        diagram,
        n_vertices,
        root,
        detail,
    });
}

/// Sweep every vertex of every diagram of every MG-validated process as an alternative
/// root (all *other* diagrams held at `VtxIdx(0)`), and require the resulting |M|² to
/// match the baseline within `REL_TOL`. A correct amplitude is root-invariant; a
/// deviation is a soundness bug in momentum routing / Lorentz-output rooting /
/// fermion-spine signs (see `research/notes/19` §V5).
///
/// This passes: the honest currents are rooting-invariant tensors and every
/// rooting-dependent convention sign (build-convention, spine, reversed-bilinear) is
/// lifted to the diagram's `fermi_sign` at the canonical `VtxIdx(0)` rooting. Ignored
/// only because the full O(Σ vertices) recompile sweep is slow; run it explicitly after
/// touching the rooting / Lorentz-output / fermion-sign machinery.
#[test]
#[ignore = "rooting-soundness gate: slow full sweep; run explicitly with --ignored"]
fn all_rootings_preserve_amplitude() {
    let model = Arc::new(sm_model(SMRestrict::Default));
    let tables = table_index();
    let mut failures: Vec<Failure> = Vec::new();
    let mut swept = 0usize;

    for process in MG_VALIDATED_PROCESSES {
        let sets = generate(process);
        let set = &sets[0];
        let reference = tables
            .get(&normalize(process))
            .unwrap_or_else(|| panic!("no committed amplitude table for '{process}'"));
        let points = &reference.momenta;
        let evaluated = EvaluatedModel::from_model_card((*model).clone(), &reference.card);

        // Baseline oracle: production rooting.
        clear_root_override();
        let base = match eval_m2_all(set, &model, &evaluated, points) {
            Ok(v) => v,
            Err(e) => {
                failures.push(Failure {
                    process,
                    diagram: usize::MAX,
                    n_vertices: 0,
                    root: 0,
                    detail: format!("baseline eval failed: {e}"),
                });
                continue;
            }
        };

        // Per candidate re-rooting we recompile the *whole* amplitude, so per-diagram
        // isolation costs O(Σ vertices) full compiles — fine for the small processes,
        // where it pinpoints the offending diagram, but prohibitive for the 579/615-
        // diagram 8-point processes. Those fall back to whole-process re-rooting (root
        // *every* diagram at the same vertex position, ~max_vertices compiles), which
        // still detects orientation-dependence, just without the per-diagram locus.
        if set.diagrams.len() <= PER_DIAGRAM_MAX {
            for di in 0..set.diagrams.len() {
                let n_vertices = set.diagrams[di].vertices.len();
                let target = diagram_key(&set.diagrams[di]);
                for r in 1..n_vertices {
                    // Re-root only the target diagram; leave the rest at VtxIdx(0).
                    set_root_override(Box::new(move |d| {
                        if diagram_key(d) == target {
                            VtxIdx(r)
                        } else {
                            VtxIdx(0)
                        }
                    }));
                    let cand = eval_m2_all(set, &model, &evaluated, points);
                    clear_root_override();
                    swept += 1;
                    record(&mut failures, process, di, n_vertices, r, &base, cand);
                }
            }
        } else {
            let max_v = set
                .diagrams
                .iter()
                .map(|d| d.vertices.len())
                .max()
                .unwrap_or(1);
            for s in 1..max_v {
                // Root every diagram at vertex position `s` (clamped into range).
                set_root_override(Box::new(move |d| VtxIdx(s.min(d.vertices.len() - 1))));
                let cand = eval_m2_all(set, &model, &evaluated, points);
                clear_root_override();
                swept += 1;
                record(&mut failures, process, usize::MAX, max_v, s, &base, cand);
            }
        }
    }
    clear_root_override();

    // Report a compact per-process tally plus the worst few loci.
    let mut per_process: HashMap<&str, usize> = HashMap::new();
    for f in &failures {
        *per_process.entry(f.process).or_insert(0) += 1;
    }
    println!(
        "\nrooting-soundness sweep: {swept} re-rootings, {} failures across {} processes\n",
        failures.len(),
        per_process.len()
    );
    let mut tally: Vec<_> = per_process.iter().collect();
    tally.sort_by_key(|(p, _)| *p);
    for (p, n) in tally {
        println!("  {p}: {n} failing re-rootings");
    }
    for f in failures.iter().take(40) {
        let locus = if f.diagram == usize::MAX {
            format!("whole-process, all diagrams at vertex position {}", f.root)
        } else {
            format!(
                "diagram {}/{} root VtxIdx({})",
                f.diagram,
                f.n_vertices.saturating_sub(1),
                f.root
            )
        };
        println!("  FAIL {} {locus}: {}", f.process, f.detail);
    }

    assert!(
        failures.is_empty(),
        "{} re-rootings changed |M|² — production rooting is orientation-dependent",
        failures.len()
    );
}

/// Fast guard (runs under default `cargo test`): the override hook is wired into the
/// production compile path and an explicit `VtxIdx(0)` override reproduces the baseline
/// |M|² bit-for-bit — i.e. the hook itself is transparent. Keeps the machinery honest
/// without the slow, currently-failing full sweep.
#[test]
fn root_override_hook_is_transparent() {
    let model = Arc::new(sm_model(SMRestrict::Default));
    let tables = table_index();
    for process in ["e+ e- > mu+ mu-", "e+ e- > e+ e-"] {
        let sets = generate(process);
        let set = &sets[0];
        let reference = tables
            .get(&normalize(process))
            .unwrap_or_else(|| panic!("no committed amplitude table for '{process}'"));
        let points = &reference.momenta;
        let evaluated = EvaluatedModel::from_model_card((*model).clone(), &reference.card);

        clear_root_override();
        let base = eval_m2_all(set, &model, &evaluated, points).expect("baseline eval");

        set_root_override(Box::new(|_| VtxIdx(0)));
        let forced = eval_m2_all(set, &model, &evaluated, points).expect("forced-0 eval");
        clear_root_override();

        for (b, f) in base.iter().zip(&forced) {
            assert_eq!(
                b.to_bits(),
                f.to_bits(),
                "explicit VtxIdx(0) override diverged from default rooting for {process}"
            );
        }
    }
    clear_root_override();
}

/// A momentum-slashed γ-chain is rooting-invariant, including at the vertex's own
/// vector leg.
///
/// SMEFTsim's top dipoles (`FFV9`, and `FFV2` with a `Gamma5` in the middle) are the
/// first structures in the suite whose `P` names the *output* leg of a vertex that
/// also closes a fermion pair. Rooted at either fermion leg that `P` reads a bound
/// leg's momentum; rooted at the vector leg it becomes `PMomOut`, the negated
/// all-incoming sum over the vertex's inputs — and a fermion pair enters that sum as
/// `p_bra − p_ket`, not as `p_bra + p_ket`, because a fermion current stores the
/// momentum flowing along its line rather than into the vertex. Summing the pair
/// with two plus signs leaves every other row of the suite untouched (no Standard
/// Model structure puts a `P` on an `FFV` output leg) and moves this one by percent,
/// which is why the falsifier is here rather than left to the process gate.
///
/// Fast enough to run unignored: ten diagrams, two rootings, one phase-space point.
#[test]
#[cfg(feature = "extended-validation")]
fn momentum_slashed_chain_is_rooting_invariant() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let model = Arc::new(
        UFOModel::load(
            &root.join("../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO"),
            Some(&root.join("../validation/madgraph/cards/smeft/restrict_vg_ctdipole.dat")),
        )
        .expect("load SMEFTsim under the dipole card"),
    );
    let table: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            root.join("../validation/madgraph/amplitudes/ee_to_ttx_dipole.json"),
        )
        .expect("the banked dipole table"),
    )
    .expect("parse the banked dipole table");
    let card: ParamCard = table["param_card"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        .parse()
        .expect("the banked param card");
    let evaluated = EvaluatedModel::from_model_card((*model).clone(), &card);
    let opts = ParsingOptions::default();
    let pc = parse_proc_card("generate e+ e- > t t~ NP<=1", &opts).unwrap();
    let sets = generate_from_proc_card(&pc, model.as_ref()).unwrap();
    let set = sets.iter().find(|s| !s.diagrams.is_empty()).unwrap();
    let point = &table["points"][0];
    let momenta: Vec<LorentzVector<f64>> = point["momenta"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            let c: Vec<f64> = m
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect();
            LorentzVector::new(c[0], c[1], c[2], c[3])
        })
        .collect();
    let hel: Vec<i32> = table["helicities"]
        [point["detail"]["helicities"][0].as_u64().unwrap() as usize]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as i32)
        .collect();

    let per_root = |r: usize| -> Vec<num_complex::Complex<f64>> {
        set_root_override(Box::new(move |_| VtxIdx(r)));
        let values = set
            .diagrams
            .iter()
            .map(|d| {
                let one = DiagramSet {
                    particles_in: set.particles_in.clone(),
                    particles_out: set.particles_out.clone(),
                    diagrams: vec![d.clone()],
                };
                let ev = AmplitudeEvaluator::compile(&one, model.as_ref()).unwrap();
                let bound = BoundAmplitude::<f64>::bind(&ev, &evaluated);
                let mut scratch = bound.scratch_space();
                bound.eval_amplitude(&momenta, &hel, &mut scratch)
            })
            .collect();
        clear_root_override();
        values
    };

    let (at_fermion, at_vector) = (per_root(1), per_root(0));
    let scale = at_fermion
        .iter()
        .fold(0.0f64, |m, z| m.max(z.norm()))
        .max(1e-300);
    for (d, (a, b)) in at_fermion.iter().zip(&at_vector).enumerate() {
        assert!(
            (a - b).norm() / scale < REL_TOL,
            "diagram {d}: rooted at a fermion leg {a:?}, at the vector leg {b:?}"
        );
    }
}

/// A four-fermion vertex with an *off-shell* fermion output, re-rooted.
///
/// Every four-fermion diagram of the gated rows is a single contact vertex, which has
/// exactly one rooting, so the row cannot reach the code that carries a fermion line
/// *through* such a vertex: three of its four fermion legs are inputs, two of them
/// close a line there and the third continues into the output. Adding a photon to
/// `e+ e- > mu+ mu-` puts the contact vertex on an internal muon line and makes that
/// path reachable, and re-rooting is its falsifier — the continuing input is chosen by
/// the vertex's own pairing, so picking the first fermion child instead (the rule that
/// was right while every sink had two fermion legs) sends the line through the wrong
/// leg at some rootings and not others.
///
/// The oracle is the amplitude's invariance under the root choice, as elsewhere in this
/// module; the row itself has no MadGraph reference.
#[test]
#[cfg(feature = "extended-validation")]
fn four_fermion_currents_are_rooting_invariant() {
    use crate::phasespace::rambo_massless;
    use rand::SeedableRng;

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let model = Arc::new(
        UFOModel::load(
            &root.join("../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO"),
            Some(&root.join("../validation/madgraph/cards/smeft/restrict_vg_c4l.dat")),
        )
        .expect("load SMEFTsim under the four-lepton card"),
    );
    let table: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("../validation/madgraph/amplitudes/ee_to_mumu_4f.json"))
            .expect("the banked four-lepton table"),
    )
    .expect("parse the banked four-lepton table");
    let card: ParamCard = table["param_card"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        .parse()
        .expect("the banked param card");
    let evaluated = EvaluatedModel::from_model_card((*model).clone(), &card);
    let opts = ParsingOptions::default();
    let pc = parse_proc_card("generate e+ e- > mu+ mu- a NP<=1", &opts).unwrap();
    let sets = generate_from_proc_card(&pc, model.as_ref()).unwrap();
    let set = sets.iter().find(|s| !s.diagrams.is_empty()).unwrap();

    // Diagrams whose four-fermion vertex is not the whole diagram: those are the ones
    // that carry a fermion line through it.
    let through: Vec<&Diagram> = set
        .diagrams
        .iter()
        .filter(|d| {
            d.vertices.len() > 1
                && d.vertices.iter().any(|v| {
                    model
                        .vertex_def(v.interaction)
                        .particles
                        .iter()
                        .filter(|&&p| model.particle(p).spin == 2)
                        .count()
                        == 4
                })
        })
        .collect();
    assert!(
        through.len() >= 4,
        "expected several diagrams routing a fermion line through a four-fermion \
         vertex, found {}",
        through.len()
    );

    let sqrt_s = 500.0;
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0x4f4f_4f4f);
    let mut momenta = vec![
        LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
        LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
    ];
    momenta.extend(rambo_massless(sqrt_s, 3, &mut rng));

    // Every helicity combination, so the comparison is over amplitudes that are
    // actually non-zero: most of a four-fermion contact's combinations vanish, and a
    // vanishing amplitude compares equal to anything.
    let helicities: Vec<Vec<i32>> = (0..32)
        .map(|m: usize| {
            (0..5)
                .map(|i| if m >> i & 1 == 1 { 1 } else { -1 })
                .collect()
        })
        .collect();

    // Control: the same comparison on this process's diagrams that have no
    // four-fermion vertex, so a failure above is the four-fermion path and not the
    // harness. (It was: the consecutive-slot reading of the spinor pairs sent the
    // rooted output to a leg of the other line, and only the four-fermion diagrams
    // moved.)
    let control: Vec<&Diagram> = set
        .diagrams
        .iter()
        .filter(|d| {
            !d.vertices.iter().any(|v| {
                model
                    .vertex_def(v.interaction)
                    .particles
                    .iter()
                    .filter(|&&p| model.particle(p).spin == 2)
                    .count()
                    == 4
            })
        })
        .collect();
    let mut compared = 0usize;
    for (d, diagram) in control.iter().chain(through.iter()).enumerate() {
        let per_root = |r: usize| -> Vec<num_complex::Complex<f64>> {
            set_root_override(Box::new(move |_| VtxIdx(r)));
            let one = DiagramSet {
                particles_in: set.particles_in.clone(),
                particles_out: set.particles_out.clone(),
                diagrams: vec![(*diagram).clone()],
            };
            let ev = AmplitudeEvaluator::compile(&one, model.as_ref()).unwrap();
            let bound = BoundAmplitude::<f64>::bind(&ev, &evaluated);
            let mut scratch = bound.scratch_space();
            let values = helicities
                .iter()
                .map(|hel| bound.eval_amplitude(&momenta, hel, &mut scratch))
                .collect();
            clear_root_override();
            values
        };
        let base = per_root(0);
        let scale = base.iter().fold(0.0f64, |m, z| m.max(z.norm()));
        assert!(scale > 0.0, "diagram {d} vanishes at every helicity");
        for r in 1..diagram.vertices.len() {
            for (h, (a, b)) in base.iter().zip(&per_root(r)).enumerate() {
                assert!(
                    (a - b).norm() / scale < REL_TOL,
                    "diagram {d} helicity {h}: rooted at vertex 0 {a:?}, at vertex {r} {b:?}"
                );
                compared += 1;
            }
        }
    }
    assert!(compared >= 100, "only {compared} amplitudes compared");
}

/// A cyclic tensor⊗tensor four-fermion vertex with an *off-shell* fermion output,
/// re-rooted.
///
/// The gated row's contact saturates its four external legs, so it has exactly one
/// rooting — the amplitude sink — and never reaches the path that carries a fermion line
/// *through* a tensor contact: the cycle is then cut at the line the output leg is not
/// on, the surviving line's continuing spinor takes the cut line's Clifford element as
/// an operator (`MultivectorIout`/`MultivectorOout`), and the output leg's own chiral
/// projector applies to the current the vertex produces rather than to any input.
/// Adding a photon to `ta+ ta- > t t~` puts the contact on an internal fermion line and
/// makes all of that reachable.
///
/// Re-rooting is its falsifier twice over. The amplitude must not depend on which
/// vertex is the root, which is what pins the element's momentum routing (a fermion pair
/// hands over `p_bra − p_ket`, and the continuing current subtracts or adds it by its
/// own adjoint — the sign no single-vertex row can see); and rooting at the other vertex
/// swaps which of the two fermion lines is cut, so the two cuts are compared against
/// each other on a real process rather than only on random spinors.
///
/// The oracle is the amplitude's invariance under the root choice, as elsewhere in this
/// module; the row itself has no MadGraph reference.
#[test]
#[cfg(feature = "extended-validation")]
fn tensor_four_fermion_currents_are_rooting_invariant() {
    use crate::phasespace::rambo_massless;
    use rand::SeedableRng;

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let model = Arc::new(
        UFOModel::load(
            &root.join("../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO"),
            Some(&root.join("../validation/madgraph/cards/smeft/restrict_vg_cleQt3.dat")),
        )
        .expect("load SMEFTsim under the tensor four-fermion card"),
    );
    let table: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            root.join("../validation/madgraph/amplitudes/tata_to_ttx_tensor4f.json"),
        )
        .expect("the banked tensor four-fermion table"),
    )
    .expect("parse the banked tensor four-fermion table");
    let card: ParamCard = table["param_card"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        .parse()
        .expect("the banked param card");
    let evaluated = EvaluatedModel::from_model_card((*model).clone(), &card);
    let opts = ParsingOptions::default();
    let pc = parse_proc_card("generate ta+ ta- > t t~ a NP<=1", &opts).unwrap();
    let sets = generate_from_proc_card(&pc, model.as_ref()).unwrap();
    let set = sets.iter().find(|s| !s.diagrams.is_empty()).unwrap();

    // The diagrams whose four-fermion vertex is not the whole diagram: those carry a
    // fermion line through it.
    let is_four_fermion = |v: &crate::diagrams::diagram::Vertex| {
        model
            .vertex_def(v.interaction)
            .particles
            .iter()
            .filter(|&&p| model.particle(p).spin == 2)
            .count()
            == 4
    };
    let through: Vec<&Diagram> = set
        .diagrams
        .iter()
        .filter(|d| d.vertices.len() > 1 && d.vertices.iter().any(is_four_fermion))
        .collect();
    assert!(
        through.len() >= 4,
        "expected several diagrams routing a fermion line through a tensor four-fermion \
         vertex, found {}",
        through.len()
    );

    let sqrt_s = 500.0;
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0x7e_4f_4f_4f);
    let mut momenta = vec![
        LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
        LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
    ];
    momenta.extend(rambo_massless(sqrt_s, 3, &mut rng));

    let helicities: Vec<Vec<i32>> = (0..32)
        .map(|m: usize| {
            (0..5)
                .map(|i| if m >> i & 1 == 1 { 1 } else { -1 })
                .collect()
        })
        .collect();

    let mut compared = 0usize;
    for (d, diagram) in through.iter().enumerate() {
        let per_root = |r: usize| -> Vec<num_complex::Complex<f64>> {
            set_root_override(Box::new(move |_| VtxIdx(r)));
            let one = DiagramSet {
                particles_in: set.particles_in.clone(),
                particles_out: set.particles_out.clone(),
                diagrams: vec![(*diagram).clone()],
            };
            let ev = AmplitudeEvaluator::compile(&one, model.as_ref()).unwrap();
            let bound = BoundAmplitude::<f64>::bind(&ev, &evaluated);
            let mut scratch = bound.scratch_space();
            let values = helicities
                .iter()
                .map(|hel| bound.eval_amplitude(&momenta, hel, &mut scratch))
                .collect();
            clear_root_override();
            values
        };
        let base = per_root(0);
        let scale = base.iter().fold(0.0f64, |m, z| m.max(z.norm()));
        assert!(scale > 0.0, "diagram {d} vanishes at every helicity");
        for r in 1..diagram.vertices.len() {
            for (h, (a, b)) in base.iter().zip(&per_root(r)).enumerate() {
                assert!(
                    (a - b).norm() / scale < REL_TOL,
                    "diagram {d} helicity {h}: rooted at vertex 0 {a:?}, at vertex {r} {b:?}"
                );
                compared += 1;
            }
        }
    }
    assert!(compared >= 100, "only {compared} amplitudes compared");
}

/// The toy model's literal-`Sigma` structures with an *off-shell* output, re-rooted.
///
/// The two banked toy rows put each structure in a vertex all of whose legs are
/// external, so between them they reach only the vector-leg rooting (the dipole's
/// s-channel current) and the amplitude sink (the contact). Radiating a `vt` off the
/// quark line puts both on an internal fermion line, which is what reaches the
/// fermion-leg rootings: the dipole becomes a Clifford element acting on the continuing
/// spinor (`SigmaMv` + `MultivectorIout`/`MultivectorOout`), and the `Sigma ⊗ Sigma`
/// contact cuts the line the output leg is not on.
///
/// Re-rooting is the falsifier. The amplitude must not depend on which vertex is the
/// root, so it pins the element's momentum routing, the `±` a line read against the
/// vertex's own adjoint takes, and — for the dipole — that a `Sigma`-chained chiral
/// projector keeps its chirality: conjugating it at one rooting and not at another
/// moves the amplitude. The oracle is the baseline amplitude itself, as everywhere in
/// this module; the absolute conventions are the banked rows'.
///
/// Hermetic: the model, its restrict cards and their parameter defaults are all in
/// tree, and MadGraph is not consulted.
#[test]
fn literal_sigma_currents_are_rooting_invariant() {
    use super::op::Op;
    use super::tree::Tree;
    use crate::phasespace::rambo_massless;
    use rand::SeedableRng;

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = root.join("../validation/ufo/vibegraph_toy_UFO");
    // (card, process, the ops the structure must reach — an empty intersection would
    // make the whole comparison vacuous).
    let cases: [(&str, &str, &[Op]); 2] = [
        (
            "restrict_dipole.dat",
            "lt~ lt > qt qt~ vt NP<=1",
            &[Op::SigmaVout, Op::SigmaMv],
        ),
        (
            "restrict_tensor.dat",
            "lt~ lt > qt qt~ vt NP<=1 NPGG<=1",
            &[Op::SigmaOut],
        ),
    ];

    let sqrt_s = 900.0;
    let mut compared = 0usize;
    for (card, process, must_reach) in cases {
        let model =
            Arc::new(UFOModel::load(&dir, Some(&dir.join(card))).expect("load the toy model"));
        let evaluated = EvaluatedModel::from_model(Arc::clone(&model));
        let opts = ParsingOptions::default();
        let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
        let sets = generate_from_proc_card(&pc, model.as_ref()).unwrap();
        let set = sets.iter().find(|s| !s.diagrams.is_empty()).unwrap();

        let ev = AmplitudeEvaluator::compile(set, model.as_ref()).unwrap();
        let ast = &ev.folded().ast;
        let reached: Vec<Op> = ast.iter().map(|id| ast.value(id).op).collect();
        for op in must_reach {
            assert!(
                reached.contains(op),
                "[{card}] '{process}' never reaches {op:?}, so this comparison is vacuous"
            );
        }

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0x5169_4a5f);
        let mut momenta = vec![
            LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
            LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
        ];
        momenta.extend(rambo_massless(sqrt_s, 3, &mut rng));
        let helicities: Vec<Vec<i32>> = (0..32)
            .map(|m: usize| {
                (0..5)
                    .map(|i| if m >> i & 1 == 1 { 1 } else { -1 })
                    .collect()
            })
            .collect();

        let multi: Vec<&Diagram> = set
            .diagrams
            .iter()
            .filter(|d| d.vertices.len() > 1)
            .collect();
        assert!(
            !multi.is_empty(),
            "[{card}] no diagram with an internal line to re-root"
        );
        for (d, diagram) in multi.iter().enumerate() {
            let per_root = |r: usize| -> Vec<num_complex::Complex<f64>> {
                set_root_override(Box::new(move |_| VtxIdx(r)));
                let one = DiagramSet {
                    particles_in: set.particles_in.clone(),
                    particles_out: set.particles_out.clone(),
                    diagrams: vec![(*diagram).clone()],
                };
                let ev = AmplitudeEvaluator::compile(&one, model.as_ref()).unwrap();
                let bound = BoundAmplitude::<f64>::bind(&ev, &evaluated);
                let mut scratch = bound.scratch_space();
                let values = helicities
                    .iter()
                    .map(|hel| bound.eval_amplitude(&momenta, hel, &mut scratch))
                    .collect();
                clear_root_override();
                values
            };
            let base = per_root(0);
            let scale = base.iter().fold(0.0f64, |m, z| m.max(z.norm()));
            assert!(
                scale > 0.0,
                "[{card}] diagram {d} vanishes at every helicity"
            );
            for r in 1..diagram.vertices.len() {
                for (h, (a, b)) in base.iter().zip(&per_root(r)).enumerate() {
                    assert!(
                        (a - b).norm() / scale < REL_TOL,
                        "[{card}] diagram {d} helicity {h}: rooted at vertex 0 {a:?}, \
                         at vertex {r} {b:?}"
                    );
                    compared += 1;
                }
            }
        }
    }
    assert!(compared >= 100, "only {compared} amplitudes compared");
}
