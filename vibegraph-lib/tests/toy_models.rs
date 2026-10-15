//! The two authored toy UFO models as a coverage instrument.
//!
//! `vibegraph_toy_UFO` and `vibegraph_toy_color_UFO` exist because no model in
//! reach writes the structures they carry: a literal `Sigma`, the bare
//! `Identity` and `Gamma5` bilinears, the symmetric structure constant `d`, the
//! baryonic `Epsilon` and the sextet Clebsch coefficients `K6`. Every one of
//! their rows is enforced against MadGraph at the amplitude level, which is what
//! makes the census below a statement about MadGraph's reference rather than
//! about this crate's own arithmetic.
//!
//! The same three questions `smeftsim.rs` asks of the vendored model, asked of
//! these two at once: the list this file runs on is the manifest's own gated set,
//! each row's process string is the one its banked amplitude table was generated
//! for, and the ops the rows compile to are measured against an allowlist that
//! must shrink as coverage grows.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;

use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::LorentzVector;
use vibegraph::phasespace::rambo;
use vibegraph::ufo::{EvaluatedModel, UFOModel};

/// The two model directories whose rows this file owns, as the manifest spells
/// them.
const TOY_MODEL_DIRS: [&str; 2] = [
    "validation/ufo/vibegraph_toy_UFO",
    "validation/ufo/vibegraph_toy_color_UFO",
];

/// The toy rows the manifest gates, with the process each one enumerates.
///
/// Order bounds are part of the process string, and here they are load-bearing
/// twice over: each structure family has a coupling order of its own, which is
/// what makes MadGraph split a two-structure vertex into two interactions and so
/// into two diagrams with an `AMP()` each, and MadGraph's WEIGHTED default would
/// drop the higher-order half of every interference these rows are built around.
const GATED_ROWS: [(&str, &str); 6] = [
    ("ll_to_qqx_toy_dipole", "lt~ lt > qt qt~ NP<=1"),
    ("ll_to_qqx_toy_tensor", "lt~ lt > qt qt~ NP<=1 NPGG<=1"),
    ("ll_to_qqx_toy_yukawa", "lt~ lt > qt qt~ NP<=2 NPCP<=2"),
    ("qqx_to_o8o8_toy_dcolor", "qt qt~ > o8 o8 NP<=2"),
    ("p3r3_to_p3r3_toy_epsilon", "p3 r3 > p3 r3 NP<=2"),
    ("p3r3_to_p3r3_toy_sextet", "p3 r3 > p3 r3 NP<=2"),
];

fn diagram_count(model: &UFOModel, process: &str) -> usize {
    let opts = ParsingOptions::default();
    let pc = parse_proc_card(&format!("generate {process}"), &opts)
        .unwrap_or_else(|e| panic!("parse '{process}': {e}"));
    let sets = generate_from_proc_card(&pc, model)
        .unwrap_or_else(|e| panic!("enumerate '{process}': {e}"));
    sets.iter().map(|s| s.diagrams.len()).sum()
}

/// MadGraph's own diagram count per manifest row, from the committed
/// `validation/madgraph/diagrams.json`.
fn banked_diagram_counts() -> BTreeMap<String, usize> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/diagrams.json");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let json: serde_json::Value = serde_json::from_str(&text).expect("parse diagrams.json");
    json["processes"]
        .as_object()
        .expect("diagrams.json has a `processes` table")
        .iter()
        .map(|(key, entry)| {
            (
                key.clone(),
                entry["total_diagrams"].as_u64().expect("total_diagrams") as usize,
            )
        })
        .collect()
}

/// [`GATED_ROWS`] is exactly the set of toy-model rows the manifest declares
/// `amplitudes = gate`.
///
/// The census below is only a coverage statement if the list it runs on is the
/// gated set: a seventh toy row promoted in the manifest without being added here
/// would stay outside the census, and an op it newly covers would stay on the
/// allowlist.
#[test]
fn gated_rows_are_the_manifests_gated_toy_rows() {
    let modes = common::manifest::category_modes("amplitudes");
    let declared: BTreeSet<String> = common::manifest::row_models()
        .into_iter()
        .filter(|(key, row)| {
            TOY_MODEL_DIRS.contains(&row.dir.as_str())
                && modes.get(key).map(String::as_str) == Some("gate")
        })
        .map(|(key, _)| key)
        .collect();
    let listed: BTreeSet<String> = GATED_ROWS
        .iter()
        .map(|(key, _)| (*key).to_owned())
        .collect();
    assert_eq!(listed, declared);
}

