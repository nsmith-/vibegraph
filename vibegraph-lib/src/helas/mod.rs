//! # HELAS (HELicity Amplitude Subroutines) implementation in Rust.
//!
//! This module provides a Rust implementation of the HELAS formalism for computing helicity amplitudes in quantum field theory.
//! The main components include:
//! - `repr`: Data structures for Lorentz representations (vectors, spinors, antisymmetric tensors) and related utilities.
//! - `color`: The symbolic SU(3) colour algebra that factors colour out of the amplitude.
//! - `wavefn`: Wavefunction constructors for external legs
//! - `vertex`: Vertex functions (mirroring HELAS vertex subroutines)
//! - `eval`: Evaluation engine for executing compiled HELAS ASTs.
pub mod color;
pub mod eval;
pub mod repr;
pub(crate) mod vertex;
pub mod wavefn;

pub use repr::lorentz::{Bispinor, LorentzVector};
pub use vertex::{iovxxx, jioxxx};
pub use wavefn::{InDiracWf, OutDiracWf};

#[cfg(test)]
mod tests {
    use crate::helas::repr::lorentz::{ComplexVector, SpinorRepr};

    use super::wavefn::VectorWf;
    use super::*;
    use itertools::iproduct;
    use num_complex::Complex64;
    use repr::numbers::Charge::{Antiparticle, Particle};
    use repr::numbers::SpinorHelicity::{Down, Up};

    /// e⁺e⁻ → μ⁺μ⁻ via s-channel photon exchange.
    ///
    /// Kinematics (CoM frame, √s = 2, θ = 90°, all massless):
    ///   e⁻: p = (1, 0, 0,  1)
    ///   e⁺: p = (1, 0, 0, -1)
    ///   μ⁻: p = (1, 1, 0,  0)
    ///   μ⁺: p = (1,-1, 0,  0)
    ///
    /// The electron current is `jioxxx` with unit vector coupling `[1, 1]` and a
    /// massless propagator `1/q²`; the muon current is contracted with it by
    /// `iovxxx` at the same coupling.
    ///
    /// Expected: Σ|M|² = 4  (analytic: 4·e⁴·(1+cos²θ) = 4 at θ = 90°, e = 1).
    /// The four non-zero helicity combinations each give |M|² = 1.
    #[test]
    fn test_ee_to_mumu_spin_sum() {
        // 4-momenta [E, px, py, pz]
        let p_em = LorentzVector::new(1.0, 0.0, 0.0, 1.0); // e⁻
        let p_ep = LorentzVector::new(1.0, 0.0, 0.0, -1.0); // e⁺
        let p_mm = LorentzVector::new(1.0, 1.0, 0.0, 0.0); // μ⁻
        let p_mp = LorentzVector::new(1.0, -1.0, 0.0, 0.0); // μ⁺

        let gc = [1.0_f64, 1.0_f64]; // unit vector coupling, both chiralities

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
            let v = jioxxx(&fo_ep, &fi_em, gc, 0.0, 0.0);

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
        let p_em = LorentzVector::new(1.0, 0.0, 0.0, 1.0);
        let p_ep = LorentzVector::new(1.0, 0.0, 0.0, -1.0);
        let p_mm = LorentzVector::new(1.0, 1.0, 0.0, 0.0);
        let p_mp = LorentzVector::new(1.0, -1.0, 0.0, 0.0);

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

            let v = jioxxx(&fo_ep, &fi_em, gc, 0.0, 0.0);
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

            let v = jioxxx(&fo_ep, &fi_em, gc, 0.0, 0.0);
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
    /// This pins that the muon vector current `ψ̄γ^μψ` built from on-shell
    /// `ixxxxx`/`oxxxxx` spinors is conserved, which needs each spinor to satisfy
    /// its Dirac equation. `q·J` is linear and homogeneous in each spinor, so the
    /// check is blind to every normalisation and phase of the external
    /// wavefunctions: the magnitudes are pinned by the spin-sum and `2E`-norm tests.
    #[test]
    fn test_ward_identity() {
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

            let v_phys = jioxxx(&fo_ep, &fi_em, gc, 0.0, 0.0);

            // Replace ε^μ with the off-shell momentum of the current.
            // For a conserved current (Ward identity) the amplitude must vanish.
            let q = v_phys.momentum; // [E, px, py, pz] of the virtual photon
            let v_ward = VectorWf {
                eps: ComplexVector::from(q),
                momentum: q,
            };

            let amp = iovxxx(&fo_mp, &fi_mm, &v_ward, gc);
            assert!(
                amp.norm() < 1e-12,
                "Ward identity violated for helicities \
                 ({nhel_em},{nhel_ep},{nhel_mm},{nhel_mp}): |M|={:.2e}",
                amp.norm()
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
    /// This exercises `fvixxx` (≡ the `GammaIout` dispatch path), which 2→2 ee→μμ
    /// never tests — there the boson is consumed at the amplitude (`iovxxx`, a dot),
    /// not slashed onto a fermion. A sign swap between `σ·v` and `σ̄·v` in
    /// `SpinorRepr::slash` breaks `p̸ψ = mψ`, and the propagator then survives at
    /// O(1) relative size.
    #[test]
    fn test_ward_identity_offshell_fermion() {
        let g = repr::C::new(0.0, -1.4); // arbitrary nonzero coupling
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

                    // Ket current (fvixxx): q = fi.p − v.p
                    let fi = InDiracWf::from_momentum(p_f, mass, nhel, charge);
                    let out = vertex::fvixxx(&fi, &v, [g.im, g.im], mass, 0.0);
                    let expect = fi.spinor * (-Complex64::I * g);
                    let diff: f64 = (out.spinor - expect).bare_norm_sq().sqrt();
                    let scale: f64 = expect.bare_norm_sq().sqrt().max(1e-30);
                    assert!(
                        diff / scale < 1e-12,
                        "fvixxx off-shell Ward (m={mass}, {nhel}, {charge:?}): \
                         current is not g·ψ (propagator failed to cancel), \
                         rel diff={:.3e}",
                        diff / scale
                    );
                    assert_eq!(out.momentum, fi.momentum - p_gamma);
                }
            }
        }
    }

