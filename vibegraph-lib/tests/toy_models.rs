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
//!
//! Beside those, two checks on copies of `vibegraph_toy_UFO` whose coupling orders
//! put two structures on one vertex: the merged vertex equals the sum of its split
//! diagrams, and a merged vertex whose structures disagree on the reversed-bilinear
//! parity is refused when it is compiled.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::LorentzVector;
use vibegraph::phasespace::rambo_massive;
use vibegraph::ufo::{EvaluatedModel, UFOModel, UfoError};

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

/// The authored toy model directory.
fn toy_ufo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../validation/ufo/vibegraph_toy_UFO")
}

/// The pseudoscalar Yukawa moved to the `NP` order of its `Identity` partner, so
/// `V_ltltst`/`V_qtqtst` reach a diagram as one vertex carrying both structures.
const MERGE_NPCP: (&str, &str, &str) = ("couplings.py", "order={'NPCP': 1}", "order={'NP': 1}");

/// The derivative cubic scalar moved to the `NP` order of its `SSS1` partner, so
/// `V_ststst` reaches a diagram as one vertex carrying both structures.
const MERGE_NPD: (&str, &str, &str) = ("couplings.py", "order={'NPD': 1}", "order={'NP': 1}");

/// A copy of `vibegraph_toy_UFO` in a fresh scratch directory, with each
/// `(file, from, to)` replacement applied and each `(file, text)` of `appends`
/// appended to its file, loaded under `restrict_all`. Every `from` must occur in its
/// file.
fn try_edited_toy_model(
    edits: &[(&str, &str, &str)],
    appends: &[(&str, &str)],
) -> Result<Arc<UFOModel>, UfoError> {
    let src = toy_ufo_dir();
    let scratch = tempfile::tempdir().expect("create a scratch directory");
    let dst = scratch.path();
    for entry in std::fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            std::fs::copy(&path, dst.join(path.file_name().unwrap())).unwrap();
        }
    }
    for (file, from, to) in edits {
        let text = std::fs::read_to_string(dst.join(file)).unwrap();
        assert!(
            text.contains(from),
            "the toy model's {file} has no `{from}`"
        );
        std::fs::write(dst.join(file), text.replace(from, to)).unwrap();
    }
    for (file, tail) in appends {
        let text = std::fs::read_to_string(dst.join(file)).unwrap();
        std::fs::write(dst.join(file), format!("{text}\n{tail}\n")).unwrap();
    }
    UFOModel::load(dst, Some(&dst.join("restrict_all.dat")))
}

fn edited_toy_model(edits: &[(&str, &str, &str)]) -> Arc<UFOModel> {
    try_edited_toy_model(edits, &[]).unwrap_or_else(|e| panic!("load the edited toy model: {e}"))
}

const LV_ON_GL_ALT: (&str, &str, &str) = (
    "couplings.py",
    "value='complex(0,1)*gl'",
    "value='complex(0,1)*gl_alt'",
);

/// A parameter defined only under `if configuration.<flag>:` / `else:` is refused
/// at load with an error naming it. The loader cannot know which branch Python
/// takes (`configuration` is a module the UFO imports, and MadGraph supplies no
/// default for it), so it never picks one.
#[test]
fn parameter_defined_only_under_a_conditional_is_refused_by_name() {
    let tail = "\
if configuration.use_alt:
    gl_alt = Parameter(name='gl_alt', nature='internal', type='real', value='1.5', texname='g')
else:
    gl_alt = Parameter(name='gl_alt', nature='internal', type='real', value='2.5', texname='g')
";
    let err = try_edited_toy_model(&[LV_ON_GL_ALT], &[("parameters.py", tail)])
        .err()
        .expect("a conditionally defined parameter must be refused");
    assert!(
        matches!(&err, UfoError::Parameters(_)) && err.to_string().contains("'gl_alt'"),
        "error does not name the parameter: {err}"
    );
}

/// A coupling that names a parameter `parameters.py` never defines is refused at
/// load, naming both, where evaluation would otherwise have no value to read.
#[test]
fn coupling_referencing_an_undefined_parameter_is_refused_by_name() {
    let edit = (
        "couplings.py",
        "value='complex(0,1)*gl'",
        "value='complex(0,1)*gl_missing'",
    );
    let err = try_edited_toy_model(&[edit], &[])
        .err()
        .expect("an undefined parameter must be refused");
    let msg = err.to_string();
    assert!(
        msg.contains("'gl_missing'") && msg.contains("'GC_LV'"),
        "error does not name parameter and coupling: {msg}"
    );
}