/// Each row's process string is the one its banked amplitude table was generated
/// for.
///
/// Without this the list above would be a transcription: a row whose
/// `mg_amplitude.process` changed would go on being censused under the old
/// process, and the census would report coverage of diagrams MadGraph never
/// evaluated.
#[test]
fn gated_row_processes_are_the_banked_ones() {
    let banked = common::manifest::mg_amplitude_processes();
    for (key, process) in GATED_ROWS {
        let want = banked
            .get(key)
            .unwrap_or_else(|| panic!("[{key}] declares no mg_amplitude process"));
        assert_eq!(process, want, "[{key}]");
    }
}

/// Every gated row's diagram count against MadGraph's own, from the committed
/// `validation/madgraph/diagrams.json`.
///
/// Every toy row's `diagrams` cell is enforced, so unlike the SMEFTsim census
/// there is no informational arm here: a count that moves is a failure.
#[test]
fn gated_row_diagram_counts_match_madgraph() {
    let banked = banked_diagram_counts();
    let modes = common::manifest::category_modes("diagrams");
    for (key, process) in GATED_ROWS {
        assert_eq!(
            modes.get(key).map(String::as_str),
            Some("gate"),
            "[{key}] its diagrams cell stopped being enforced"
        );
        let model = common::model_for_row(key)
            .unwrap_or_else(|e| panic!("[{key}] load the row's model: {e}"));
        let want = banked
            .get(key)
            .unwrap_or_else(|| panic!("[{key}] has no entry in diagrams.json"));
        let ours = diagram_count(&model, process);
        assert_eq!(ours, *want, "[{key}] '{process}'");
        println!("[{key}] '{process}': {ours} diagrams, MadGraph's {want}");
    }
}

/// The gated rows as these models' op-coverage instrument.
///
/// The same two-way census the Standard Model's MG-validated suite and the
/// SMEFTsim ladder run, on the third and last model family in the tree: the ops
/// these rows compile to are ops MadGraph's reference actually exercises,
/// everything else is listed with why, and an op the rows start covering must be
/// struck from the allowlist.
///
/// This is the census that owns the three ops no other model reaches at process
/// level — `SigmaVout`, `SigmaOut` and the bare `Gamma5Amp` — and, with
/// SMEFTsim's `bbx_to_h_identity`, the second one to reach `IdentityAmp`.
#[test]
fn gated_rows_op_census() {
    use vibegraph::helas::eval::op_census::{assert_op_coverage_across, Op};

    // What these two models write is a short list, and it is what the allowlist
    // is read off: `1`, `Identity(2,1)`, `Gamma5(2,1)`, `Gamma(3,2,1)`, the
    // dipole `Sigma(3,-1,2,-2)*P(-1,3)*ProjM(-2,1)`, the tensor contact
    // `Sigma(-1,-2,2,1)*Sigma(-1,-2,4,3)` and its gamma-chain spelling — the
    // colour model's every vertex being the scalar `1`.
    //
    // `Hels` is never emitted at compile time in any model; the helicity
    // expansion derives it. Of the rest:
    //
    // * **No structure here writes `ProjP`**, and the one `ProjM` sits inside the
    //   dipole's chain rather than alone, so `ProjP`, `ProjPAmp` and `ProjMAmp`
    //   — a bare chiral projector closed into the amplitude — are all out.
    // * **No `Epsilon` and no `Metric` in a Lorentz structure**: `EpsilonVout`,
    //   `EpsilonAmp` and `MetricVout` need a vertex that writes one. (`Metric`
    //   itself is covered — it arrives with the vector propagator, not with a
    //   vertex.)
    // * **`Gamma5` appears only as a bilinear of its own**, never inside a chain,
    //   which is `Gamma5Amp`; the in-chain `Gamma5` belongs to SMEFTsim's dipole.
    // * **Every `Gamma(3,2,1)` here is rooted at its vector leg.** All six rows
    //   are `2 -> 2` with both fermions external and the boson internal, so the
    //   FFV vertex only ever produces `GammaVout`; `GammaIout`/`GammaOout` need
    //   the fermion on the internal line. For the same reason the momentum of the
    //   dipole is rooted out (`PMomOut`) and never contracted into the amplitude
    //   (`PMom`).
    // * **The fused chiral-pair forms `FfvVout`/`FfvIout`/`FfvOout`** need a
    //   `Gamma` beside a chiral *pair* on one vertex; the plain gauge coupling
    //   here carries no projector at all.
    // * **The tensor contact saturates its four external legs**, so it has
    //   exactly one rooting — the amplitude sink — and reaches
    //   `FierzOut`/`FierzOutRev`/`FierzPair` but never the fermion-continuing
    //   `MultivectorIout`/`MultivectorOout`, and `SigmaMv` likewise needs the
    //   dipole on an *internal* fermion line. `SigmaVoutRev` and `SigmaOutRev`
    //   need an index order and a crossing no model in reach writes. All five
    //   rest on the hermetic pins `literal_sigma_currents_are_rooting_invariant`,
    //   `tensor_four_fermion_currents_are_rooting_invariant` and the kernel
    //   identities `SigmaVoutRev = -SigmaVout`, `SigmaOutRev = -SigmaOut`.
    // * **No coupling stays a bare pool read.** Every toy coupling scales a
    //   single-reader product alongside a constant coefficient or colour factor,
    //   so the constant collection folds it into one composite pool entry (a
    //   `CoeffRat` leaf) resolved at bind time. The coupling's value is still
    //   read, through that entry; the bare `Coupling` leaf is the SM census's to
    //   cover, and it does.
    const KNOWN_UNCOVERED: [Op; 20] = [
        Op::Hels,
        Op::GammaIout,
        Op::GammaOout,
        Op::ProjP,
        Op::ProjMAmp,
        Op::ProjPAmp,
        Op::MetricVout,
        Op::Gamma5,
        Op::EpsilonVout,
        Op::EpsilonAmp,
        Op::MultivectorIout,
        Op::MultivectorOout,
        Op::SigmaVoutRev,
        Op::SigmaMv,
        Op::SigmaOutRev,
        Op::FfvVout,
        Op::FfvIout,
        Op::FfvOout,
        Op::PMom,
        Op::Coupling,
    ];

    let models: Vec<_> = GATED_ROWS
        .iter()
        .map(|(key, process)| {
            let model = common::model_for_row(key)
                .unwrap_or_else(|e| panic!("[{key}] load the row's model: {e}"));
            (*key, model, [*process])
        })
        .collect();
    let instances: Vec<_> = models
        .iter()
        .map(|(key, model, processes)| (*key, model.as_ref(), &processes[..]))
        .collect();
    assert_op_coverage_across(&instances, &KNOWN_UNCOVERED);
}

