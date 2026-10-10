//! Extended validation: σ(e⁺e⁻→μ⁺μ⁻) via `AmplitudeEvaluator` + VEGAS agrees with
//! the QED analytic formula and the MadGraph5 reference cross section.
//!
//! Gated behind the `extended-validation` cargo feature (slow — ~10⁵ integrand evals).
//!
//!     cargo test -p vibegraph-lib --features extended-validation \
//!                --test validate_vegas

mod common;

use common::{generate, sm_model};

const ALPHA_QED_MZ: f64 = 1.0 / 132.507;
use std::f64::consts::PI;
use std::sync::Arc;
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::LorentzVector;
use vibegraph::phasespace::{self, GEV2_TO_PB};
use vibegraph::ufo::{EvaluatedModel, UFOModel};

/// Compute σ(e⁺e⁻→μ⁺μ⁻) via VEGAS, using `AmplitudeEvaluator::eval_m2` as the integrand.
///
/// Returns `(sigma_GeV2, sigma_err_GeV2)`.  Multiply by `GEV2_TO_PB` for picobarns.
fn sigma_ee_mumu(
    evaluator: &AmplitudeEvaluator,
    evaluated: &EvaluatedModel,
    sqrt_s: f64,
    cos_range: (f64, f64),
    neval: usize,
    niter: usize,
) -> (f64, f64) {
    use rand::SeedableRng;
    use vibegraph::vegas::Vegas;

    let ext = evaluator.external_particles();
    let m_in = evaluated.mass(ext[0]);
    let m_out = evaluated.mass(ext[2]);
    let bound = BoundAmplitude::<f64>::bind(evaluator, evaluated);
    let mut scratch = bound.scratch_space();

    let (cos_min, cos_max) = cos_range;
    let prefactor = phasespace::prefactor2(sqrt_s) * (cos_max - cos_min) / 2.0;

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut vegas = Vegas::new(1, 50, 1.5);

    let result = vegas.integrate(
        |u| {
            let cos_theta = cos_min + u[0] * (cos_max - cos_min);
            let e_beam = sqrt_s / 2.0;
            let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
            let p3_in = (e_beam * e_beam - m_in * m_in).max(0.0).sqrt();
            let p3_out = (e_beam * e_beam - m_out * m_out).max(0.0).sqrt();
            let momenta = vec![
                LorentzVector::new(e_beam, 0.0, 0.0, -p3_in),
                LorentzVector::new(e_beam, 0.0, 0.0, p3_in),
                LorentzVector::new(e_beam, -p3_out * sin_theta, 0.0, -p3_out * cos_theta),
                LorentzVector::new(e_beam, p3_out * sin_theta, 0.0, p3_out * cos_theta),
            ];
            bound.eval_m2(&momenta, &mut scratch)
        },
        neval,
        niter,
        &mut rng,
    );

    (result.integral * prefactor, result.std_dev * prefactor)
}

fn build_evaluator() -> (AmplitudeEvaluator, Arc<UFOModel>) {
    let sets = generate("e+ e- > mu+ mu-");
    let model = sm_model().clone();
    let mut evaluator = AmplitudeEvaluator::compile(&sets[0], &model)
        .expect("failed to compile AmplitudeEvaluator for e+e-→μ+μ-");
    // Production configuration: helicity-filtered (bit-for-bit with unpruned).
    evaluator.prune_zero_helicities(&EvaluatedModel::from_model(model.clone()));
    (evaluator, model)
}

/// How many of its own quoted Monte-Carlo errors a fixed-seed estimate may sit
/// from its target, on top of any stated systematic. A fixed seed makes the run
/// reproducible, not exact: four standard errors is a 6e-5 chance per test of a
/// sampling-order change tripping it.
const MC_ERRORS: f64 = 4.0;

