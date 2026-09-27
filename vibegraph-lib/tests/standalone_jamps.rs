//! Per-helicity, per-flow amplitudes against MadGraph standalone output.
//!
//! Each table under `validation/madgraph/standalone/` (written by
//! `validation/madgraph/gen_standalone_jamps.py`) carries MadGraph's JAMP(1..NCOLOR)
//! and colour-summed |M|² for every helicity combination of its NHEL table, at a
//! few RAMBO points, together with the param card it computed them with. This side
//! binds the same card, evaluates the same helicities at the same momenta, and
//! compares:
//!
//! * the per-flow amplitudes, flow by flow in MadGraph's colour-basis order, under
//!   one complex constant fitted over every (point, helicity, flow) entry — the
//!   finest linear object MadGraph exposes that is one-to-one with ours;
//! * the per-helicity colour-summed |M|², which the fitted constant must also
//!   explain (it is `|G|²` there).
//!
//! The fitted constant must have modulus one — the two sides normalise amplitudes
//! alike — but its phase is free: external-wavefunction phase conventions differ
//! between the two programs by a process-dependent `±1`/`±i`.
//!
//! What it cannot see: that global phase (a sign shared by every diagram of the
//! process); the split of a flow into diagrams, so two compensating errors in
//! diagrams that reach the same flows with the same weights at every point; and any
//! helicity or kinematic region the few banked points do not visit.
//!
//! Needs only the committed tables, the interned SM model and the vendored SMEFTsim
//! UFO.

mod common;

use std::path::PathBuf;

use common::{generate_with, sm_model};
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::repr::C;
use vibegraph::helas::LorentzVector;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;

/// Relative deviation allowed per entry, against the largest MadGraph amplitude of
/// the table. Both sides round over the same few dozen diagrams, which sits near
/// 1e-15; a sign on one diagram class is O(1).
const REL_TOL: f64 = 1e-12;

fn table_path(key: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/madgraph/standalone")
        .join(format!("{key}.json"))
}

fn complex(v: &serde_json::Value) -> C<f64> {
    C::new(v[0].as_f64().unwrap(), v[1].as_f64().unwrap())
}

struct Measured {
    g: C<f64>,
    worst_flow: f64,
    worst_m2: f64,
    n_entries: usize,
}

fn measure(key: &str) -> Measured {
    let path = table_path(key);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let table: serde_json::Value = serde_json::from_str(&text).unwrap();
    let model = match table["model"].as_str().unwrap() {
        "sm" => sm_model(),
        row => common::model_for_row(row).unwrap_or_else(|e| panic!("{key}: {e}")),
    };
    let process = table["process"].as_str().unwrap();
    let card: ParamCard = table["param_card"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        .parse()
        .unwrap();
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &card);
    let set = generate_with(process, model.as_ref()).remove(0);
    let evaluator = AmplitudeEvaluator::compile(&set, model.as_ref()).unwrap();
    let n_flows = table["n_flows"].as_u64().unwrap() as usize;
    assert_eq!(evaluator.n_flows(), n_flows, "{key}: NCOLOR");
    let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);
    let mut scratch = bound.scratch_space();

    let helicities: Vec<Vec<i32>> = table["helicities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| {
            h.as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_i64().unwrap() as i32)
                .collect()
        })
        .collect();
    let ours: std::collections::BTreeSet<&Vec<i32>> = evaluator.helicities().iter().collect();
    let theirs: std::collections::BTreeSet<&Vec<i32>> = helicities.iter().collect();
    assert_eq!(
        ours, theirs,
        "{key}: the helicity sums run over different sets"
    );

    // (MadGraph, ours) per flow entry, and per-helicity |M|² pairs.
    let mut flows: Vec<(C<f64>, C<f64>, String)> = Vec::new();
    let mut m2: Vec<(f64, f64, String)> = Vec::new();
    for (pi, point) in table["points"].as_array().unwrap().iter().enumerate() {
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
        for (hel, entry) in helicities.iter().zip(point["helicity"].as_array().unwrap()) {
            let mg: Vec<C<f64>> = entry["jamps"]
                .as_array()
                .unwrap()
                .iter()
                .map(complex)
                .collect();
            let vg = if n_flows == 1 {
                vec![bound.eval_amplitude(&momenta, hel, &mut scratch)]
            } else {
                bound.run_flows(&momenta, hel, &mut scratch)
            };
            for (f, (a, b)) in mg.iter().zip(&vg).enumerate() {
                flows.push((*a, *b, format!("point {pi}, helicities {hel:?}, flow {f}")));
            }
            let cf = evaluator.cf_matrix();
            let mut ours_m2 = 0.0;
            for i in 0..n_flows {
                for j in 0..n_flows {
                    let c =
                        *cf[i * n_flows + j].numer() as f64 / *cf[i * n_flows + j].denom() as f64;
                    ours_m2 += c * (vg[i].conj() * vg[j]).re;
                }
            }
            m2.push((
                entry["m2"].as_f64().unwrap(),
                ours_m2,
                format!("point {pi}, helicities {hel:?}"),
            ));
        }
    }

    let (mut num, mut den, mut scale) = (C::new(0.0, 0.0), 0.0f64, 0.0f64);
    for (mg, vg, _) in &flows {
        num += mg.conj() * vg;
        den += mg.norm_sqr();
        scale = scale.max(mg.norm());
    }
    let g = num / den;
    let mut worst_flow = 0.0f64;
    for (mg, vg, what) in &flows {
        let dev = (vg - g * mg).norm() / scale;
        worst_flow = worst_flow.max(dev);
        assert!(
            dev < REL_TOL,
            "{key}: {what}: vibegraph {vg:?} against G·MadGraph {:?} ({dev:.3e} of the \
             largest amplitude, G = {g:?})",
            g * mg
        );
    }
    let m2_scale = m2.iter().map(|e| e.0.abs()).fold(0.0, f64::max);
    let mut worst_m2 = 0.0f64;
    for (mg, vg, what) in &m2 {
        let dev = (vg - g.norm_sqr() * mg).abs() / m2_scale;
        worst_m2 = worst_m2.max(dev);
        assert!(
            dev < REL_TOL,
            "{key}: {what}: |M|² {vg:e} against |G|²·MadGraph {:e} ({dev:.3e})",
            g.norm_sqr() * mg
        );
    }
    Measured {
        g,
        worst_flow,
        worst_m2,
        n_entries: flows.len(),
    }
}