/// An internal parameter whose expression names an undefined parameter is refused
/// at load, naming both.
#[test]
fn internal_parameter_referencing_an_undefined_parameter_is_refused_by_name() {
    let tail = "gx = Parameter(name='gx', nature='internal', type='real', \
                value='2*gx_missing', texname='g')";
    let err = try_edited_toy_model(&[], &[("parameters.py", tail)])
        .err()
        .expect("an undefined parameter must be refused");
    let msg = err.to_string();
    assert!(
        msg.contains("'gx_missing'") && msg.contains("'gx'"),
        "error does not name both parameters: {msg}"
    );
}

/// An object a UFO file defines only under `if configuration.<flag>:` is refused
/// at load, naming it and its file, rather than silently left out of the model.
#[test]
fn conditional_particle_and_vertex_definitions_are_refused_by_name() {
    let particle = "\
if configuration.extra:
    extra_part = Particle(pdg_code=9000099, name='xx', antiname='xx', spin=1, color=1,
                          mass=Param.ZERO, width=Param.ZERO, charge=0)
";
    let err = try_edited_toy_model(&[], &[("particles.py", particle)])
        .err()
        .expect("a conditional particle must be refused");
    let msg = err.to_string();
    assert!(
        matches!(
            err,
            UfoError::ConditionalDefinition {
                file: "particles.py",
                ..
            }
        ) && msg.contains("'extra_part'"),
        "{msg}"
    );

    let vertex = "\
if configuration.extra:
    V_extra = Vertex(name='V_extra', particles=[], color=['1'], lorentz=[], couplings={})
";
    let err = try_edited_toy_model(&[], &[("vertices.py", vertex)])
        .err()
        .expect("a conditional vertex must be refused");
    let msg = err.to_string();
    assert!(
        matches!(
            err,
            UfoError::ConditionalDefinition {
                file: "vertices.py",
                ..
            }
        ) && msg.contains("'V_extra'"),
        "{msg}"
    );
}

/// The same refusal for couplings and Lorentz structures.
#[test]
fn conditional_coupling_and_lorentz_definitions_are_refused_by_name() {
    let coupling = "\
if configuration.extra:
    GC_extra = Coupling(name='GC_extra', value='gl', order={'QED': 1})
";
    let err = try_edited_toy_model(&[], &[("couplings.py", coupling)])
        .err()
        .expect("a conditional coupling must be refused");
    assert!(err.to_string().contains("'GC_extra'"), "{err}");

    let lorentz = "\
if configuration.extra:
    L_extra = Lorentz(name='L_extra', spins=[1, 1], structure='Identity(1,2)')
";
    let err = try_edited_toy_model(&[], &[("lorentz.py", lorentz)])
        .err()
        .expect("a conditional Lorentz structure must be refused");
    assert!(err.to_string().contains("'L_extra'"), "{err}");
}

/// `recompute` and `set_alpha_s` return an error naming the parameter the model
/// lacks, rather than panicking.
#[test]
fn recompute_of_an_unknown_parameter_is_an_error() {
    let model = edited_toy_model(&[]);
    let mut ev = EvaluatedModel::from_model(model);
    let err = ev
        .recompute("no_such_parameter", 1.0.into())
        .expect_err("an unknown parameter must be refused");
    assert!(err.to_string().contains("'no_such_parameter'"), "{err}");
    let err = ev.set_alpha_s(0.118).expect_err("the toy model has no aS");
    assert!(err.to_string().contains("'aS'"), "{err}");
}

