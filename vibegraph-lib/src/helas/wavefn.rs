//! Wavefunction types for HELAS external legs and off-shell currents.
//!
//! ## Conventions
//!
//! The HELAS conventions (see Appendix A of the HELAS reference) are as follows:
//!
//! ### Spinor wavefunctions
//!
//! For Dirac spinors, we use the Weyl (chiral) basis where the $\gamma^5$ matrix is:
//! $$
//! \gamma^5 = \begin{pmatrix} -\mathbb{1} & 0 \\\\ 0 & \mathbb{1} \end{pmatrix}.
//! $$
//! where $\mathbb{1}$ is the 2×2 identity matrix.  This means the upper two components
//! of a Dirac spinor are left-chiral and the lower two are right-chiral.
//!
//! Correspondingly, the $\gamma^\mu$ matrices are defined as
//! $$
//! \gamma^\mu = \begin{pmatrix} 0 & \sigma^\mu \\\\ \bar{\sigma}^\mu & 0 \end{pmatrix},
//! $$
//! where $\sigma^\mu = (1, \vec{\sigma})$ and $\bar{\sigma}^\mu = (1, -\vec{\sigma})$ are
//! the Pauli matrices.
//!
//! The helicity eigenspinors $\chi_\pm$ for [`SpinorHelicity`] are defined as
//! $$
//! \chi_+(\vec{p}) = \frac{1}{\sqrt{2|\vec{p}|(|\vec{p}| + p_z)}} \begin{pmatrix} |\vec{p}| + p_z \\\\ p_x + i p_y \end{pmatrix}, \quad \chi_-(\vec{p}) = \frac{1}{\sqrt{2|\vec{p}|(|\vec{p}| + p_z)}} \begin{pmatrix} -p_x + i p_y \\\\ |\vec{p}| + p_z \end{pmatrix},
//! $$
//! where $\vec{p} = (p_x, p_y, p_z)$ is the 3-momentum. These eigenspinors satisfy
//! $$
//! \frac{\vec{\sigma} \cdot \vec{p}}{|\vec{p}|} \chi_\pm(\vec{p}) = \pm \chi_\pm(\vec{p}).
//! $$
//! For particles with $|\vec{p}| = -p_z$, the limit $p_y=0$, $p_x \to 0^+$ is taken to define the helicity eigenspinors,
//! which ensures a smooth limit as $|\vec{p}| \to 0$. Specifically, in this limit we have
//! $$
//! \chi_+(\vec{p}) \to \begin{pmatrix} 0 \\\\ 1 \end{pmatrix}, \quad \chi_-(\vec{p}) \to \begin{pmatrix} -1 \\\\ 0 \end{pmatrix}.
//! $$
//!
//! The Dirac bispinors $u$ and $v$ are then constructed from the helicity eigenspinors as follows:
//! $$
//! u(p) = \begin{pmatrix} \omega_\mp(p)\chi_\pm(\vec{p}) \\\\ \omega_\pm(p)\chi_\pm(\vec{p}) \end{pmatrix}, \quad v(p) = \begin{pmatrix} \mp\omega_\pm(p)\chi_\mp(\vec{p}) \\\\ \pm \omega_\mp(p)\chi_\mp(\vec{p}) \end{pmatrix},
//! $$
//!
//! where $\omega_\pm(p) = \sqrt{E \pm |\vec{p}|}$ are the energy-dependent factors that appear in the construction of the spinors.
//! The $u$ spinor will be used for [`Charge::Particle`] and the $v$ spinor for [`Charge::Antiparticle`] in the HELAS convention.
//!
//! ### Vector wavefunctions
//!
//! (TODO)
use crate::helas::repr::lorentz::{
    Bispinor, Bra, ComplexVector, Contravariant, Covariant, DiracAdjoint, Ket, LorentzVector,
    SpinorRepr, Variance,
};
use crate::helas::repr::numbers::{Charge, Chirality, SpinorHelicity};
use crate::helas::repr::{Real, C};
use num_traits::Zero;

// ──────────────────────────────────────────────────────────────────────────────
// Spinor wavefunction
// ──────────────────────────────────────────────────────────────────────────────