/// Validate `sigma_ee_mumu` (evaluator) vs the QED analytic formula σ = 4πα²/3s.
///
/// At √s = 10 GeV the Standard Model's Z exchange moves the cross section by at
/// most [`common::ee_to_mumu_z_bound`] (about `1.2e-4`), so that is the
/// systematic allowed for, plus [`MC_ERRORS`] of the run's own quoted error. The
/// result pins the evaluator's normalisation — couplings, spin average, the
/// two-body phase space — at that scale: a weight off by 2% misses by a hundred
/// times the bound.
#[test]
fn sigma_qed_limit() {
    let sqrt_s = 10.0_f64;
    let s = sqrt_s * sqrt_s;
    let sigma_analytic = 4.0 * PI * ALPHA_QED_MZ * ALPHA_QED_MZ / (3.0 * s);

    let (evaluator, model) = build_evaluator();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let z_bound = common::ee_to_mumu_z_bound(&evaluated, s);

    let (sigma, err) = sigma_ee_mumu(&evaluator, &evaluated, sqrt_s, (-1.0, 1.0), 50_000, 10);
    let sigma_pb = sigma * GEV2_TO_PB;
    let analytic_pb = sigma_analytic * GEV2_TO_PB;

    let rel = (sigma - sigma_analytic).abs() / sigma_analytic;
    let bound = z_bound + MC_ERRORS * err / sigma_analytic;
    println!(
        "σ(e+e-→μ+μ-) at √s={sqrt_s} GeV: MC = {sigma_pb:.6} ± {:.6} pb, QED = \
         {analytic_pb:.6} pb, |rel| = {rel:.3e} against {bound:.3e} (Z bound {z_bound:.3e})",
        err * GEV2_TO_PB
    );
    assert!(
        rel < bound,
        "σ(e+e-→μ+μ-) at √s={sqrt_s} GeV: MC = {sigma_pb:.6} pb ± {:.6} pb, \
         QED = {analytic_pb:.6} pb, |rel| = {rel:.3e} > {bound:.3e}",
        err * GEV2_TO_PB
    );
}

/// The banked MadGraph cross section of `e+ e- > mu+ mu-`, read from the
/// committed reference with the beam energies it was run at.
fn banked_ee_to_mumu() -> (f64, f64, f64) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/madgraph/sigma_reference.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let reference: serde_json::Value = serde_json::from_str(&text).expect("reference parses");
    let row = &reference["ee_to_mumu"];
    let field = |name: &str| {
        row[name]
            .as_f64()
            .unwrap_or_else(|| panic!("ee_to_mumu has no numeric {name}"))
    };
    (
        field("sigma_pb"),
        field("sigma_err_pb"),
        field("ebeam1") + field("ebeam2"),
    )
}

/// Validate `sigma_ee_mumu` (evaluator) at the Z pole against the banked MadGraph
/// cross section, at the run's own `√s` (`ebeam1 + ebeam2`, 91.2 GeV, not
/// `M_Z`) and acceptance: the run card's `ptl > 10 GeV` and `etal < 2.5`, which
/// for a back-to-back massless pair are one cut on `|cos θ|`.
///
/// The bound is a pull of three on the combined error, MadGraph's `0.042%`
/// dominating it. The `12 MeV` between `M_Z` and the run's `√s` moves σ by about
/// `2e-4`, half the reference's own error: small, and the reason the reference's
/// own energy is read rather than assumed.
#[test]
fn sigma_z_pole() {
    const PTL_CUT: f64 = 10.0;
    const ETAL_CUT: f64 = 2.5;
    const PULL_LIMIT: f64 = 3.0;

    let (mg_pb, mg_err_pb, sqrt_s) = banked_ee_to_mumu();
    let p_cm = sqrt_s / 2.0;
    let cos_max_ptl = (1.0 - (PTL_CUT / p_cm).powi(2)).sqrt();
    let cos_max_eta = ETAL_CUT.tanh();
    let cos_max = cos_max_ptl.min(cos_max_eta);

    let (evaluator, model) = build_evaluator();
    let evaluated = EvaluatedModel::from_model(model.clone());

    let (sigma, err) = sigma_ee_mumu(
        &evaluator,
        &evaluated,
        sqrt_s,
        (-cos_max, cos_max),
        100_000,
        10,
    );
    let sigma_pb = sigma * GEV2_TO_PB;
    let err_pb = err * GEV2_TO_PB;

    let pull = (sigma_pb - mg_pb) / (err_pb * err_pb + mg_err_pb * mg_err_pb).sqrt();
    println!(
        "Z-pole σ at √s = {sqrt_s} GeV: {sigma_pb:.3} ± {err_pb:.3} pb vs MadGraph \
         {mg_pb:.3} ± {mg_err_pb:.3} pb, pull {pull:+.2}, rel {:+.3e}",
        sigma_pb / mg_pb - 1.0
    );
    assert!(
        pull.abs() < PULL_LIMIT,
        "Z-pole σ (AmplitudeEvaluator, MadGraph's cuts) at √s = {sqrt_s} GeV: \
         {sigma_pb:.3} ± {err_pb:.3} pb vs MadGraph {mg_pb:.3} ± {mg_err_pb:.3} pb, \
         cos_max = {cos_max:.4}, pull = {pull:+.2}"
    );
}