/// Incoming momenta along ±z for masses `initial` at `√s`, then the final state
/// from massive RAMBO.
fn toy_point(
    sqrt_s: f64,
    initial: [f64; 2],
    fin: &[f64],
    rng: &mut ChaCha8Rng,
) -> Vec<LorentzVector<f64>> {
    let s = sqrt_s * sqrt_s;
    let [m1, m2] = initial;
    let lambda = (s - (m1 + m2).powi(2)) * (s - (m1 - m2).powi(2));
    let p = lambda.sqrt() / (2.0 * sqrt_s);
    let e1 = (s + m1 * m1 - m2 * m2) / (2.0 * sqrt_s);
    let mut momenta = vec![
        LorentzVector::new(e1, 0.0, 0.0, p),
        LorentzVector::new(sqrt_s - e1, 0.0, 0.0, -p),
    ];
    momenta.extend(rambo_massive(sqrt_s, fin, rng));
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
        &toy_ufo_dir(),
        Some(&toy_ufo_dir().join("restrict_all.dat")),
    )
    .expect("load the toy model");
    let merged = edited_toy_model(&[MERGE_NPCP, MERGE_NPD]);
    const MLT: f64 = 10.0;
    const MQT: f64 = 50.0;
    const MST: f64 = 200.0;
    struct Case {
        split: &'static str,
        merged: &'static str,
        initial: [f64; 2],
        fin: &'static [f64],
    }
    let cases = [
        Case {
            split: "lt~ lt > qt qt~ st NP<=3 NPCP<=3 NPD<=3",
            merged: "lt~ lt > qt qt~ st NP<=3",
            initial: [MLT, MLT],
            fin: &[MQT, MQT, MST],
        },
        Case {
            split: "st lt > st lt NP<=2 NPCP<=2 NPD<=2",
            merged: "st lt > st lt NP<=2",
            initial: [MST, MLT],
            fin: &[MST, MLT],
        },
        Case {
            split: "lt~ lt > st st NP<=2 NPCP<=2 NPD<=2",
            merged: "lt~ lt > st st NP<=2",
            initial: [MLT, MLT],
            fin: &[MST, MST],
        },
    ];
    for Case {
        split: split_process,
        merged: merged_process,
        initial,
        fin,
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
        assert_eq!(
            ev_split.helicities(),
            ev_merged.helicities(),
            "'{merged_process}' and its split process sum different helicities"
        );
        let em_split = EvaluatedModel::from_model(Arc::clone(&split));
        let em_merged = EvaluatedModel::from_model(Arc::clone(&merged));
        let b_split = BoundAmplitude::<f64>::bind(&ev_split, &em_split);
        let b_merged = BoundAmplitude::<f64>::bind(&ev_merged, &em_merged);
        let (mut s_split, mut s_merged) = (b_split.scratch_space(), b_merged.scratch_space());

        let mut rng = ChaCha8Rng::seed_from_u64(0x70E5);
        let (mut worst, mut largest) = (0.0f64, 0.0f64);
        for _ in 0..3 {
            let momenta = toy_point(1000.0, initial, fin, &mut rng);
            for hel in ev_split.helicities() {
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

/// A vertex whose structures disagree on the reversed-bilinear parity is refused when
/// a process is compiled, rather than evaluated with one structure's sign.
///
/// The copy puts a derivative `P(-1,3)*Gamma(-1,2,1)` Yukawa in place of `Gamma5`,
/// under `Identity`'s order. Where the vertex's fermion pair sinks into the scalar
/// (`lt~ lt > st`), the gamma term is a fermion→vector sink and takes the reversed
/// parity (`C·γ^μᵀ·C⁻¹ = −γ^μ`) while the `Identity` term does not
/// (`C·1ᵀ·C⁻¹ = 1`), so the vertex has no one parity to lift into the diagram's
/// convention sign. Rooted the other way round, with the scalar on leg 0
/// (`st > lt lt~`), the same vertex compiles: this pins the refusal's path out of
/// `compile`, not which vertices a sign exists for.
#[test]
fn mixed_sign_vertex_is_refused_at_compile() {
    let model = edited_toy_model(&[
        MERGE_NPCP,
        (
            "lorentz.py",
            "structure='Gamma5(2,1)'",
            "structure='P(-1,3)*Gamma(-1,2,1)'",
        ),
    ]);
    let process = "lt~ lt > st NP<=1";
    let set = common::generate_with(process, model.as_ref()).remove(0);
    let err = match AmplitudeEvaluator::compile(&set, model.as_ref()) {
        Ok(_) => panic!("'{process}' compiled a vertex with mixed reversed-bilinear parities"),
        Err(e) => e.to_string(),
    };
    assert!(
        err.starts_with("unsupported vertex: ")
            && err.contains("carry mixed reversed-bilinear parities"),
        "'{process}' was refused for another reason: {err}"
    );
}