/// A copy of `vibegraph_toy_UFO` in which the pseudoscalar Yukawa and the derivative
/// cubic scalar share the `NP` order of their partners, so `V_ltltst`/`V_qtqtst`
/// (`Identity` beside `Gamma5`) and `V_ststst` (`SSS1` beside `SSSD`) each reach the
/// diagram as one vertex carrying both structures.
fn merged_order_toy_model() -> Arc<UFOModel> {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../validation/ufo/vibegraph_toy_UFO");
    let dst = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("vibegraph_toy_merged_orders_UFO");
    std::fs::create_dir_all(&dst).unwrap();
    for entry in std::fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            std::fs::copy(&path, dst.join(path.file_name().unwrap())).unwrap();
        }
    }
    let couplings = std::fs::read_to_string(src.join("couplings.py")).unwrap();
    let merged = couplings
        .replace("order={'NPCP': 1}", "order={'NP': 1}")
        .replace("order={'NPD': 1}", "order={'NP': 1}");
    assert_ne!(
        merged, couplings,
        "the toy couplings no longer carry NPCP/NPD"
    );
    std::fs::write(dst.join("couplings.py"), merged).unwrap();
    UFOModel::load(&dst, Some(&dst.join("restrict_all.dat"))).expect("load the merged toy model")
}

/// Incoming momenta along ±z for masses `m1`, `m2` at `√s`, then the final state
/// from massive RAMBO with fixed uniforms.
fn toy_point(sqrt_s: f64, initial: [f64; 2], fin: &[f64], seed: usize) -> Vec<LorentzVector<f64>> {
    let s = sqrt_s * sqrt_s;
    let [m1, m2] = initial;
    let lambda = (s - (m1 + m2).powi(2)) * (s - (m1 - m2).powi(2));
    let p = lambda.sqrt() / (2.0 * sqrt_s);
    let e1 = (s + m1 * m1 - m2 * m2) / (2.0 * sqrt_s);
    let mut momenta = vec![
        LorentzVector::new(e1, 0.0, 0.0, p),
        LorentzVector::new(sqrt_s - e1, 0.0, 0.0, -p),
    ];
    let u: Vec<f64> = (0..4 * fin.len())
        .map(|k| ((k + 1 + 7 * seed) as f64 * 0.618_033_988_749_894_9).fract())
        .collect();
    momenta.extend(rambo(sqrt_s, fin, &u).momenta);
    momenta
}

