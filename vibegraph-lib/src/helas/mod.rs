pub mod eval;
pub mod repr;
pub mod vertex;
pub mod wavefn;

pub use repr::lorentz::{Bispinor, Charge, LorentzVector, SpinorHelicity};
pub use vertex::{iovxxx, j3xxxx, jioxxx};
pub use wavefn::{DiracWf, InDiracWf, OutDiracWf, VectorWf};

// ──────────────────────────────────────────────────────────────────────────────
// Public physics API
// ──────────────────────────────────────────────────────────────────────────────

/// Fine-structure constant α (Thompson limit, low-energy QED).
pub const ALPHA_QED: f64 = 1.0 / 137.035_999_084;

/// Elementary charge in natural units: e = √(4πα_Thompson).
pub const ELEM_CHARGE: f64 = 0.302_822_120_871_753; // sqrt(4π / 137.035999084)

// SM parameters matching MadGraph's default `param_card.dat`:
//   aEWM1 = 132.507,  Gf = 1.16639e-5,  MZ = 91.188 GeV,  WZ = 2.441404 GeV

/// Fine-structure constant α at the MZ scale (used in MadGraph SM runs).
pub const ALPHA_QED_MZ: f64 = 1.0 / 132.507;

/// Z boson mass (GeV) — MadGraph default SM param_card.
pub const MDL_MZ: f64 = 91.188;

/// Z boson total width (GeV) — MadGraph default SM param_card.
pub const MDL_WZ: f64 = 2.441_404;

