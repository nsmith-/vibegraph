//! The compiled amplitude evaluator against a chain of this crate's own HELAS
//! kernels, hand-composed for e⁺e⁻ → μ⁺μ⁻ and evaluated over a dense
//! (√s, cos θ) grid.
//!
//! MadGraph is not involved: the reference here is
//! [`compute_m2_ee_mumu`], which calls `ixxxxx`/`oxxxxx`/`jioxxx`/`iovxxx`
//! directly and sums the helicities by hand. That separates a diagram-generation,
//! rooting or compilation error — which moves the evaluator and not the hand-built
//! chain — from a kernel error, which moves both together and is what the Fortran77
//! comparison in `validate_helas.rs` is for.
//!
//! Needs nothing but the interned SM model, so it runs on a bare clone.

mod common;

use common::{generate_with, sm_lepton_masses_model};
use itertools::iproduct;
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::LorentzVector;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;

// SM parameters matching MadGraph's default param_card.dat:
//   aEWM1 = 132.507, Gf = 1.16639e-5, MZ = 91.188 GeV, WZ = 2.441404 GeV
const ALPHA_QED_MZ: f64 = 1.0 / 132.507;
const MDL_MZ: f64 = 91.188;
const MDL_WZ: f64 = 2.441_404;
const MDL_ME: f64 = 0.000_511;
const MDL_MMU: f64 = 0.105_658;

fn derive_gammaz_couplings() -> ([f64; 2], [f64; 2]) {
    let aew = ALPHA_QED_MZ;
    let gf = 1.166_39e-5_f64;
    let ee = (4.0 * std::f64::consts::PI * aew).sqrt();
    let sw2 = 0.5
        - (0.25 - std::f64::consts::PI * aew / (gf * std::f64::consts::SQRT_2 * MDL_MZ * MDL_MZ))
            .sqrt();
    let sw = sw2.sqrt();
    let cw = (1.0 - sw2).sqrt();
    let gc_gamma = [-ee, -ee];
    let gl_z = ee * (-0.5 + sw2) / (sw * cw);
    let gr_z = ee * sw / cw;
    ([gc_gamma[0], gc_gamma[1]], [gl_z, gr_z])
}

fn compute_m2_ee_mumu(sqrt_s: f64, cos_theta: f64) -> f64 {
    use itertools::iproduct;
    use vibegraph::helas::repr::numbers::{Charge, SpinorHelicity};
    use vibegraph::helas::{iovxxx, jioxxx};
    use vibegraph::helas::{InDiracWf, LorentzVector, OutDiracWf};
    use SpinorHelicity::{Down, Up};

    let e_beam = sqrt_s / 2.0;
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    let (gc_gamma, gc_z) = derive_gammaz_couplings();
    let p3_e = (e_beam * e_beam - MDL_ME * MDL_ME).max(0.0).sqrt();
    let p3_mu = (e_beam * e_beam - MDL_MMU * MDL_MMU).max(0.0).sqrt();
    let p_ep = LorentzVector::new(e_beam, 0.0, 0.0, -p3_e);
    let p_em = LorentzVector::new(e_beam, 0.0, 0.0, p3_e);
    let p_mp = LorentzVector::new(e_beam, -p3_mu * sin_theta, 0.0, -p3_mu * cos_theta);
    let p_mm = LorentzVector::new(e_beam, p3_mu * sin_theta, 0.0, p3_mu * cos_theta);

    let mut sum = 0.0;
    for (nhel_em, nhel_ep) in iproduct!([Down, Up], [Down, Up]) {
        let fi_em = InDiracWf::from_momentum(p_em, MDL_ME, nhel_em, Charge::Particle);
        let fo_ep = OutDiracWf::from_momentum(p_ep, MDL_ME, nhel_ep, Charge::Antiparticle);
        let v_gamma = jioxxx(&fo_ep, &fi_em, gc_gamma, 0.0, 0.0);
        let v_z = jioxxx(&fo_ep, &fi_em, gc_z, MDL_MZ, MDL_WZ);
        for (nhel_mm, nhel_mp) in iproduct!([Down, Up], [Down, Up]) {
            let fo_mm = OutDiracWf::from_momentum(p_mm, MDL_MMU, nhel_mm, Charge::Particle);
            let fi_mp = InDiracWf::from_momentum(p_mp, MDL_MMU, nhel_mp, Charge::Antiparticle);
            let amp_gamma = iovxxx(&fo_mm, &fi_mp, &v_gamma, gc_gamma);
            let amp_z = iovxxx(&fo_mm, &fi_mp, &v_z, gc_z);
            sum += (amp_gamma + amp_z).norm_sqr();
        }
    }
    sum
}