/// A Dirac spinor wavefunction together with its (signed) 4-momentum.
///
/// `momentum` stores `p * nsf.sign()`: positive for particles, negative for
/// antiparticles.  This matches the HELAS convention used when building
/// currents and computing the s-channel propagator momentum.
///
/// `spinor` has type `B::Fiber` (= `[C<F>; 4]` for any `B: SpinorRepr<F>`),
/// since [`SpinorRepr<F>`] is a subtrait of [`crate::helas::repr::LorentzRepr<F>`]
/// with `Fiber = [C<F>; 4]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiracWf<F: Real, Adj: DiracAdjoint> {
    pub spinor: Bispinor<F, Adj>,
    /// Signed momentum: particle → +p, antiparticle → −p
    pub momentum: LorentzVector<F, Contravariant>,
}

/// Flowing-IN typed spinor wavefunction.
pub type InDiracWf<F> = DiracWf<F, Ket>;

/// Flowing-OUT typed spinor wavefunction.
pub type OutDiracWf<F> = DiracWf<F, Bra>;

impl<F: Real, Adj: DiracAdjoint> DiracWf<F, Adj> {
    /// Create a spinor wavefunction from a bispinor and momentum.
    pub fn from_momentum(
        p: LorentzVector<F, Contravariant>,
        mass: F,
        nhel: SpinorHelicity,
        nsf: Charge,
    ) -> Self {
        let spinor = Bispinor::from_momentum(p, mass, nhel, nsf);
        Self {
            spinor,
            momentum: match nsf {
                Charge::Particle => p,      // outgoing: +p
                Charge::Antiparticle => -p, // incoming: -p
            },
        }
    }

    /// Create a spinor wavefunction from a bispinor and momentum.
    pub fn from_spinor(
        spinor: Bispinor<F, Adj>,
        momentum: LorentzVector<F, Contravariant>,
    ) -> Self {
        Self { spinor, momentum }
    }

    /// Return the charge (particle vs antiparticle) based on the sign of the energy component of the momentum.
    ///
    /// This relies on the HELAS convention that the momentum stored in the wavefunction is `p * nsf.sign()`,
    /// where `nsf` is the charge sign parameter used when constructing the spinor.
    pub fn charge(&self) -> Charge {
        if self.momentum.e().is_sign_positive() {
            Charge::Particle
        } else {
            Charge::Antiparticle
        }
    }

    /// Flip the adjoint (ket/bra) by taking the Dirac conjugate of the spinor (`u ↔ ū`).
    ///
    /// This is the bra/ket dual of the *same* physical particle, so the stored
    /// (HELAS-signed) momentum is carried through unchanged — matching the
    /// no-flip momentum routing used throughout off-shell-current evaluation.
    pub fn flip_adjoint(self) -> DiracWf<F, Adj::Dual> {
        DiracWf {
            spinor: self.spinor.dualize(),
            momentum: self.momentum,
        }
    }
}

impl<F: Real> InDiracWf<F> {
    /// Convert to a bra wavefunction by taking the Dirac conjugate of the spinor
    pub fn to_outgoing(self) -> OutDiracWf<F> {
        self.flip_adjoint()
    }
}

impl<F: Real> OutDiracWf<F> {
    /// Convert to a ket wavefunction by taking the Dirac conjugate of the spinor
    ///
    /// This is the inverse of [`InDiracWf::to_outgoing`].
    pub fn to_incoming(self) -> InDiracWf<F> {
        self.flip_adjoint()
    }

    pub fn scalar_bilinear(self, other: &InDiracWf<F>, chirality: Chirality) -> C<F> {
        Bispinor::scalar_bilinear(&self.spinor, &other.spinor, chirality)
    }