/// Rows whose comparison is known to disagree with MadGraph, with the finding. A listed
/// row is measured in full and reported; the test fails if it starts agreeing, so the
/// exemption cannot outlive the defect.
const KNOWN_DISAGREEMENT: &[(&str, &str)] = &[(
    "wpwm_to_epem",
    "a W pair at the anchor beside a final-state lepton line: the neutrino exchange \
     comes out with the wrong sign relative to the photon and Z s-channel, while \
     `e+ e- > w+ w-` (the same graphs crossed) agrees",
)];

fn check(key: &str) {
    let known = KNOWN_DISAGREEMENT.iter().find(|(k, _)| *k == key);
    let outcome = std::panic::catch_unwind(|| measure(key));
    match (outcome, known) {
        (Ok(m), None) => {
            println!(
                "{key}: {} flow entries, G = {:?}, worst flow {:.3e}, worst |M|² {:.3e}",
                m.n_entries, m.g, m.worst_flow, m.worst_m2
            );
            assert!(
                (m.g.norm() - 1.0).abs() < REL_TOL,
                "{key}: the flows agree only up to |G| = {}",
                m.g.norm()
            );
        }
        (Err(e), None) => std::panic::resume_unwind(e),
        (Ok(m), Some((_, why))) => panic!(
            "{key} is listed as disagreeing ({why}) but now agrees: G = {:?}, worst flow \
             {:.3e}; remove it from KNOWN_DISAGREEMENT",
            m.g, m.worst_flow
        ),
        (Err(_), Some((_, why))) => println!("{key}: informational, disagrees as listed: {why}"),
    }
}

/// Five gluons: ten of the 25 diagrams join a four-gluon contact to a triple-gluon
/// vertex, the smallest multiplicity where a contact is not the whole diagram.
#[test]
fn gg_to_ggg() {
    check("gg_to_ggg");
}

/// A quark-line anchor with triple-gluon vertices and a four-gluon contact as sources.
#[test]
fn uux_to_ggg() {
    check("uux_to_ggg");
}

/// Quark-gluon scattering with the quark first: a triple-gluon source beside the
/// quark-line anchor.
#[test]
fn ug_to_ug() {
    check("ug_to_ug");
}

/// Colourless four-vector contacts as sources beside a lepton-line anchor.
#[test]
fn ee_to_wpwmz() {
    check("ee_to_wpwmz");
}

/// A colourless four-vector contact as the anchor, beside photon, Z and Higgs exchange.
#[test]
fn wpwm_to_wpwm() {
    check("wpwm_to_wpwm");
}

/// O_HG's gluon-pair–scalar vertex producing the s-channel Higgs current, interfering
/// with QCD.
#[test]
fn ttx_to_gg_chg() {
    check("ttx_to_gg_chg");
}

/// Known disagreement, kept running.
#[test]
fn wpwm_to_epem() {
    check("wpwm_to_epem");
}