/// Compare the runtime `AmplitudeEvaluator` against the hardcoded `compute_m2_ee_mumu`
/// reference for e⁺e⁻→μ⁺μ⁻ at multiple angles and CM energies.
#[test]
fn test_eval_m2_ee_mumu_vs_hardcoded() {
    // The hardcoded reference uses physical lepton masses (MDL_ME/MDL_MMU).
    // The default SM restriction (restrict_default) locks Me/MM to zero, so a
    // param card cannot revive them; use the `lepton_masses` restriction, which
    // keeps the lepton masses settable, then supply the physical values here.
    let model = sm_lepton_masses_model();
    let sets = generate_with("e+ e- > mu+ mu-", &model);
    assert!(!sets.is_empty(), "no diagram sets generated for e⁺e⁻→μ⁺μ⁻");

    // The `lepton_masses` restriction also keeps the lepton Yukawas non-zero,
    // so e⁺e⁻→μ⁺μ⁻ gains an s-channel Higgs diagram (γ, Z, H); the Goldstone
    // (G0) diagram is excluded in unitary gauge like MadGraph. The Higgs
    // coupling is ∝ the lepton Yukawa. The hardcoded reference is γ+Z only, so
    // we decouple the scalar by zeroing the lepton Yukawas (YUKAWA 11/13) in
    // the param card while keeping the physical masses (MASS 11/13). The H
    // diagram is still built but evaluates to zero. TODO: once
    // forbidden-propagator filtering (`/ H`) is implemented this is a good
    // test of that syntax — drop the scalar there and the Yukawa override here.
    let set = &sets[0];
    assert_eq!(set.diagrams.len(), 3, "expected 3 diagrams (γ, Z, H)");

    let card = format!(
        "Block MASS\n 11 {}\n 13 {}\nBlock YUKAWA\n 11 0.0\n 13 0.0\n",
        MDL_ME, MDL_MMU
    )
    .parse::<ParamCard>()
    .unwrap();
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &card);

    let evaluator =
        AmplitudeEvaluator::compile(set, &model).expect("failed to compile amplitude evaluator");
    let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);
    let mut scratch = bound.scratch_space();
    assert_eq!(
        evaluator.helicities().len(),
        16,
        "expected 16 helicity combinations"
    );
    assert_eq!(evaluator.n_ext(), 4, "expected 4 external legs");
    assert_eq!(evaluator.n_in(), 2, "expected 2 incoming legs");
    assert_eq!(
        evaluator.n_diagrams(),
        set.diagrams.len(),
        "AST count mismatch"
    );

    // UFO couplings are pure imaginary; the factor of i appears twice per diagram
    // so amplitudes agree up to sign (hence MadGraph's JAMP = -AMP(γ) - AMP(Z)).
    let (gc_gamma, gc_z) = derive_gammaz_couplings();
    let gc_3 = evaluated.coupling(model.coupling_id("GC_3").unwrap());
    assert!(
        (gc_gamma[0] - gc_3.im).abs() < 1e-6,
        "photon coupling mismatch vs GC_3"
    );
    assert!(gc_3.re.abs() < 1e-6, "GC_3 has unexpected real part");
    let gc_59 = evaluated.coupling(model.coupling_id("GC_59").unwrap());
    let gc_50 = evaluated.coupling(model.coupling_id("GC_50").unwrap());
    assert!(
        (gc_z[0] - (gc_59.im + gc_50.im)).abs() < 1e-6,
        "Z left coupling mismatch"
    );
    assert!(
        (gc_z[1] - 2.0 * gc_59.im).abs() < 1e-6,
        "Z right coupling mismatch"
    );

    let me = evaluated.mass(model.particle_id("e-").unwrap());
    let mmu = evaluated.mass(model.particle_id("mu-").unwrap());

    let test_angles = (0..=100).map(|i| -1.0 + 2.0 * (i as f64) / 100.0);
    let test_roots = (1..=200).map(|i| i as f64);

    for (cos_theta, sqrt_s) in iproduct!(test_angles, test_roots) {
        let e_beam = sqrt_s / 2.0;
        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
        let p3_e = (e_beam * e_beam - me * me).max(0.0).sqrt();
        let p3_mu = (e_beam * e_beam - mmu * mmu).max(0.0).sqrt();
        let momenta = vec![
            LorentzVector::new(e_beam, 0.0, 0.0, -p3_e),
            LorentzVector::new(e_beam, 0.0, 0.0, p3_e),
            LorentzVector::new(e_beam, -p3_mu * sin_theta, 0.0, -p3_mu * cos_theta),
            LorentzVector::new(e_beam, p3_mu * sin_theta, 0.0, p3_mu * cos_theta),
        ];

        let m2_runtime = bound.eval_m2(&momenta, &mut scratch);
        let m2_hardcoded = compute_m2_ee_mumu(sqrt_s, cos_theta);

        let rel_diff = (m2_runtime - m2_hardcoded).abs() / m2_hardcoded.max(1e-10);
        assert!(
            rel_diff < 1e-4,
            "Mismatch at √s={:.1}, cos_θ={:.1}: runtime={:.4e}, hardcoded={:.4e}, rel_diff={:.2e}",
            sqrt_s,
            cos_theta,
            m2_runtime,
            m2_hardcoded,
            rel_diff
        );
    }
}
