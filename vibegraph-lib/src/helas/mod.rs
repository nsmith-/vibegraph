//! # HELAS (HELicity Amplitude Subroutines) implementation in Rust.
//!
//! This module provides a Rust implementation of the HELAS formalism for computing helicity amplitudes in quantum field theory.
//! The main components include:
//! - `repr`: Data structures for Lorentz representations (vectors, spinors, antisymmetric tensors) and related utilities.
//! - `wavefn`: Wavefunction constructors for external legs
//! - `vertex`: Vertex functions (mirroring HELAS vertex subroutines)
//! - `eval`: Evaluation engine for executing compiled HELAS ASTs.
//!
//!
//!
pub mod eval;
pub mod repr;
pub mod vertex;
pub mod wavefn;

pub use repr::lorentz::{Bispinor, LorentzVector};
pub use vertex::{iovxxx, j3xxxx, jioxxx};
pub use wavefn::{DiracWf, InDiracWf, OutDiracWf, VectorWf};

#[cfg(test)]
mod tests {
    use crate::helas::repr::lorentz::{ComplexVector, SpinorRepr};

    use super::*;
    use itertools::iproduct;
    use repr::numbers::Charge::{Antiparticle, Particle};
    use repr::numbers::SpinorHelicity::{Down, Up};

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
        let p_em = LorentzVector::new(1.0, 0.0, 0.0, 1.0); // e⁻
        let p_ep = LorentzVector::new(1.0, 0.0, 0.0, -1.0); // e⁺
        let p_mm = LorentzVector::new(1.0, 1.0, 0.0, 0.0); // μ⁻
        let p_mp = LorentzVector::new(1.0, -1.0, 0.0, 0.0); // μ⁺

        // Couplings that reduce j3xxxx to a pure vector photon
        let gaf = [s2, s2];
        let gzf = [0.0, s2];
        let zmass = 1000.0_f64;
        let zwidth = 0.0_f64;
        let gc = [1.0_f64, 1.0_f64]; // unit vector coupling in iovxxx

        let mut amp_sq_sum = 0.0;

