//! The RAMBO weight normalisation through a flat Monte-Carlo cross section.
//!
//! The deterministic map is pinned by the committed replay fixture
//! (`rambo_oracle.rs`); what a fixture cannot reach is whether the weight's
//! `R_n` and `(2pi)^(4-3n)` factors integrate to the right number. These do,
//! against the QED analytic sigma at a smooth 2-body point and — for the 2 -> 6
//! continuum, where the integrand is heavy-tailed — against the banked MadGraph
//! value, to within an order of magnitude. Banked for cost: about 10^5
//! integrand evaluations. The 2 -> 6 check is `#[ignore]`d and also reads the
//! banked run's parameter card; run it with
//!
//!     cargo test -p vibegraph-lib --features extended-validation \
//!         --test rambo_flat_mc -- --ignored --nocapture

mod common;

use std::path::Path;

use vibegraph::phasespace::rambo;

/// Decisive, low-variance check of the RAMBO weight normalization: σ(e⁺e⁻→μ⁺μ⁻)
/// at √s = 10 GeV via a flat `rambo` n=2 Monte-Carlo, against the QED analytic
/// σ = 4πα²/(3s). Because ee→μμ is angularly smooth (no soft/collinear peak),
/// flat sampling converges fast: the run's own error is about `5e-4` relative.
/// The bound is four of those plus the Z exchange's whole possible effect at
/// this energy ([`common::ee_to_mumu_z_bound`], about `1.2e-4`), so it pins the
/// weight's `R_n` and `(2π)^{4-3n}` factors at about `0.2%` — the precision the
/// heavy-tailed 2→6 check cannot reach.
#[test]
fn flat_mc_two_body_normalization() {
    use std::f64::consts::PI;
    use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
    use vibegraph::helas::LorentzVector;
    use vibegraph::phasespace::{rng::SubStream, GEV2_TO_PB};
    use vibegraph::ufo::EvaluatedModel;

    const ALPHA_QED_MZ: f64 = 1.0 / 132.507;

    let sqrt_s = 10.0f64;
    let s = sqrt_s * sqrt_s;
    let n_out = 2usize;
    let sigma_analytic = 4.0 * PI * ALPHA_QED_MZ * ALPHA_QED_MZ / (3.0 * s) * GEV2_TO_PB;

    let sets = common::generate("e+ e- > mu+ mu-");
    assert!(!sets.is_empty(), "no diagrams for e+ e- > mu+ mu-");
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let evaluator = AmplitudeEvaluator::compile(&sets[0], &model).expect("compile");
    let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);
    let mut scratch = bound.scratch_space();

    // No color for leptons; spin-average 1/4 over the e⁺e⁻ initial state.
    let two_pi_pow = (2.0 * PI).powi(4 - 3 * n_out as i32);
    let prefactor = 1.0 / (2.0 * s) * (1.0 / 4.0) * two_pi_pow * GEV2_TO_PB;

    let masses = [0.0f64; 2];
    let beams = [
        LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
        LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
    ];

    let n_points: usize = 200_000;
    let mut stream = SubStream::from_stream(0xEE22_u64, 0);
    let mut sum = 0.0f64;
    let mut sum_sq = 0.0f64;
    for _ in 0..n_points {
        let u = stream.uniforms::<f64>(4 * n_out);
        let pt = rambo(sqrt_s, &masses, &u);
        let mut momenta = Vec::with_capacity(2 + n_out);
        momenta.extend_from_slice(&beams);
        momenta.extend_from_slice(&pt.momenta);
        let integrand = pt.weight * bound.eval_m2(&momenta, &mut scratch);
        sum += integrand;
        sum_sq += integrand * integrand;
    }
    let mean = sum / n_points as f64;
    let var = (sum_sq / n_points as f64 - mean * mean).max(0.0);
    let sigma = prefactor * mean;
    let sigma_err = prefactor * (var / n_points as f64).sqrt();
    let rel = (sigma - sigma_analytic).abs() / sigma_analytic;

    eprintln!(
        "flat-MC σ(e+e-→μ+μ-, √s=10) = {sigma:.5} ± {sigma_err:.5} pb  \
         (QED 4πα²/3s = {sigma_analytic:.5} pb, rel {rel:.4}, N={n_points})"
    );
    // Four of the run's own errors: a 6e-5 chance per run of a sampling-order
    // change tripping it, and a weight off by 2% misses by ten times the bound.
    let bound = common::ee_to_mumu_z_bound(&evaluated, s) + 4.0 * sigma_err / sigma_analytic;
    assert!(
        rel < bound,
        "flat-MC σ(ee→μμ) {sigma:.5} ± {sigma_err:.5} pb vs QED {sigma_analytic:.5} pb, \
         |rel| {rel:.3e} > {bound:.3e}"
    );
}

