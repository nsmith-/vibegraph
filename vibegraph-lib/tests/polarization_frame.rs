//! The frame a polarized squared amplitude is evaluated in.
//!
//! A sum over all of a leg's helicities is Lorentz invariant; a sum over some
//! of a massive leg's helicities is not, because a massive particle's helicity
//! is not. MadGraph evaluates a polarized matrix element in the rest frame its
//! run card's `me_frame` names, `[1, 2]` by default: `frame_id` 6, for which
//! MadEvent hands its generated partonic centre-of-mass momenta to the matrix
//! element unboosted (`auto_dsig_v4.inc:134`). The committed amplitude tables
//! bank MadGraph's `MATRIX1` at partonic centre-of-mass points, and every
//! evaluator here is handed such points.
//!
//! What is pinned, per polarized table:
//!
//! - at the banked points this side reproduces MadGraph's value;
//! - at the same points boosted to another frame, the polarized value moves
//!   away from MadGraph's when a massive leg is polarized — so the agreement
//!   above is a statement about the frame, and a caller handing the evaluator
//!   lab-frame momenta would be caught by the amplitude values — while the
//!   same process summed over every helicity does not move (the boost itself
//!   is a Lorentz transformation), and neither does a process polarized only
//!   on massless legs, which is why a run card's `me_frame` is refused only
//!   for a polarized *massive* leg;
//! - `e+ e- > w+{0} w-` and `e+ e- > w+{T} w-` add up to `e+ e- > w+ w-` point
//!   by point, in any frame: an external leg's helicities are summed
//!   incoherently, so the polarized terms of a decomposition have no
//!   interference to lose and their cross sections add exactly.

use std::path::Path;

use serde_json::Value;
use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::LorentzVector;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::sm::{sm_model, SMRestrict};
use vibegraph::ufo::{EvaluatedModel, UFOModel};

type V = LorentzVector<f64>;

struct Table {
    process: String,
    card: ParamCard,
    points: Vec<(Vec<V>, f64)>,
}

fn table(key: &str) -> Table {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../validation/madgraph/amplitudes/{key}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let json: Value = serde_json::from_str(&text).expect("an amplitude table");
    let card_lines: Vec<&str> = json["param_card"]
        .as_array()
        .expect("param card lines")
        .iter()
        .map(|l| l.as_str().unwrap())
        .collect();
    let points = json["points"]
        .as_array()
        .expect("points")
        .iter()
        .map(|p| {
            let momenta = p["momenta"]
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
                    V::new(c[0], c[1], c[2], c[3])
                })
                .collect();
            (momenta, p["m2"].as_f64().unwrap())
        })
        .collect();
    Table {
        process: json["process"].as_str().unwrap().to_owned(),
        card: card_lines
            .join("\n")
            .parse()
            .expect("the banked param card parses"),
        points,
    }
}

/// The unpruned evaluator of a process: helicity pruning drops combinations
/// that vanish only in the partonic centre of mass, so it may not be used in
/// another frame.
fn evaluator(process: &str, model: &UFOModel) -> AmplitudeEvaluator {
    let card = parse_proc_card(&format!("generate {process}"), &ParsingOptions::default())
        .unwrap_or_else(|e| panic!("{process}: {e}"));
    let sets = generate_from_proc_card(&card, model).unwrap_or_else(|e| panic!("{process}: {e}"));
    let set = sets
        .iter()
        .find(|s| !s.diagrams.is_empty())
        .expect("a subprocess");
    AmplitudeEvaluator::compile(set, model).expect("compiles")
}

fn unpolarized(process: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    for c in process.chars() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// A pure boost by velocity `beta`.
fn boost(p: &V, beta: [f64; 3]) -> V {
    let b2: f64 = beta.iter().map(|b| b * b).sum();
    let gamma = 1.0 / (1.0 - b2).sqrt();
    let bp = beta[0] * p.px() + beta[1] * p.py() + beta[2] * p.pz();
    let k = (gamma - 1.0) * bp / b2 + gamma * p.e();
    V::new(
        gamma * (p.e() + bp),
        p.px() + k * beta[0],
        p.py() + k * beta[1],
        p.pz() + k * beta[2],
    )
}

/// A two-body centre-of-mass point moved onto this side's own mass shells:
/// the same beams and outgoing direction, the momentum magnitude recomputed.
///
/// The banked points are on the shells of the param card's printed masses,
/// and the W mass the card prints (80.419) is the derived one rounded to seven
/// digits; both programs evaluate at the unrounded mass, so they agree with
/// each other at those points but the points are 6e-8 off the W shell in
/// `p²`, and an off-shell W's helicity sum is not Lorentz invariant — at
/// ~2e-8 of `|M|²` for `e+ e- > w+ w-` under the boost used here, growing as
/// `beta²`. The invariance control therefore runs on points exactly on shell.
fn on_shell(momenta: &[V], masses: &[f64]) -> Vec<V> {
    assert_eq!(momenta.len(), 4, "a two-body final state");
    let sqrt_s = momenta[0].e() + momenta[1].e();
    let s = sqrt_s * sqrt_s;
    let (m3, m4) = (masses[2], masses[3]);
    let lambda = (s - (m3 + m4).powi(2)) * (s - (m3 - m4).powi(2));
    let p = lambda.sqrt() / (2.0 * sqrt_s);
    let d = momenta[2];
    let norm = (d.px() * d.px() + d.py() * d.py() + d.pz() * d.pz()).sqrt();
    let n = [d.px() / norm, d.py() / norm, d.pz() / norm];
    vec![
        momenta[0],
        momenta[1],
        V::new((m3 * m3 + p * p).sqrt(), p * n[0], p * n[1], p * n[2]),
        V::new((m4 * m4 + p * p).sqrt(), -p * n[0], -p * n[1], -p * n[2]),
    ]
}

const BETA: [f64; 3] = [0.3, -0.2, 0.5];

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / a.abs().max(b.abs()).max(f64::MIN_POSITIVE)
}