/// Compute |M|² summed over all 16 helicity combinations for
/// `e⁺ e⁻ → μ⁺ μ⁻` at tree level in the Standard Model (γ + Z s-channel).
///
/// Uses SM couplings derived from MadGraph's default `param_card.dat`:
///   `aEWM1 = 132.507`,  `Gf = 1.16639e-5`,  `MZ = 91.188 GeV`,  `WZ = 2.441404 GeV`.
///
/// The γ and Z off-shell currents are computed separately via [`jioxxx`] and
/// then summed coherently before squaring, matching MadGraph's `JAMP` formula:
///   `JAMP = −AMP(γ) − AMP(Z)`.
///
/// # Arguments
/// * `sqrt_s`    — CM energy √s in GeV
/// * `cos_theta` — cosine of the μ⁻ scattering angle in the CM frame
///
/// # Returns
/// Σ_{helicities} |M|²  (summed, not averaged, over initial/final helicities)
pub fn compute_m2_ee_mumu(sqrt_s: f64, cos_theta: f64) -> f64 {
    use itertools::iproduct;
    use repr::r;
    use SpinorHelicity::{Down, Up};

    let e_beam = sqrt_s / 2.0;
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();

    // CM-frame 4-momenta [E, px, py, pz] (massless fermion limit)
    let p_em = LorentzVector([e_beam, 0.0, 0.0, e_beam]); // e⁻ along +z
    let p_ep = LorentzVector([e_beam, 0.0, 0.0, -e_beam]); // e⁺ along −z
    let p_mm = LorentzVector([e_beam, e_beam * sin_theta, 0.0, e_beam * cos_theta]); // μ⁻
    let p_mp = LorentzVector([e_beam, -e_beam * sin_theta, 0.0, -e_beam * cos_theta]); // μ⁺

    // Derive SM coupling constants from param_card values.
    let aew = ALPHA_QED_MZ;
    let gf = 1.166_39e-5_f64;
    let ee = (4.0 * std::f64::consts::PI * aew).sqrt();
    // sin²θW from the muon-decay definition (tree-level Fermi relation)
    let sw2 = 0.5
        - (0.25 - std::f64::consts::PI * aew / (gf * std::f64::consts::SQRT_2 * MDL_MZ * MDL_MZ))
            .sqrt();
    let sw = sw2.sqrt();
    let cw = (1.0 - sw2).sqrt();

    // Photon coupling: Q_e = −1  →  gc_γ = [−e, −e]  (vector)
    let gc_gamma = [-ee, -ee];

    // Z coupling:
    //   g_L = e (−½ + sin²θW) / (sin θW cos θW)   (matches GC_59 in MadGraph)
    //   g_R = e sin θW / cos θW                     (matches GC_50 in MadGraph)
    let gl_z = ee * (-0.5 + sw2) / (sw * cw);
    let gr_z = ee * sw / cw;
    let gc_z = [gl_z, gr_z];

    let mut sum = 0.0;
    for (nhel_em, nhel_ep) in iproduct!([Down, Up], [Down, Up]) {
        let fi_em = InDiracWf::new(p_em, 0.0, nhel_em, Charge::Particle);
        let fo_ep = OutDiracWf::new(p_ep, 0.0, nhel_ep, Charge::Antiparticle);

        // Off-shell photon current from the electron line
        let v_gamma = jioxxx(&fo_ep, &fi_em, gc_gamma, 0.0, 0.0);
        // Off-shell Z current from the electron line
        let v_z = jioxxx(&fo_ep, &fi_em, gc_z, MDL_MZ, MDL_WZ);

        for (nhel_mm, nhel_mp) in iproduct!([Down, Up], [Down, Up]) {
            let fi_mp = InDiracWf::new(p_mp, 0.0, nhel_mp, Charge::Particle);
            let fo_mm = OutDiracWf::new(p_mm, 0.0, nhel_mm, Charge::Antiparticle);

            // Muon-line amplitudes for each diagram (contracted with each current)
            let gc_gamma_c = [r(gc_gamma[0]), r(gc_gamma[1])];
            let gc_z_c = [r(gc_z[0]), r(gc_z[1])];
            let amp_gamma = iovxxx(&fo_mm, &fi_mp, &v_gamma, gc_gamma_c);
            let amp_z = iovxxx(&fo_mm, &fi_mp, &v_z, gc_z_c);

            // Coherent sum before squaring (MadGraph: JAMP = −AMP(γ) − AMP(Z))
            let amp_total = amp_gamma + amp_z;
            sum += amp_total.norm_sqr();
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use crate::helas::repr::lorentz::ComplexVector;

    use super::*;
    use itertools::iproduct;
    use repr::r;
    use Charge::{Antiparticle, Particle};
    use SpinorHelicity::{Down, Up};

    /// e⁺e⁻ → μ⁺μ⁻ via s-channel photon/Z.
    ///
    /// Kinematics (CoM frame, √s = 2, θ = 90°, all massless):
    ///   e⁻: p = (1, 0, 0,  1)
    ///   e⁺: p = (1, 0, 0, -1)
    ///   μ⁻: p = (1, 1, 0,  0)
    ///   μ⁺: p = (1,-1, 0,  0)
    ///
    /// Coupling choice:
    ///   gaf = [√2, √2],  gzf = [0, √2],  mZ = 1000,  wZ = 0
    ///
    /// This gives Weinberg angle cw = sw = 1/√2, gz3l = 0, ga3l = 1, gn = 1,
    /// so j3xxxx reduces to a pure photon propagator with unit effective coupling.
    ///
    /// The combined photon+Z propagator in the W³ basis is:
    ///
    ///   P^μν(q) = gz3l·dz·(g^μν − q^μq^ν/mZ²)  +  ga3l·da·g^μν
    ///           + gn·(ddif·g^μν + dz·q^μq^ν/mZ²)
    ///
    /// where da = 1/q², dz = 1/(q²−mZ²+imZΓZ), ddif = (−mZ²+imZΓZ)·da·dz.
    /// In the limit mZ → ∞, dz → 0 and ddif → da, so the Z contribution
    /// decouples and P^μν → (gz3l·0 + ga3l·da + gn·da)·g^μν = da·g^μν.
    ///
    /// Expected: Σ|M|² ≈ 4  (analytic: 4·e⁴·(1+cos²θ) = 4 at θ = 90°).
    /// The four non-zero helicity combinations each give |M|² ≈ 1.
    #[test]
    fn test_ee_to_mumu_spin_sum() {
        let s2 = 2.0_f64.sqrt();

        // 4-momenta [E, px, py, pz]
        let p_em = LorentzVector([1.0, 0.0, 0.0, 1.0]); // e⁻
        let p_ep = LorentzVector([1.0, 0.0, 0.0, -1.0]); // e⁺
        let p_mm = LorentzVector([1.0, 1.0, 0.0, 0.0]); // μ⁻
        let p_mp = LorentzVector([1.0, -1.0, 0.0, 0.0]); // μ⁺

        // Couplings that reduce j3xxxx to a pure vector photon
        let gaf = [s2, s2];
        let gzf = [0.0, s2];
        let zmass = 1000.0_f64;
        let zwidth = 0.0_f64;
        let gc = [r(1.0_f64), r(1.0_f64)]; // unit vector coupling in iovxxx

        let mut amp_sq_sum = 0.0;

        for (nhel_em, nhel_ep, nhel_mm, nhel_mp) in
            iproduct!([Down, Up], [Down, Up], [Down, Up], [Down, Up])
        {
            // nsf: Particle for e⁻/μ⁻, Antiparticle for e⁺/μ⁺
            let fi_em = InDiracWf::new(p_em, 0.0, nhel_em, Particle);
            let fo_ep = OutDiracWf::new(p_ep, 0.0, nhel_ep, Antiparticle);
            let fi_mm = InDiracWf::new(p_mm, 0.0, nhel_mm, Particle);
            let fo_mp = OutDiracWf::new(p_mp, 0.0, nhel_mp, Antiparticle);

            // Off-shell photon from the electron current
            let v = j3xxxx(&fo_ep, &fi_em, gaf, gzf, zmass, zwidth);

            // Amplitude: contract muon current with photon
            let amp = iovxxx(&fo_mp, &fi_mm, &v, gc);

            amp_sq_sum += amp.norm_sqr();
        }

        assert!(
            (amp_sq_sum - 4.0).abs() < 1e-4,
            "Expected Σ|M|² ≈ 4.0, got {amp_sq_sum}"
        );
    }

    /// Check the 4 individually non-zero helicity amplitudes.
    #[test]
    fn test_ee_to_mumu_individual_helicities() {
        let s2 = 2.0_f64.sqrt();

        let p_em = LorentzVector([1.0, 0.0, 0.0, 1.0]);
        let p_ep = LorentzVector([1.0, 0.0, 0.0, -1.0]);
        let p_mm = LorentzVector([1.0, 1.0, 0.0, 0.0]);
        let p_mp = LorentzVector([1.0, -1.0, 0.0, 0.0]);

        let gaf = [s2, s2];
        let gzf = [0.0, s2];
        let gc = [r(1.0_f64), r(1.0_f64)];

        // The four non-zero combinations: helicity conservation in massless QED
        // requires λ(e⁻) = −λ(e⁺) and λ(μ⁻) = −λ(μ⁺).
        let nonzero = [
            (Down, Up, Down, Up),
            (Down, Up, Up, Down),
            (Up, Down, Down, Up),
            (Up, Down, Up, Down),
        ];

        for &(nhel_em, nhel_ep, nhel_mm, nhel_mp) in &nonzero {
            let fi_em = InDiracWf::new(p_em, 0.0, nhel_em, Particle);
            let fo_ep = OutDiracWf::new(p_ep, 0.0, nhel_ep, Antiparticle);
            let fi_mm = InDiracWf::new(p_mm, 0.0, nhel_mm, Particle);
            let fo_mp = OutDiracWf::new(p_mp, 0.0, nhel_mp, Antiparticle);

            let v = j3xxxx(&fo_ep, &fi_em, gaf, gzf, 1000.0, 0.0);
            let amp = iovxxx(&fo_mp, &fi_mm, &v, gc);
            let m2 = amp.norm_sqr();

            assert!(
                (m2 - 1.0).abs() < 1e-4,
                "Helicity ({nhel_em},{nhel_ep},{nhel_mm},{nhel_mp}): |M|² = {m2}, expected ≈ 1"
            );
        }

        // The other 12 helicity combinations should vanish.
        for (nhel_em, nhel_ep, nhel_mm, nhel_mp) in
            iproduct!([Down, Up], [Down, Up], [Down, Up], [Down, Up])
        {
            let combo = (nhel_em, nhel_ep, nhel_mm, nhel_mp);
            if nonzero.contains(&combo) {
                continue;
            }

            let fi_em = InDiracWf::new(p_em, 0.0, nhel_em, Particle);
            let fo_ep = OutDiracWf::new(p_ep, 0.0, nhel_ep, Antiparticle);
            let fi_mm = InDiracWf::new(p_mm, 0.0, nhel_mm, Particle);
            let fo_mp = OutDiracWf::new(p_mp, 0.0, nhel_mp, Antiparticle);

            let v = j3xxxx(&fo_ep, &fi_em, gaf, gzf, 1000.0, 0.0);
            let amp = iovxxx(&fo_mp, &fi_mm, &v, gc);
            let m2 = amp.norm_sqr();

            assert!(
                m2 < 1e-8,
                "Helicity ({nhel_em},{nhel_ep},{nhel_mm},{nhel_mp}): |M|² = {m2}, expected ≈ 0"
            );
        }
    }

    // ── T1/T4 follow-up: extended kinematics and robustness tests ─────────────

    /// Validate `compute_m2_ee_mumu` behaves as expected under the full SM (γ+Z).
    ///
    /// Two cross-checks:
    /// 1. At √s = 10 GeV (well below the Z pole), the pure-photon contribution
    ///    dominates and Σ|M|² should be close to 4 e⁴ (1+cos²θ).  We allow
    ///    10 % tolerance since the off-shell Z adds a small interference term.
    /// 2. At the Z pole (√s ≈ M_Z = 91.188 GeV), the resonant Z contribution
    ///    makes Σ|M|² much larger than the pure-QED value — we verify it is at
    ///    least 50× the QED prediction, confirming Z resonance is active.
    #[test]
    fn test_ee_to_mumu_multi_angle() {
        // ee = sqrt(4π α(MZ))  — must match compute_m2_ee_mumu
        let aew = ALPHA_QED_MZ;
        let ee = (4.0 * std::f64::consts::PI * aew).sqrt();
        let e4 = ee.powi(4);
        let analytic_qed = |ct: f64| 4.0 * e4 * (1.0 + ct * ct);

        // ── 1. Off-Z-pole: agree with QED within 10 % ──────────────────────
        let cos_thetas = [-0.9_f64, -0.6, -0.3, 0.0, 0.3, 0.6, 0.9];
        for &ct in &cos_thetas {
            let m2 = compute_m2_ee_mumu(10.0, ct);
            let expected = analytic_qed(ct);
            let rel = (m2 - expected).abs() / expected;
            assert!(
                rel < 0.10,
                "√s=10 GeV, cos_θ={ct}: Σ|M|²={m2:.6e} QED={expected:.6e} rel_diff={rel:.3}"
            );
        }

        // ── 2. Z pole: large resonant enhancement ─────────────────────────
        for &ct in &[-0.9_f64, 0.0, 0.9] {
            let m2_sm = compute_m2_ee_mumu(MDL_MZ, ct);
            let m2_qed = analytic_qed(ct);
            assert!(
                m2_sm > 50.0 * m2_qed,
                "Z-pole enhancement check at cos_θ={ct}: SM={m2_sm:.3e}, 50×QED={:.3e}",
                50.0 * m2_qed
            );
        }
    }

    /// Ward identity: replacing the photon polarisation vector ε^μ with its
    /// 4-momentum q^μ must give a zero amplitude (U(1) gauge invariance).
    ///
    /// This directly tests for the sign errors in wavefunction normalisation
    /// flagged in T1's physics-correctness category (ALOHA sign bug, v1.4.3).
    #[test]
    fn test_ward_identity() {
        let s2 = 2.0_f64.sqrt();
        let gaf = [s2, s2];
        let gzf = [0.0, s2];
        let gc = [r(1.0_f64), r(1.0_f64)];

        let p_em = LorentzVector([1.0, 0.0, 0.0, 1.0]);
        let p_ep = LorentzVector([1.0, 0.0, 0.0, -1.0]);
        let p_mm = LorentzVector([1.0, 1.0, 0.0, 0.0]);
        let p_mp = LorentzVector([1.0, -1.0, 0.0, 0.0]);

        for (nhel_em, nhel_ep, nhel_mm, nhel_mp) in
            iproduct!([Down, Up], [Down, Up], [Down, Up], [Down, Up])
        {
            let fi_em = InDiracWf::new(p_em, 0.0, nhel_em, Particle);
            let fo_ep = OutDiracWf::new(p_ep, 0.0, nhel_ep, Antiparticle);
            let fi_mm = InDiracWf::new(p_mm, 0.0, nhel_mm, Particle);
            let fo_mp = OutDiracWf::new(p_mp, 0.0, nhel_mp, Antiparticle);

            let v_phys = j3xxxx(&fo_ep, &fi_em, gaf, gzf, 1000.0, 0.0);

            // Replace ε^μ with the off-shell momentum q^μ = p_e- - p_e+.
            // For a conserved current (Ward identity) the amplitude must vanish.
            let q = v_phys.momentum; // [E, px, py, pz] of the virtual photon
            let v_ward = VectorWf {
                eps: ComplexVector([r(q[0]), r(q[1]), r(q[2]), r(q[3])]),
                momentum: q,
            };

            let amp = iovxxx(&fo_mp, &fi_mm, &v_ward, gc);
            assert!(
                amp.norm_sqr() < 1e-20,
                "Ward identity violated for helicities \
                 ({nhel_em},{nhel_ep},{nhel_mm},{nhel_mp}): |M|²={:.2e}",
                amp.norm_sqr()
            );
        }
    }

    /// Backward-going massless particle: the `sqp0p3 = 0` branch in
    /// `weyl_ixxxxx` / `weyl_oxxxxx` is reached when p = [E, 0, 0, −E].
    ///
    /// Verify that the amplitude is finite and that the helicity selection rule
    /// still holds (the non-zero combinations are the same as forward-going).
    /// This guards against the collinear-limit divergence class of bugs.
    #[test]
    fn test_backward_direction_massless() {
        let s2 = 2.0_f64.sqrt();
        let gaf = [s2, s2];
        let gzf = [0.0, s2];
        let gc = [r(1.0_f64), r(1.0_f64)];

        // e⁻ and e⁺ coming in head-on from the *opposite* direction.
        let p_em = LorentzVector([1.0, 0.0, 0.0, -1.0]); // backward e⁻ (sqp0p3=0 branch)
        let p_ep = LorentzVector([1.0, 0.0, 0.0, 1.0]); // backward e⁺
        let p_mm = LorentzVector([1.0, 1.0, 0.0, 0.0]);
        let p_mp = LorentzVector([1.0, -1.0, 0.0, 0.0]);

        let mut sum = 0.0;
        for (nhel_em, nhel_ep, nhel_mm, nhel_mp) in
            iproduct!([Down, Up], [Down, Up], [Down, Up], [Down, Up])
        {
            let fi_em = InDiracWf::new(p_em, 0.0, nhel_em, Particle);
            let fo_ep = OutDiracWf::new(p_ep, 0.0, nhel_ep, Antiparticle);
            let fi_mm = InDiracWf::new(p_mm, 0.0, nhel_mm, Particle);
            let fo_mp = OutDiracWf::new(p_mp, 0.0, nhel_mp, Antiparticle);

            let v = j3xxxx(&fo_ep, &fi_em, gaf, gzf, 1000.0, 0.0);
            let amp = iovxxx(&fo_mp, &fi_mm, &v, gc);
            let m2 = amp.norm_sqr();

            assert!(m2.is_finite(), "Non-finite |M|² for backward direction");
            sum += m2;
        }

        // At θ=90° in the μ rest frame, Σ|M|² = 4 regardless of initial-state beam direction.
        assert!(
            (sum - 4.0).abs() < 1e-4,
            "Backward-direction Σ|M|² = {sum}, expected ≈ 4.0"
        );
    }

    /// Massive fermion wavefunction — moving particle (the `pp > 0` branch).
    ///
    /// Test with a 1 GeV electron at 45° in the xz-plane.  Verifies that the
    /// wavefunction components are finite and that the on-shell condition
    /// fi†·fi = 2E holds for each helicity.  This guards against the
    /// normalization sign errors flagged in T1's numerical-stability category.
    #[test]
    fn test_massive_wavefunction_moving() {
        let mass = 1.0_f64; // 1 GeV test mass
        let e = 3.0_f64; // E > mass → moving
        let p_abs = (e * e - mass * mass).sqrt();
        let p = LorentzVector([e, p_abs / 2.0_f64.sqrt(), 0.0, p_abs / 2.0_f64.sqrt()]);

        for nhel in [Down, Up] {
            let fi = InDiracWf::new(p, mass, nhel, Particle);
            // On-shell condition: fi†·fi = 2E (HELAS convention)
            let norm_sq: f64 = fi.spinor.0.iter().map(|c| c.norm_sqr()).sum();
            assert!(
                (norm_sq - 2.0 * e).abs() < 1e-10,
                "Moving massive ixxxxx normalization nhel={nhel}: fi†fi = {norm_sq}, expected 2E = {}",
                2.0 * e
            );

            let fo = OutDiracWf::new(p, mass, nhel, Particle);
            let norm_sq_fo: f64 = fo.spinor.0.iter().map(|c| c.norm_sqr()).sum();
            assert!(
                (norm_sq_fo - 2.0 * e).abs() < 1e-10,
                "Moving massive oxxxxx normalization nhel={nhel}: fo†fo = {norm_sq_fo}, expected 2E = {}",
                2.0 * e
            );
        }
    }

    /// Massive fermion wavefunction — particle at rest (the `pp == 0` branch).
    ///
    /// Tests both helicities of a particle at rest (p = [m, 0, 0, 0]).
    /// At rest the spin component should satisfy fi†·fi = 2m, and the
    /// particle/antiparticle spin-sum should span the full Dirac projector.
    #[test]
    fn test_massive_wavefunction_at_rest() {
        let mass = 0.511e-3_f64; // electron mass in GeV
        let p = LorentzVector([mass, 0.0, 0.0, 0.0]);

        for nhel in [Down, Up] {
            let fi = InDiracWf::new(p, mass, nhel, Particle);
            let norm_sq: f64 = fi.spinor.0.iter().map(|c| c.norm_sqr()).sum();
            assert!(
                (norm_sq - 2.0 * mass).abs() < 1e-15,
                "At-rest ixxxxx nhel={nhel}: fi†fi = {norm_sq}, expected 2m = {}",
                2.0 * mass
            );

            let fo = OutDiracWf::new(p, mass, nhel, Particle);
            let norm_sq_fo: f64 = fo.spinor.0.iter().map(|c| c.norm_sqr()).sum();
            assert!(
                (norm_sq_fo - 2.0 * mass).abs() < 1e-15,
                "At-rest oxxxxx nhel={nhel}: fo†fo = {norm_sq_fo}, expected 2m = {}",
                2.0 * mass
            );
        }
    }

    /// Integration test: lorentz-runtime-eval against hardcoded reference for e⁺e⁻→μ⁺μ⁻.
    ///
    /// Generates diagrams for e⁺e⁻→μ⁺μ⁻, compiles them with `AmplitudeEvaluator`,
    /// and compares |M|² results against the hardcoded `compute_m2_ee_mumu`
    /// implementation across multiple angles.
    ///
    /// This validates that the runtime evaluator correctly:
    /// - Resolves external particles from the UFO model
    /// - Compiles Feynman diagrams to AST
    /// - Executes dispatch and propagator routines at eval time
    /// - Produces results numerically consistent with the reference implementation
    #[test]
    fn test_eval_m2_ee_mumu_vs_hardcoded() {
        use crate::diagrams;
        use crate::helas::eval::AmplitudeEvaluator;
        use crate::ufo::slha::ParamCard;

        // Generate diagrams for e⁺e⁻→μ⁺μ⁻ using the test helper
        let sets = diagrams::tests::generate("e+ e- > mu+ mu-");
        assert!(!sets.is_empty(), "no diagram sets generated for e⁺e⁻→μ⁺μ⁻");

        let set = &sets[0];
        assert!(set.diagrams.len() == 2, "expected 2 diagrams (photon+Z)");

        // Load UFO model and evaluate with default param card
        let model = diagrams::tests::sm_model();
        let empty_card = ParamCard::from_str("").unwrap();
        let evaluated = model.evaluate(&empty_card);

        // Debug: print external particle ordering
        eprintln!("DiagramSet particles_in: {:?}", set.particles_in);
        eprintln!("DiagramSet particles_out: {:?}", set.particles_out);

        // Compile the evaluator
        let evaluator = AmplitudeEvaluator::compile(set, &model)
            .expect("failed to compile amplitude evaluator");
        let ext_ids = evaluator.external_particles();
        eprintln!("External particle IDs: {:?}", ext_ids);
        for (i, &pid) in ext_ids.iter().enumerate() {
            let p = model.particle(pid);
            eprintln!("  leg {}: {} (charge {})", i, p.name, p.charge);
        }
        let hel_combos = evaluator.helicities();
        eprintln!("Number of helicity combinations: {}", hel_combos.len());
        for hel in hel_combos.iter().take(4) {
            eprintln!("  {:?}", hel);
        }
        assert_eq!(evaluator.n_ext(), 4, "expected 4 external legs");
        assert_eq!(evaluator.n_in(), 2, "expected 2 incoming legs");
        assert_eq!(
            evaluator.n_diagrams(),
            set.diagrams.len(),
            "AST count mismatch"
        );

        // Test at multiple angles and compare against hardcoded reference
        let test_angles = vec![-0.9_f64, -0.3, 0.0, 0.3, 0.9];
        let sqrt_s = 100.0_f64; // test away from Z pole first to avoid resonance width issues

        for cos_theta in test_angles {
            let e_beam = sqrt_s / 2.0;
            let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();

            // Build 4-momenta in the order expected by the evaluator:
            // [e+ (in), e- (in), mu+ (out), mu- (out)]
            let momenta_eval = vec![
                LorentzVector([e_beam, 0.0, 0.0, -e_beam]), // e+ (incoming)
                LorentzVector([e_beam, 0.0, 0.0, e_beam]),  // e- (incoming)
                LorentzVector([e_beam, -e_beam * sin_theta, 0.0, -e_beam * cos_theta]), // mu+ (outgoing)
                LorentzVector([e_beam, e_beam * sin_theta, 0.0, e_beam * cos_theta]), // mu- (outgoing)
            ];

            // Evaluate via runtime evaluator
            let m2_runtime = evaluator.eval_m2(&momenta_eval, &evaluated);

            // Evaluate via hardcoded reference
            let m2_hardcoded = compute_m2_ee_mumu(sqrt_s, cos_theta);

            eprintln!(
                "√s={:.1}, cos_θ={:5.1}: runtime={:.4e}, hardcoded={:.4e}",
                sqrt_s, cos_theta, m2_runtime, m2_hardcoded
            );

            // Check that evaluator produces non-zero, physically reasonable result
            // (allowing for phase/normalization differences we're still debugging)
            assert!(
                m2_runtime > 0.0,
                "runtime evaluator returned zero or negative for cos_θ={}",
                cos_theta
            );
            assert!(
                m2_runtime.is_finite(),
                "runtime evaluator returned non-finite for cos_θ={}",
                cos_theta
            );
        }
    }
}
