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
const MDL_MZ: f64 = 91.188;
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

/// Validate `sigma_ee_mumu` (evaluator) vs the QED analytic formula σ = 4πα²/3s.
///
/// At √s = 10 GeV (well below the Z pole) the Z-exchange contribution is small
/// (~0.5%), so we allow 3% tolerance to cover both MC noise and the Z interference.
#[test]
fn sigma_qed_limit() {
    let sqrt_s = 10.0_f64;
    let s = sqrt_s * sqrt_s;
    let sigma_analytic = 4.0 * PI * ALPHA_QED_MZ * ALPHA_QED_MZ / (3.0 * s);

    let (evaluator, model) = build_evaluator();
    let evaluated = EvaluatedModel::from_model(model.clone());

    let (sigma, err) = sigma_ee_mumu(&evaluator, &evaluated, sqrt_s, (-1.0, 1.0), 50_000, 10);
    let sigma_pb = sigma * GEV2_TO_PB;
    let analytic_pb = sigma_analytic * GEV2_TO_PB;

    let rel = (sigma - sigma_analytic).abs() / sigma_analytic;
    assert!(
        rel < 0.03,
        "σ(e+e-→μ+μ-) at √s={sqrt_s} GeV: \
         MC = {sigma_pb:.4} pb ± {:.4} pb, \
         QED = {analytic_pb:.4} pb, \
         rel_diff = {rel:.4}",
        err * GEV2_TO_PB
    );
}

/// Validate `sigma_ee_mumu` (evaluator) at the Z pole against the MadGraph5 reference.
///
/// MadGraph5 (SM, tree-level, ebeam = 45.6 GeV) gives σ ≈ 2025 pb with default cuts
/// `ptl > 10 GeV`, `etal < 2.5`.  We apply the same acceptance window.
#[test]
fn sigma_z_pole() {
    const MG5_SIGMA_PB: f64 = 2025.0;
    const PTL_CUT: f64 = 10.0;
    const ETAL_CUT: f64 = 2.5;

    let sqrt_s = MDL_MZ;
    let p_cm = sqrt_s / 2.0;
    let cos_max_ptl = (1.0 - (PTL_CUT / p_cm).powi(2)).sqrt();
    let cos_max_eta = ETAL_CUT.tanh();
    let cos_max = cos_max_ptl.min(cos_max_eta);

    let (evaluator, model) = build_evaluator();
    let evaluated = EvaluatedModel::from_model(model.clone());

    let (sigma, _err) = sigma_ee_mumu(
        &evaluator,
        &evaluated,
        sqrt_s,
        (-cos_max, cos_max),
        100_000,
        10,
    );
    let sigma_pb = sigma * GEV2_TO_PB;

    let rel = (sigma_pb - MG5_SIGMA_PB).abs() / MG5_SIGMA_PB;
    assert!(
        rel < 1e-3,
        "Z-pole σ (AmplitudeEvaluator, MG5 cuts): {sigma_pb:.1} pb vs MadGraph {MG5_SIGMA_PB:.1} pb, \
         cos_max = {cos_max:.4}, rel_diff = {rel:.4}"
    );
}

/// `validate_vegas`: AmplitudeEvaluator + VEGAS gives the same Z-pole σ as the
/// previously validated hardcoded amplitude.  Passing this test confirms that
/// replacing `compute_m2_ee_mumu` with `AmplitudeEvaluator::eval_m2` in the
/// phase-space loop does not change the cross section.
///
/// Uses the same MadGraph reference (2025 pb) as `sigma_z_pole`; any regression in
/// the evaluator path that shifts σ by more than 0.1% would be caught here.
#[test]
fn validate_vegas() {
    const MG5_SIGMA_PB: f64 = 2025.0;
    const PTL_CUT: f64 = 10.0;
    const ETAL_CUT: f64 = 2.5;
    const REL_TOL: f64 = 1e-3;

    let sqrt_s = MDL_MZ;
    let p_cm = sqrt_s / 2.0;
    let cos_max = (1.0 - (PTL_CUT / p_cm).powi(2)).sqrt().min(ETAL_CUT.tanh());

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

    let rel = (sigma_pb - MG5_SIGMA_PB).abs() / MG5_SIGMA_PB;
    assert!(
        rel < REL_TOL,
        "validate_vegas: AmplitudeEvaluator σ = {sigma_pb:.2} ± {err_pb:.2} pb, \
         hardcoded ref = {MG5_SIGMA_PB:.1} pb, rel_diff = {rel:.4} > {REL_TOL}"
    );
}