/// Worst relative deviations over a table's points: this side against
/// MadGraph in the centre of mass, the polarized value boosted against
/// MadGraph, and the unpolarized value boosted against itself unboosted.
fn frame_deviations(key: &str) -> (f64, f64, f64) {
    let t = table(key);
    let model = sm_model(SMRestrict::Default);
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &t.card);
    let pol = evaluator(&t.process, &model);
    assert!(
        pol.is_polarized(),
        "{key}: '{}' is not polarized",
        t.process
    );
    let sum = evaluator(&unpolarized(&t.process), &model);
    assert!(!sum.is_polarized());
    let (bp, bs) = (
        BoundAmplitude::<f64>::bind(&pol, &evaluated),
        BoundAmplitude::<f64>::bind(&sum, &evaluated),
    );
    let (mut sp, mut ss) = (bp.scratch_space(), bs.scratch_space());
    let masses: Vec<f64> = sum
        .external_particles()
        .iter()
        .map(|&id| evaluated.mass(id))
        .collect();
    let (mut cm, mut moved, mut invariant) = (0.0f64, 0.0f64, 0.0f64);
    for (momenta, mg) in &t.points {
        let boosted: Vec<V> = momenta.iter().map(|p| boost(p, BETA)).collect();
        cm = cm.max(rel(bp.eval_m2(momenta, &mut sp), *mg));
        moved = moved.max(rel(bp.eval_m2(&boosted, &mut sp), *mg));
        let exact = on_shell(momenta, &masses);
        let exact_boosted: Vec<V> = exact.iter().map(|p| boost(p, BETA)).collect();
        invariant = invariant.max(rel(
            bs.eval_m2(&exact_boosted, &mut ss),
            bs.eval_m2(&exact, &mut ss),
        ));
    }
    println!(
        "  {key} ({}): centre of mass {cm:.2e} against MadGraph; boosted by {BETA:?}: \
         polarized {moved:.2e} away from MadGraph, unpolarized {invariant:.2e} from itself",
        t.process
    );
    (cm, moved, invariant)
}

/// A polarized massive leg: MadGraph's value is the centre-of-mass one, and
/// the boosted evaluation disagrees with it.
#[test]
fn a_polarized_massive_leg_is_evaluated_in_the_partonic_centre_of_mass() {
    for key in [
        "ee_to_tlt",
        "ee_to_wp0wmt",
        "ee_to_wp0wm",
        "ee_to_z0h",
        "uux_to_ztg",
    ] {
        let (cm, moved, invariant) = frame_deviations(key);
        assert!(
            cm < 1e-12,
            "{key}: {cm:.2e} from MadGraph at its own points"
        );
        assert!(
            invariant < 1e-11,
            "{key}: the helicity sum moved by {invariant:.2e} under a boost"
        );
        assert!(
            moved > 1e-2,
            "{key}: the polarized value moved by only {moved:.2e} under a boost, so these \
             points cannot tell the frame it was evaluated in"
        );
    }
}

/// A leg polarized only where it is massless: its helicity is Lorentz
/// invariant, and so is the polarized squared amplitude.
#[test]
fn a_polarized_massless_leg_does_not_depend_on_the_frame() {
    let (cm, moved, invariant) = frame_deviations("ee_to_mumu_eml");
    assert!(cm < 1e-12, "{cm:.2e} from MadGraph at its own points");
    assert!(invariant < 1e-11);
    assert!(
        moved < 1e-11,
        "the massless polarized value moved by {moved:.2e}"
    );
}

/// The polarized terms of `e+ e- > w+ w-`, split by the W+ helicity, add up to
/// it point by point in the centre of mass and in a boosted frame alike.
#[test]
fn the_polarized_terms_add_up_to_the_unpolarized_amplitude() {
    let t = table("ee_to_wp0wm");
    let model = sm_model(SMRestrict::Default);
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &t.card);
    let terms =
        ["e+ e- > w+{0} w-", "e+ e- > w+{T} w-", "e+ e- > w+ w-"].map(|p| evaluator(p, &model));
    let bound: Vec<_> = terms
        .iter()
        .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
        .collect();
    let mut scratch: Vec<_> = bound.iter().map(|b| b.scratch_space()).collect();
    let (mut worst, mut longitudinal) = (0.0f64, (f64::INFINITY, 0.0f64));
    for (momenta, _) in &t.points {
        let boosted: Vec<V> = momenta.iter().map(|p| boost(p, BETA)).collect();
        for frame in [momenta, &boosted] {
            let v: Vec<f64> = bound
                .iter()
                .zip(scratch.iter_mut())
                .map(|(b, s)| b.eval_m2(frame, s))
                .collect();
            worst = worst.max(rel(v[0] + v[1], v[2]));
            let share = v[0] / v[2];
            longitudinal = (longitudinal.0.min(share), longitudinal.1.max(share));
        }
    }
    println!(
        "  |M(w+{{0}} w-)|² + |M(w+{{T}} w-)|² against |M(w+ w-)|²: worst {worst:.2e}; the \
         longitudinal share runs {:.3e}..{:.3e}",
        longitudinal.0, longitudinal.1
    );
    assert!(
        worst < 1e-12,
        "the polarized terms miss the sum by {worst:.2e}"
    );
    assert!(
        longitudinal.0 > 0.0 && longitudinal.1 < 1.0,
        "a term of the decomposition is empty, so the sum does not test it"
    );
}