#[test]
#[ignore = "slow 2->6 Monte-Carlo; run explicitly with --ignored"]
fn flat_mc_partonic_sigma() {
    use std::f64::consts::PI;
    use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
    use vibegraph::helas::LorentzVector;
    use vibegraph::phasespace::{rng::SubStream, GEV2_TO_PB};
    use vibegraph::ufo::slha::ParamCard;
    use vibegraph::ufo::EvaluatedModel;

    const ROW: &str = "uux_to_ccx_emmm_qcd0";
    // The banked MadGraph partonic σ̂, its process and its beams, from the
    // committed reference.
    let reference_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/sigma_reference.json");
    let reference: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&reference_path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", reference_path.display())),
    )
    .expect("reference parses");
    let field = |name: &str| {
        reference[ROW][name]
            .as_f64()
            .unwrap_or_else(|| panic!("{ROW} has no numeric {name}"))
    };
    let banked_sigma_pb = field("sigma_pb");
    let sqrt_s = field("ebeam1") + field("ebeam2");
    let process = reference[ROW]["process"]
        .as_str()
        .expect("the reference names its process")
        .to_string();
    let process = process.as_str();
    let s = sqrt_s * sqrt_s;
    let n_out = 6usize;

    let sets = common::generate(process);
    assert!(!sets.is_empty(), "no diagrams for {process}");
    let model = common::sm_model();

    // MadGraph's own param card for the run, so the electroweak couplings are the
    // ones the banked σ̂ was computed with.
    let card_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/madgraph/output")
        .join(ROW)
        .join("Cards/param_card.dat");
    let Ok(card_text) = std::fs::read_to_string(&card_path) else {
        vibegraph::validation::require(
            "flat_mc_partonic_sigma",
            "a banked run's parameter card",
            card_path.display(),
        )
    };
    let card = card_text
        .parse::<ParamCard>()
        .unwrap_or_else(|e| panic!("{}: {e:?}", card_path.display()));
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &card);

    let evaluator = AmplitudeEvaluator::compile(&sets[0], &model).expect("compile");
    let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);
    let mut scratch = bound.scratch_space();

    // σ̂ = 1/(2ŝ) · 1/(N_spin·N_color) · (2π)^{4-3n} · ∫ dR_n Σ|M|²,
    // with the RAMBO weight carrying dR_n and Σ|M|² = MATRIX1 (helicity- and
    // color-summed). Averaging: 1/4 spin × 1/9 color for the qq̄ initial state.
    let two_pi_pow = (2.0 * PI).powi(4 - 3 * n_out as i32);
    let prefactor = 1.0 / (2.0 * s) * (1.0 / 36.0) * two_pi_pow * GEV2_TO_PB;

    let masses = [0.0f64; 6];
    let beams = [
        LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
        LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
    ];

    // Flat RAMBO of this EW 2→6 amplitude has high variance (soft/collinear
    // lepton-pair regions), so the estimator converges slowly; the point count
    // is env-tunable (`RAMBO_MC_POINTS`) to trade runtime for the MC error bar.
    let n_points: usize = std::env::var("RAMBO_MC_POINTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30_000);
    let stream_idx: u64 = std::env::var("RAMBO_MC_STREAM")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut stream = SubStream::from_stream(0x5A11B0_u64, stream_idx);
    let mut sum = 0.0f64;
    let mut sum_sq = 0.0f64;
    for _ in 0..n_points {
        let u = stream.uniforms::<f64>(4 * n_out);
        let pt = rambo(sqrt_s, &masses, &u);
        let mut momenta = Vec::with_capacity(2 + n_out);
        momenta.extend_from_slice(&beams);
        momenta.extend_from_slice(&pt.momenta);
        let integrand = pt.weight * bound.eval_m2(&momenta, &mut scratch);
        sum += integrand;
        sum_sq += integrand * integrand;
    }

    let mean = sum / n_points as f64;
    let var = (sum_sq / n_points as f64 - mean * mean).max(0.0);
    let mean_err = (var / n_points as f64).sqrt();

    let sigma = prefactor * mean;
    let sigma_err = prefactor * mean_err;
    let pull = (sigma - banked_sigma_pb) / sigma_err;

    eprintln!(
        "flat-MC σ̂({process}, √ŝ={sqrt_s}) = {sigma:.4e} ± {sigma_err:.2e} pb  \
         (banked {banked_sigma_pb:.4e} pb, pull {pull:.2}σ, N={n_points})"
    );

    // Flat RAMBO of this collinear-peaked EW 6-body amplitude is heavy-tailed:
    // the naive σ/√N understates the true uncertainty, and the estimate scatters
    // over a factor of several between seeds. It also applies none of the banked
    // run's cuts, so the two numbers are not the same integral. This end-to-end
    // check therefore only confirms the weight machinery reproduces the banked
    // value to within the window below: a wrong (2π)^{4-3n} by many powers fails
    // it, one power (a factor 2π) does not. The normalization is pinned at about
    // 0.2% by `flat_mc_two_body_normalization` on the low-variance ee→μμ oracle,
    // and the n-body weight by `rambo_oracle`'s replay fixture.
    let ratio = sigma / banked_sigma_pb;
    assert!(
        sigma > 0.0 && (0.02..50.0).contains(&ratio),
        "flat-MC σ̂ {sigma:.4e} pb is not the same order as banked {banked_sigma_pb:.4e} pb \
         (ratio {ratio:.2}) — suspect a normalization/prefactor error"
    );
}