/// Vertices that mix structurally different terms under one coupling order compile,
/// and their one merged diagram equals the sum of the two diagrams the order split
/// gives.
///
/// `Identity` beside `Gamma5`, and `SSS1` beside `SSSD`, carry the same diagram-level
/// convention sign (no reversal sign for `1` and `γ⁵`, `C·1ᵀ·C⁻¹ = 1`,
/// `C·γ⁵ᵀ·C⁻¹ = γ⁵`; none for an all-scalar vertex), so the merged vertex has one sign
/// to lift. The split rows are the ones pinned per MadGraph graph in
/// `standalone_jamps`; this comparison carries their signs onto the merged vertex.
/// It is blind to a sign shared by both structures of a merged vertex, which the
/// split rows see.
#[test]
fn mixed_structure_vertices_equal_their_split_sum() {
    let split = UFOModel::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../validation/ufo/vibegraph_toy_UFO"),
        Some(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../validation/ufo/vibegraph_toy_UFO/restrict_all.dat"),
        ),
    )
    .expect("load the toy model");
    let merged = merged_order_toy_model();
    const MLT: f64 = 10.0;
    const MQT: f64 = 50.0;
    const MST: f64 = 200.0;
    struct Case {
        split: &'static str,
        merged: &'static str,
        initial: [f64; 2],
        fin: &'static [f64],
        /// The helicity states of each leg, in process order.
        states: &'static [&'static [i32]],
    }
    let cases = [
        Case {
            split: "lt~ lt > qt qt~ st NP<=3 NPCP<=3 NPD<=3",
            merged: "lt~ lt > qt qt~ st NP<=3",
            initial: [MLT, MLT],
            fin: &[MQT, MQT, MST],
            states: &[&[-1, 1], &[-1, 1], &[-1, 1], &[-1, 1], &[0]],
        },
        Case {
            split: "st lt > st lt NP<=2 NPCP<=2 NPD<=2",
            merged: "st lt > st lt NP<=2",
            initial: [MST, MLT],
            fin: &[MST, MLT],
            states: &[&[0], &[-1, 1], &[0], &[-1, 1]],
        },
        Case {
            split: "lt~ lt > st st NP<=2 NPCP<=2 NPD<=2",
            merged: "lt~ lt > st st NP<=2",
            initial: [MLT, MLT],
            fin: &[MST, MST],
            states: &[&[-1, 1], &[-1, 1], &[0], &[0]],
        },
    ];
    for Case {
        split: split_process,
        merged: merged_process,
        initial,
        fin,
        states,
    } in cases
    {
        let compile = |model: &Arc<UFOModel>, process: &str| {
            let set = common::generate_with(process, model.as_ref()).remove(0);
            let n = set.diagrams.len();
            let ev = AmplitudeEvaluator::compile(&set, model.as_ref())
                .unwrap_or_else(|e| panic!("compile '{process}': {e}"));
            (ev, n)
        };
        let (ev_split, n_split) = compile(&split, split_process);
        let (ev_merged, n_merged) = compile(&merged, merged_process);
        assert!(
            n_merged < n_split,
            "'{merged_process}' has {n_merged} diagrams against the split {n_split}, so no \
             vertex merged its structures"
        );
        let em_split = EvaluatedModel::from_model(Arc::clone(&split));
        let em_merged = EvaluatedModel::from_model(Arc::clone(&merged));
        let b_split = BoundAmplitude::<f64>::bind(&ev_split, &em_split);
        let b_merged = BoundAmplitude::<f64>::bind(&ev_merged, &em_merged);
        let (mut s_split, mut s_merged) = (b_split.scratch_space(), b_merged.scratch_space());

        let mut helicities: Vec<Vec<i32>> = vec![vec![]];
        for leg in states {
            helicities = helicities
                .into_iter()
                .flat_map(|h| {
                    leg.iter().map(move |&s| {
                        let mut h = h.clone();
                        h.push(s);
                        h
                    })
                })
                .collect();
        }
        let (mut worst, mut largest) = (0.0f64, 0.0f64);
        for seed in 0..3 {
            let momenta = toy_point(1000.0, initial, fin, seed);
            for hel in &helicities {
                let a = b_split.eval_amplitude(&momenta, hel, &mut s_split);
                let b = b_merged.eval_amplitude(&momenta, hel, &mut s_merged);
                worst = worst.max((a - b).norm());
                largest = largest.max(a.norm());
            }
        }
        assert!(largest > 0.0, "'{split_process}' evaluates to zero");
        println!(
            "'{merged_process}': {n_merged} diagrams against {n_split} split, worst \
             |A_split - A_merged| / max|A| = {:.3e}",
            worst / largest
        );
        assert!(
            worst <= 1e-12 * largest,
            "'{merged_process}' differs from its split sum: {:.3e} of max|A|",
            worst / largest
        );
    }
}