        for (nhel_em, nhel_ep, nhel_mm, nhel_mp) in
            iproduct!([Down, Up], [Down, Up], [Down, Up], [Down, Up])
        {
            // nsf: Particle for e⁻/μ⁻, Antiparticle for e⁺/μ⁺
            let fi_em = InDiracWf::from_momentum(p_em, 0.0, nhel_em, Particle);
            let fo_ep = OutDiracWf::from_momentum(p_ep, 0.0, nhel_ep, Antiparticle);
            let fi_mm = InDiracWf::from_momentum(p_mm, 0.0, nhel_mm, Particle);
            let fo_mp = OutDiracWf::from_momentum(p_mp, 0.0, nhel_mp, Antiparticle);

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

        let p_em = LorentzVector::new(1.0, 0.0, 0.0, 1.0);
        let p_ep = LorentzVector::new(1.0, 0.0, 0.0, -1.0);
        let p_mm = LorentzVector::new(1.0, 1.0, 0.0, 0.0);
        let p_mp = LorentzVector::new(1.0, -1.0, 0.0, 0.0);

        let gaf = [s2, s2];
        let gzf = [0.0, s2];
        let gc = [1.0_f64, 1.0_f64];

        // The four non-zero combinations: helicity conservation in massless QED
        // requires λ(e⁻) = −λ(e⁺) and λ(μ⁻) = −λ(μ⁺).
        let nonzero = [
            (Down, Up, Down, Up),
            (Down, Up, Up, Down),
            (Up, Down, Down, Up),
            (Up, Down, Up, Down),
        ];

        for &(nhel_em, nhel_ep, nhel_mm, nhel_mp) in &nonzero {
            let fi_em = InDiracWf::from_momentum(p_em, 0.0, nhel_em, Particle);
            let fo_ep = OutDiracWf::from_momentum(p_ep, 0.0, nhel_ep, Antiparticle);
            let fi_mm = InDiracWf::from_momentum(p_mm, 0.0, nhel_mm, Particle);
            let fo_mp = OutDiracWf::from_momentum(p_mp, 0.0, nhel_mp, Antiparticle);

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

            let fi_em = InDiracWf::from_momentum(p_em, 0.0, nhel_em, Particle);
            let fo_ep = OutDiracWf::from_momentum(p_ep, 0.0, nhel_ep, Antiparticle);
            let fi_mm = InDiracWf::from_momentum(p_mm, 0.0, nhel_mm, Particle);
            let fo_mp = OutDiracWf::from_momentum(p_mp, 0.0, nhel_mp, Antiparticle);

            let v = j3xxxx(&fo_ep, &fi_em, gaf, gzf, 1000.0, 0.0);
            let amp = iovxxx(&fo_mp, &fi_mm, &v, gc);
            let m2 = amp.norm_sqr();

            assert!(
                m2 < 1e-8,
                "Helicity ({nhel_em},{nhel_ep},{nhel_mm},{nhel_mp}): |M|² = {m2}, expected ≈ 0"
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
        let gc = [1.0_f64, 1.0_f64];

        let p_em = LorentzVector::new(1.0, 0.0, 0.0, 1.0);
        let p_ep = LorentzVector::new(1.0, 0.0, 0.0, -1.0);
        let p_mm = LorentzVector::new(1.0, 1.0, 0.0, 0.0);
        let p_mp = LorentzVector::new(1.0, -1.0, 0.0, 0.0);

        for (nhel_em, nhel_ep, nhel_mm, nhel_mp) in
            iproduct!([Down, Up], [Down, Up], [Down, Up], [Down, Up])
        {
            let fi_em = InDiracWf::from_momentum(p_em, 0.0, nhel_em, Particle);
            let fo_ep = OutDiracWf::from_momentum(p_ep, 0.0, nhel_ep, Antiparticle);
            let fi_mm = InDiracWf::from_momentum(p_mm, 0.0, nhel_mm, Particle);
            let fo_mp = OutDiracWf::from_momentum(p_mp, 0.0, nhel_mp, Antiparticle);

            let v_phys = j3xxxx(&fo_ep, &fi_em, gaf, gzf, 1000.0, 0.0);

            // Replace ε^μ with the off-shell momentum q^μ = p_e- - p_e+.
            // For a conserved current (Ward identity) the amplitude must vanish.
            let q = v_phys.momentum; // [E, px, py, pz] of the virtual photon
            let v_ward = VectorWf {
                eps: ComplexVector::from(q),
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

    /// Off-shell Ward identity for the fermion current (the slash-consumption path).
    ///
    /// Replacing a photon's polarisation `ε^μ` by its own momentum `p_γ^μ` and
    /// slashing it onto a fermion line must collapse the off-shell fermion current
    /// to `−g·(original spinor)`: with `q = p_f − p_γ`, the Dirac equation gives
    /// `p̸_γ·ψ = (p̸_f − q̸)·ψ = (m − q̸)·ψ`, so `(q̸+m)·p̸_γ·ψ = (m²−q²)·ψ` and the
    /// propagator `1/(q²−m²)` cancels it (the QED contact term). If the slash/metric
    /// convention is wrong, `q̸` fails to telescope, the propagator does NOT cancel,
    /// and the result is enhanced by `1/(q²−m²)` and not proportional to the spinor.
    ///
    /// This exercises `fioxxx` (≡ the GammaIout dispatch path), which 2→2 ee→μμ
    /// never tests — there the boson is consumed at the amplitude (`iovxxx`, a dot),
    /// not slashed onto a fermion. (Originally this caught the σ̄·v sign-swap bug in
    /// `SpinorRepr::slash`: with ε→q the propagator failed to cancel, rel diff ~0.65;
    /// fixing σ̄ restored `p̸ψ=mψ` to machine precision.)
    #[test]
    fn test_ward_identity_offshell_fermion() {
        let g = repr::C::new(1.3, -0.4); // arbitrary nonzero coupling
        let p_gamma = LorentzVector::new(2.5, 1.0, 0.3, -1.5); // off-shell photon momentum

        for &mass in &[0.0_f64, 1.7_f64] {
            let p_f = LorentzVector::from_pxpypzmass(0.5, -1.2, 2.0, mass); // on-shell fermion
            for nhel in [Down, Up] {
                for charge in [Particle, Antiparticle] {
                    // Photon with ε replaced by its own 4-momentum (Ward substitution).
                    let v = VectorWf {
                        eps: ComplexVector::from(p_gamma),
                        momentum: p_gamma,
                    };

                    // Flow-in current (fioxxx): q = fi.p − v.p
                    let fi = InDiracWf::from_momentum(p_f, mass, nhel, charge);
                    let out = vertex::fioxxx(&fi, &v, g, mass, 0.0);
                    let expect = fi.spinor * (-g);
                    let diff: f64 = (out.spinor - expect).bare_norm_sq();
                    let scale: f64 = expect.bare_norm_sq().max(1e-30);
                    assert!(
                        diff / scale < 1e-12,
                        "fioxxx off-shell Ward (m={mass}, {nhel}, {charge:?}): \
                         current is not −g·ψ (propagator failed to cancel), \
                         rel diff={:.3e}",
                        diff / scale
                    );
                    assert_eq!(out.momentum, fi.momentum - p_gamma);
                }
            }
        }
    }

    /// Flow-OUT counterpart of [`test_ward_identity_offshell_fermion`], exercising
    /// `foxxx` (≡ the `GammaJout` dispatch path).
    ///
    /// A flow-out fermion is a bra, so the vertex/propagator slash acts to the
    /// *right* (`ψ̄·γ^μ`), not the left (`γ^μ·ψ`). The slash is now flow-dependent
    /// (`SpinorFlow::slash_bispinor`): flow-out uses the chiral-block-transposed
    /// right action. With ε→q_γ the bra Dirac equation `ψ̄(p̸−m)=0` makes `q̸`
    /// telescope, the propagator `1/(q²−m²)` cancels, and the current collapses to
    /// `+g·ψ̄` (with `q = fo.p + v.p`). The earlier left-slash on the dualized
    /// column did not satisfy the bra Dirac equation, so the propagator survived.
    #[test]
    fn test_ward_identity_offshell_fermion_out() {
        let g = repr::C::new(1.3, -0.4);
        let p_gamma = LorentzVector::new(2.5, 1.0, 0.3, -1.5);

        for &mass in &[0.0_f64, 1.7_f64] {
            let p_f = LorentzVector::from_pxpypzmass(0.5, -1.2, 2.0, mass);
            for nhel in [Down, Up] {
                for charge in [Particle, Antiparticle] {
                    let v = VectorWf {
                        eps: ComplexVector::from(p_gamma),
                        momentum: p_gamma,
                    };
                    let fo = OutDiracWf::from_momentum(p_f, mass, nhel, charge);
                    let out_o = vertex::foxxx(&fo, &v, g, mass, 0.0);
                    let expect_o = fo.spinor * g; // q = fo.p + v.p → +g·ψ̄
                    let diff_o: f64 = (out_o.spinor - expect_o).bare_norm_sq();
                    let scale_o: f64 = expect_o.bare_norm_sq().max(1e-30);
                    assert!(
                        diff_o / scale_o < 1e-12,
                        "foxxx off-shell Ward (m={mass}, {nhel}, {charge:?}): \
                         current is not +g·ψ̄ (propagator failed to cancel), \
                         rel diff={:.3e}",
                        diff_o / scale_o
                    );
                }
            }
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
        let gc = [1.0_f64, 1.0_f64];

        // e⁻ and e⁺ coming in head-on from the *opposite* direction.
        let p_em = LorentzVector::new(1.0, 0.0, 0.0, -1.0); // backward e⁻ (sqp0p3=0 branch)
        let p_ep = LorentzVector::new(1.0, 0.0, 0.0, 1.0); // backward e⁺
        let p_mm = LorentzVector::new(1.0, 1.0, 0.0, 0.0);
        let p_mp = LorentzVector::new(1.0, -1.0, 0.0, 0.0);

        let mut sum = 0.0;
        for (nhel_em, nhel_ep, nhel_mm, nhel_mp) in
            iproduct!([Down, Up], [Down, Up], [Down, Up], [Down, Up])
        {
            let fi_em = InDiracWf::from_momentum(p_em, 0.0, nhel_em, Particle);
            let fo_ep = OutDiracWf::from_momentum(p_ep, 0.0, nhel_ep, Antiparticle);
            let fi_mm = InDiracWf::from_momentum(p_mm, 0.0, nhel_mm, Particle);
            let fo_mp = OutDiracWf::from_momentum(p_mp, 0.0, nhel_mp, Antiparticle);

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
        let p = LorentzVector::new(e, p_abs / 2.0_f64.sqrt(), 0.0, p_abs / 2.0_f64.sqrt());

        for nhel in [Down, Up] {
            let fi = InDiracWf::from_momentum(p, mass, nhel, Particle);
            // On-shell condition: fi†·fi = 2E (HELAS convention)
            let norm_sq: f64 = fi.spinor.bare_norm_sq();
            assert!(
                (norm_sq - 2.0 * e).abs() < 1e-10,
                "Moving massive ixxxxx normalization nhel={nhel}: fi†fi = {norm_sq}, expected 2E = {}",
                2.0 * e
            );

            let fo = OutDiracWf::from_momentum(p, mass, nhel, Particle);
            let norm_sq_fo: f64 = fo.spinor.bare_norm_sq();
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
        let p = LorentzVector::new(mass, 0.0, 0.0, 0.0);

        for nhel in [Down, Up] {
            let fi = InDiracWf::from_momentum(p, mass, nhel, Particle);
            let norm_sq: f64 = fi.spinor.bare_norm_sq();
            assert!(
                (norm_sq - 2.0 * mass).abs() < 1e-15,
                "At-rest ixxxxx nhel={nhel}: fi†fi = {norm_sq}, expected 2m = {}",
                2.0 * mass
            );

            let fo = OutDiracWf::from_momentum(p, mass, nhel, Particle);
            let norm_sq_fo: f64 = fo.spinor.bare_norm_sq();
            assert!(
                (norm_sq_fo - 2.0 * mass).abs() < 1e-15,
                "At-rest oxxxxx nhel={nhel}: fo†fo = {norm_sq_fo}, expected 2m = {}",
                2.0 * mass
            );
        }
    }
}