    /// Bra counterpart of [`test_ward_identity_offshell_fermion`], exercising
    /// `fvoxxx` (≡ the `GammaOout` dispatch path).
    ///
    /// A flow-out fermion is a bra, so the vertex/propagator slash acts to the
    /// *right* (`ψ̄·γ^μ`), not the left (`γ^μ·ψ`): the slash is flow-dependent
    /// (`DiracAdjoint::slash_bispinor`), and flow-out uses the chiral-block-transposed
    /// right action. With ε→q_γ the bra Dirac equation `ψ̄(p̸−m)=0` makes `q̸`
    /// telescope, the propagator `1/(q²−m²)` cancels, and the current collapses to
    /// `+g·ψ̄` (with `q = fo.p + v.p`). A left slash on the bra row does not satisfy
    /// the bra Dirac equation, so the propagator would survive.
    #[test]
    fn test_ward_identity_offshell_fermion_out() {
        let g = repr::C::new(0.0, -0.4);
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
                    let out_o = vertex::fvoxxx(&fo, &v, [g.im, g.im], mass, 0.0);
                    let expect_o = fo.spinor * (Complex64::I * g); // q = fo.p + v.p → +g·ψ̄
                    let diff_o: f64 = (out_o.spinor - expect_o).bare_norm_sq().sqrt();
                    let scale_o: f64 = expect_o.bare_norm_sq().sqrt().max(1e-30);
                    assert!(
                        diff_o / scale_o < 1e-12,
                        "fvoxxx off-shell Ward (m={mass}, {nhel}, {charge:?}): \
                         current is not +g·ψ̄ (propagator failed to cancel), \
                         rel diff={:.3e}",
                        diff_o / scale_o
                    );
                }
            }
        }
    }

    /// Backward-going massless particle: the `sqp0p3 = 0` branch of
    /// `weyl_ixxxxx` (which builds the ket, and through its Dirac conjugate the
    /// bra) is reached when p = [E, 0, 0, −E].
    ///
    /// Verify that every helicity amplitude is finite and that the spin sum keeps
    /// its value. The spin sum is blind to the phase convention of that branch
    /// (the `p_x → 0⁻` side of the limit); the per-helicity MadGraph comparison
    /// is what pins it.
    #[test]
    fn test_backward_direction_massless() {
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

            let v = jioxxx(&fo_ep, &fi_em, gc, 0.0, 0.0);
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
    /// A 1 GeV fermion with E = 3 GeV at 45° in the xz-plane: the HELAS
    /// normalisation fi†·fi = 2E holds for each helicity, for the ket and the bra.
    /// The norm pins the magnitude of the spinor only; it is blind to its phase and
    /// to a sign between its chiral blocks.
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
    /// Tests both helicities of a particle at rest (p = [m, 0, 0, 0]): the ket
    /// and the bra each satisfy fi†·fi = 2m. Like the moving case, this pins the
    /// magnitude only.
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