    pub fn vector_bilinear(
        self,
        other: &InDiracWf<F>,
        chirality: Chirality,
    ) -> ComplexVector<F, Contravariant> {
        Bispinor::vector_bilinear(&self.spinor, &other.spinor, chirality)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Vector (gauge boson) wavefunction
// ──────────────────────────────────────────────────────────────────────────────

/// An off-shell vector wavefunction: 4 complex polarisation components plus
/// the associated 4-momentum.
///
/// Used as both the result of `j3xxxx` and the input to `iovxxx`.
///
/// The polarisation carries its [`Variance`] in the type (`V`, default
/// [`Contravariant`]). External legs and the P-less off-shell currents are
/// contravariant `ε^μ`; index-lowering vertex/propagator kernels produce the
/// covariant `ε_μ`, so the raise/lower at the propagator seam is type-checked
/// rather than hand-coded. The `momentum` is always the physical contravariant
/// 4-momentum `p^μ`.
#[derive(Clone, Copy, Debug)]
pub struct VectorWf<F: Real, V: Variance = Contravariant> {
    /// Polarisation / Lorentz components in HELAS convention, at variance `V`.
    pub eps: ComplexVector<F, V>,
    pub momentum: LorentzVector<F, Contravariant>,
}

impl<F: Real> VectorWf<F, Covariant> {
    /// Raise the polarisation index: `ε^μ = g^{μν} ε_ν` (momentum unchanged).
    #[inline(always)]
    pub fn raise(self) -> VectorWf<F, Contravariant> {
        VectorWf {
            eps: self.eps.raise(),
            momentum: self.momentum,
        }
    }
}

impl<F: Real> VectorWf<F, Contravariant> {
    /// Lower the polarisation index: `ε_μ = g_{μν} ε^ν` (momentum unchanged).
    #[inline(always)]
    pub fn lower(self) -> VectorWf<F, Covariant> {
        VectorWf {
            eps: self.eps.lower(),
            momentum: self.momentum,
        }
    }
}

impl<F: Real> VectorWf<F, Contravariant> {
    /// On-shell polarization vector for a spin-1 external particle.
    ///
    /// # Arguments
    /// - `p`: four-momentum `[E, px, py, pz]` (on-shell: `p² = vmass²`)
    /// - `vmass`: particle mass
    /// - `nhel`: helicity `−1` (left), `0` (longitudinal), `+1` (right)
    /// - `nsv`: flow direction `+1` for outgoing, `−1` for incoming
    ///
    /// Returns a `VectorWf` with polarization components `ε_μ` and signed momentum.
    ///
    /// # Implementation
    /// Converted from ALOHA `vxxxxx.F` (Fortran77 HELAS).
    /// Handles 5 cases: massive at-rest, massive along-z, massive general,
    /// massless along-z, massless general.
    pub fn vxxxxx(p: LorentzVector<F, Contravariant>, vmass: F, nhel: i32, nsv: i32) -> Self {
        let two = F::one() + F::one();
        let sqh = (F::one() / two).sqrt();
        let hel = F::from(nhel).unwrap();
        let nsvahl = F::from(nsv).unwrap() * hel.abs();

        let p0 = p.e();
        let p1 = p.px();
        let p2 = p.py();
        let p3 = p.pz();

        let pt2 = p1 * p1 + p2 * p2;
        let pp3 = (pt2 + p3 * p3).sqrt();
        let pp = p0.min(pp3);
        let pt = pp.min(pt2.sqrt());

        let eps = if vmass != F::zero() {
            // Massive vector case
            let hel0 = F::one() - hel.abs();

            if pp == F::zero() {
                // At rest: use special case
                [
                    C::zero(),
                    C::new(-hel * sqh, F::zero()),
                    C::new(F::zero(), nsvahl * sqh),
                    C::new(hel0, F::zero()),
                ]
            } else {
                // Moving particle
                let emp = p0 / (vmass * pp);

                let eps0 = C::new(hel0 * pp / vmass, F::zero());
                let eps3 = C::new(hel0 * p3 * emp + hel * pt / pp * sqh, F::zero());

                let (eps1, eps2) = if pt != F::zero() {
                    let pzpt = p3 / (pp * pt) * sqh * hel;
                    let e1_re = hel0 * p1 * emp - p1 * pzpt;
                    let e1_im = -nsvahl * p2 / pt * sqh;
                    let e2_re = hel0 * p2 * emp - p2 * pzpt;
                    let e2_im = nsvahl * p1 / pt * sqh;
                    (C::new(e1_re, e1_im), C::new(e2_re, e2_im))
                } else {
                    let sign_p3 = if p3 >= F::zero() { F::one() } else { -F::one() };
                    (
                        C::new(-hel * sqh, F::zero()),
                        C::new(F::zero(), nsvahl * sign_p3 * sqh),
                    )
                };

                [eps0, eps1, eps2, eps3]
            }
        } else {
            // Massless vector case
            let pp_light = p0;
            let pt_light = (p1 * p1 + p2 * p2).sqrt();

            let eps0 = C::new(F::zero(), F::zero());
            let eps3 = C::new(hel * pt_light / pp_light * sqh, F::zero());

            let (eps1, eps2) = if pt_light != F::zero() {
                let pzpt = p3 / (pp_light * pt_light) * sqh * hel;
                let e1_re = -p1 * pzpt;
                let e1_im = -F::from(nsv).unwrap() * p2 / pt_light * sqh;
                let e2_re = -p2 * pzpt;
                let e2_im = F::from(nsv).unwrap() * p1 / pt_light * sqh;
                (C::new(e1_re, e1_im), C::new(e2_re, e2_im))
            } else {
                let sign_p3 = if p3 >= F::zero() { F::one() } else { -F::one() };
                (
                    C::new(-hel * sqh, F::zero()),
                    C::new(F::zero(), F::from(nsv).unwrap() * sign_p3 * sqh),
                )
            };

            [eps0, eps1, eps2, eps3]
        };

        VectorWf {
            eps: ComplexVector::new(eps),
            momentum: match nsv {
                1 => p,   // outgoing: +p
                -1 => -p, // incoming: -p
                _ => panic!("nsv must be ±1"),
            },
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Scalar wavefunction
// ──────────────────────────────────────────────────────────────────────────────

/// External scalar wavefunction: amplitude = 1, momentum stored for routing.
///
/// Used for Higgs bosons, pions, and other spin-0 particles.
/// The wavefunction value is always `1+0i`; momentum is stored to track
/// the internal leg momentum during off-shell vertex computations.
#[derive(Clone, Copy, Debug)]
pub struct ScalarWf<F: Real> {
    /// Scalar amplitude (always 1+0i for external scalars).
    pub value: C<F>,
    /// Signed momentum: particle → +p, antiparticle → −p
    pub momentum: LorentzVector<F, Contravariant>,
}

impl<F: Real> ScalarWf<F> {
    /// On-shell wavefunction for a spin-0 external particle.
    ///
    /// # Arguments
    /// - `p`: four-momentum `[E, px, py, pz]`
    /// - `nss`: flow direction `+1` for outgoing, `−1` for incoming
    ///
    /// Returns a `ScalarWf` with value = 1+0i and signed momentum.
    ///
    /// # Implementation
    /// Converted from ALOHA `sxxxxx.F` (Fortran77 HELAS).
    /// The scalar amplitude is trivial; this mainly stores momentum for routing.
    pub fn sxxxxx(p: LorentzVector<F, Contravariant>, nss: i32) -> Self {
        ScalarWf {
            value: C::new(F::one(), F::zero()),
            momentum: match nss {
                1 => p,   // outgoing: +p
                -1 => -p, // incoming: -p
                _ => panic!("nss must be ±1"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helas::repr::lorentz::VectorRepr;

    #[test]
    fn test_vxxxxx_massless_along_z_positive_helicity() {
        // Massless photon along +z axis with E=1: p = [1, 0, 0, 1]
        let p = LorentzVector::new(1.0, 0.0, 0.0, 1.0);
        let wf = VectorWf::vxxxxx(p, 0.0, 1, 1);

        // Momentum should be p (nsv=+1 for outgoing)
        assert_eq!(wf.momentum.e(), 1.0);
        assert_eq!(wf.momentum.pz(), 1.0);

        // For massless along z, ε_0 = 0
        assert_eq!(wf.eps.component(0).re, 0.0);
        assert_eq!(wf.eps.component(0).im, 0.0);
    }

    #[test]
    fn test_vxxxxx_massive_at_rest() {
        // Massive vector at rest: p = [m, 0, 0, 0]
        let p = LorentzVector::new(1.0, 0.0, 0.0, 0.0);
        let wf = VectorWf::vxxxxx(p, 1.0, 1, 1);

        // Signed momentum: nsv=1 means no sign change
        assert_eq!(wf.momentum.component(0), 1.0);

        // At rest with nhel=1, hel0=0, so ε_1 = -1/√2
        let sqh = 1.0 / 2.0_f64.sqrt();
        assert!((wf.eps.component(1).re + sqh).abs() < 1e-10);
    }

    #[test]
    fn test_vxxxxx_incoming_vs_outgoing() {
        let p = LorentzVector::new(2.0, 1.0, 0.5, 0.2);
        let wf_out = VectorWf::vxxxxx(p, 0.5, 0, 1); // outgoing (nsv=+1)
        let wf_in = VectorWf::vxxxxx(p, 0.5, 0, -1); // incoming (nsv=-1)

        // Momenta should have opposite signs
        assert_eq!(wf_out.momentum.component(0), 2.0);
        assert_eq!(wf_in.momentum.component(0), -2.0);
    }

    #[test]
    fn test_sxxxxx_scalar() {
        let p = LorentzVector::new(1.0, 0.5, 0.3, 0.2);
        let wf = ScalarWf::sxxxxx(p, 1);

        // Scalar amplitude is always 1+0i
        assert_eq!(wf.value.re, 1.0);
        assert_eq!(wf.value.im, 0.0);

        // Momentum should be +p (nsv=1)
        assert_eq!(wf.momentum.component(0), 1.0);
        assert_eq!(wf.momentum.component(1), 0.5);
    }

    #[test]
    fn test_sxxxxx_incoming() {
        let p = LorentzVector::new(2.0, 0.1, 0.2, 0.3);
        let wf = ScalarWf::sxxxxx(p, -1);

        // Momentum should be -p (nsv=-1)
        assert_eq!(wf.momentum.component(0), -2.0);
        assert_eq!(wf.momentum.component(1), -0.1);
    }

    fn generate_spinor_test_cases(
        onshell_only: bool,
    ) -> impl Iterator<
        Item = (
            (LorentzVector<f64, Contravariant>, f64),
            SpinorHelicity,
            Charge,
        ),
    > {
        let p1 = LorentzVector::new(2.0, 0.5, -0.3, 1.2);
        let p2 = LorentzVector::new((3.5_f64).sqrt(), 0.5, -1.0, 1.5);
        assert!(p2.m() < 1e-10, "p2 should be massless for this test");
        let offshell_cases = vec![
            (p1, 0.5), // off-shell massive
            (p1, 0.0), // off-shell massless
        ];
        let onshell_cases = vec![
            (p1, p1.m()), // on-shell massive
            (p2, 0.0),    // on-shell massless
        ];
        let lorentz_cases = if onshell_only {
            onshell_cases
        } else {
            offshell_cases.into_iter().chain(onshell_cases).collect()
        };
        itertools::iproduct!(
            lorentz_cases,
            [SpinorHelicity::Up, SpinorHelicity::Down],
            [Charge::Particle, Charge::Antiparticle]
        )
    }

    /// Test bilinear scalar norm for the spinors
    #[test]
    fn test_bilinear_scalar_norm() {
        for ((p, mass), nhel, nsf) in generate_spinor_test_cases(true) {
            let in_wf = InDiracWf::from_momentum(p, mass, nhel, nsf);
            let scalar_bilinear = in_wf.to_outgoing().scalar_bilinear(&in_wf, Chirality::Both);
            // HELAS convention for the scalar bilinear norm is 2 * mass when on-shell
            let expected = nsf.sign() as f64 * 2.0 * mass;
            assert!(
                (scalar_bilinear.re - expected).abs() < 1e-10,
                "Scalar bilinear norm failed for p={:?}, mass={}, nhel={:?}, nsf={:?}: got {}, expected {}",
                p, mass, nhel, nsf, scalar_bilinear.re, expected
            );
            assert!(
                scalar_bilinear.im.abs() < 1e-10,
                "Scalar bilinear should be real for p={:?}, mass={}, nhel={:?}, nsf={:?}: got imaginary part {}",
                p, mass, nhel, nsf, scalar_bilinear.im
            );

            // Opposite helicity should give zero scalar bilinear when on-shell
            let in_wf_oh = InDiracWf::from_momentum(p, mass, nhel.flip(), nsf);
            let scalar_bilinear_orthogonal = in_wf
                .to_outgoing()
                .scalar_bilinear(&in_wf_oh, Chirality::Both);
            assert!(
                scalar_bilinear_orthogonal.norm() < 1e-10,
                "Scalar bilinear should be orthogonal for opposite helicity: got {}",
                scalar_bilinear_orthogonal
            );
        }
    }

    /// Test the to_outgoing and to_incoming conversions between InDiracWf and OutDiracWf.
    #[test]
    fn test_in_out_conversion() {
        for ((p, mass), nhel, nsf) in generate_spinor_test_cases(false) {
            let in_wf = InDiracWf::from_momentum(p, mass, nhel, nsf);
            let out_wf = OutDiracWf::from_momentum(p, mass, nhel, nsf);
            let in_wf_converted = out_wf.to_incoming();
            let out_wf_converted = in_wf.to_outgoing();

            // The converted wavefunction should match the original
            assert_eq!(in_wf, in_wf_converted);
            assert_eq!(out_wf, out_wf_converted);
        }
    }
}
