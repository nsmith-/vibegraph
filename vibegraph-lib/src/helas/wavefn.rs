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
    Bispinor, Charge, Chirality, ComplexVector, LorentzVector, SpinorHelicity, SpinorRepr,
};
use crate::helas::repr::{Real, C};
use std::marker::PhantomData;

// ──────────────────────────────────────────────────────────────────────────────
// Spinor wavefunction
// ──────────────────────────────────────────────────────────────────────────────

/// Marker for flowing-IN spinors (`u`/`v` columns).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowIn;

/// Marker for flowing-OUT spinors (`ū`/`v̄` rows).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowOut;

mod sealed {
    pub trait Sealed {}
}

/// Sealed trait for spinor flow direction, implemented by `FlowIn` and `FlowOut`.
pub trait SpinorFlow: sealed::Sealed {
    type Opposite: SpinorFlow;
    const INCOMING: bool;
}
impl sealed::Sealed for FlowIn {}
impl SpinorFlow for FlowIn {
    type Opposite = FlowOut;
    const INCOMING: bool = true;
}
impl sealed::Sealed for FlowOut {}
impl SpinorFlow for FlowOut {
    type Opposite = FlowIn;
    const INCOMING: bool = false;
}

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
pub struct DiracWf<F: Real, Flow: SpinorFlow = FlowIn> {
    pub spinor: Bispinor<F>,
    /// Signed momentum: particle → +p, antiparticle → −p
    pub momentum: LorentzVector<F>,
    _flow: PhantomData<Flow>,
}

/// Flowing-IN typed spinor wavefunction.
pub type InDiracWf<F> = DiracWf<F, FlowIn>;

/// Flowing-OUT typed spinor wavefunction.
pub type OutDiracWf<F> = DiracWf<F, FlowOut>;

impl<F: Real, Flow: SpinorFlow> DiracWf<F, Flow> {
    #[inline(always)]
    fn from_parts(spinor: Bispinor<F>, momentum: LorentzVector<F>) -> Self {
        Self {
            spinor,
            momentum,
            _flow: PhantomData,
        }
    }

    #[inline(always)]
    fn flip_flow(self) -> DiracWf<F, Flow::Opposite> {
        DiracWf {
            spinor: self.spinor.dirac_adjoint(),
            momentum: self.momentum,
            _flow: PhantomData,
        }
    }

    /// Return the charge (particle vs antiparticle) based on the sign of the energy component of the momentum.
    ///
    /// This relies on the HELAS convention that the momentum stored in the wavefunction is `p * nsf.sign()`,
    /// where `nsf` is the charge sign parameter used when constructing the spinor.
    pub fn charge(&self) -> Charge {
        if self.momentum[0] >= F::zero() {
            Charge::Particle
        } else {
            Charge::Antiparticle
        }
    }

    /// Return a scalar bilinear of the form `f̄ Γ f` where `Γ` is a chiral structure (Identity, P_L, P_R).
    pub fn scalar_bilinear(
        &self,
        other: &DiracWf<F, Flow::Opposite>,
        chirality: Chirality,
    ) -> C<F> {
        match Flow::INCOMING {
            true => SpinorRepr::scalar_bilinear(&other.spinor, &self.spinor, chirality),
            false => SpinorRepr::scalar_bilinear(&self.spinor, &other.spinor, chirality),
        }
    }

    /// Return a vector bilinear of the form `f̄ γ^μ Γ f` where `Γ` is a chiral structure (Identity, P_L, P_R).
    pub fn vector_bilinear(
        &self,
        other: &DiracWf<F, Flow::Opposite>,
        chirality: Chirality,
    ) -> ComplexVector<F> {
        match Flow::INCOMING {
            true => SpinorRepr::vector_bilinear(&other.spinor, &self.spinor, chirality),
            false => SpinorRepr::vector_bilinear(&self.spinor, &other.spinor, chirality),
        }
    }
}

impl<F: Real> InDiracWf<F> {
    /// Construct a flowing-IN wavefunction.
    pub fn new(p: LorentzVector<F>, mass: F, nhel: SpinorHelicity, nsf: Charge) -> Self {
        let spinor = Bispinor::ixxxxx(p, mass, nhel, nsf);
        Self::from_parts(
            spinor,
            match nsf {
                Charge::Particle => p,      // outgoing: +p
                Charge::Antiparticle => -p, // incoming: -p
            },
        )
    }

    /// Construct an off-shell flowing-IN fermion from an arbitrary spinor and momentum.
    ///
    /// Used by `foxxx` and similar off-shell vertex routines.
    pub fn from_spinor(spinor: Bispinor<F>, momentum: LorentzVector<F>) -> Self {
        Self::from_parts(spinor, momentum)
    }

    /// Convert to a flowing-OUT wavefunction by taking the Dirac conjugate of the spinor
    pub fn to_outgoing(self) -> OutDiracWf<F> {
        self.flip_flow()
    }
}

impl<F: Real> OutDiracWf<F> {
    /// Construct a flowing-OUT wavefunction.
    pub fn new(p: LorentzVector<F>, mass: F, nhel: SpinorHelicity, nsf: Charge) -> Self {
        let spinor = Bispinor::oxxxxx(p, mass, nhel, nsf);
        Self::from_parts(
            spinor,
            match nsf {
                Charge::Particle => p,      // outgoing: +p
                Charge::Antiparticle => -p, // incoming: -p
            },
        )
    }

    /// Construct an off-shell flowing-OUT fermion from an arbitrary spinor and momentum.
    ///
    /// Used by `fioxxx` and similar off-shell vertex routines.
    pub fn from_spinor(spinor: Bispinor<F>, momentum: LorentzVector<F>) -> Self {
        Self::from_parts(spinor, momentum)
    }

    /// Convert to a flowing-IN wavefunction by taking the Dirac conjugate of the spinor
    ///
    /// This is the inverse of [`InDiracWf::to_outgoing`].
    pub fn to_incoming(self) -> InDiracWf<F> {
        self.flip_flow()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Vector (gauge boson) wavefunction
// ──────────────────────────────────────────────────────────────────────────────

/// An off-shell vector wavefunction: 4 complex polarisation components plus
/// the associated 4-momentum.
///
/// Used as both the result of `j3xxxx` and the input to `iovxxx`.
#[derive(Clone, Copy, Debug)]
pub struct VectorWf<F: Real> {
    /// Polarisation / Lorentz components in HELAS convention.
    ///
    /// `iovxxx` contracts these components with bilinear currents using an
    /// explicit Minkowski (+,−,−,−) contraction (`mink_dot`).
    pub eps: ComplexVector<F>,
    pub momentum: LorentzVector<F>,
}

impl<F: Real> VectorWf<F> {
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
    pub fn vxxxxx(p: LorentzVector<F>, vmass: F, nhel: i32, nsv: i32) -> Self {
        let two = F::one() + F::one();
        let sqh = (F::one() / two).sqrt();
        let hel = F::from(nhel).unwrap();
        let nsvahl = F::from(nsv).unwrap() * hel.abs();

        let p0 = p[0];
        let p1 = p[1];
        let p2 = p[2];
        let p3 = p[3];

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
                    C::new(F::zero(), F::zero()),
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
            eps: ComplexVector(eps),
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
    pub momentum: LorentzVector<F>,
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
    pub fn sxxxxx(p: LorentzVector<F>, nss: i32) -> Self {
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

    #[test]
    fn test_vxxxxx_massless_along_z_positive_helicity() {
        // Massless photon along +z axis with E=1: p = [1, 0, 0, 1]
        let p = LorentzVector([1.0, 0.0, 0.0, 1.0]);
        let wf = VectorWf::vxxxxx(p, 0.0, 1, 1);

        // Momentum should be p (nsv=+1 for outgoing)
        assert_eq!(wf.momentum[0], 1.0);
        assert_eq!(wf.momentum[3], 1.0);

        // For massless along z, ε_0 = 0
        assert_eq!(wf.eps.0[0].re, 0.0);
        assert_eq!(wf.eps.0[0].im, 0.0);
    }

    #[test]
    fn test_vxxxxx_massive_at_rest() {
        // Massive vector at rest: p = [m, 0, 0, 0]
        let p = LorentzVector([1.0, 0.0, 0.0, 0.0]);
        let wf = VectorWf::vxxxxx(p, 1.0, 1, 1);

        // Signed momentum: nsv=1 means no sign change
        assert_eq!(wf.momentum[0], 1.0);

        // At rest with nhel=1, hel0=0, so ε_1 = -1/√2
        let sqh = 1.0 / 2.0_f64.sqrt();
        assert!((wf.eps.0[1].re + sqh).abs() < 1e-10);
    }

    #[test]
    fn test_vxxxxx_incoming_vs_outgoing() {
        let p = LorentzVector([2.0, 1.0, 0.5, 0.2]);
        let wf_out = VectorWf::vxxxxx(p, 0.5, 0, 1); // outgoing (nsv=+1)
        let wf_in = VectorWf::vxxxxx(p, 0.5, 0, -1); // incoming (nsv=-1)

        // Momenta should have opposite signs
        assert_eq!(wf_out.momentum[0], 2.0);
        assert_eq!(wf_in.momentum[0], -2.0);
    }

    #[test]
    fn test_sxxxxx_scalar() {
        let p = LorentzVector([1.0, 0.5, 0.3, 0.2]);
        let wf = ScalarWf::sxxxxx(p, 1);

        // Scalar amplitude is always 1+0i
        assert_eq!(wf.value.re, 1.0);
        assert_eq!(wf.value.im, 0.0);

        // Momentum should be +p (nsv=1)
        assert_eq!(wf.momentum[0], 1.0);
        assert_eq!(wf.momentum[1], 0.5);
    }

    #[test]
    fn test_sxxxxx_incoming() {
        let p = LorentzVector([2.0, 0.1, 0.2, 0.3]);
        let wf = ScalarWf::sxxxxx(p, -1);

        // Momentum should be -p (nsv=-1)
        assert_eq!(wf.momentum[0], -2.0);
        assert_eq!(wf.momentum[1], -0.1);
    }

    fn generate_test_cases(
        onshell_only: bool,
    ) -> impl Iterator<Item = ((LorentzVector<f64>, f64), SpinorHelicity, Charge)> {
        let p1 = LorentzVector([2.0, 0.5, -0.3, 1.2]);
        let p2 = LorentzVector([(3.5_f64).sqrt(), 0.5, -1.0, 1.5]);
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
            offshell_cases
                .into_iter()
                .chain(onshell_cases.into_iter())
                .collect()
        };
        itertools::iproduct!(
            lorentz_cases,
            [SpinorHelicity::Up, SpinorHelicity::Down],
            [Charge::Particle, Charge::Antiparticle]
        )
    }

    /// Test that ixxxxx and oxxxxx both produce valid spinors.
    ///
    /// HELAS convention:
    /// - ixxxxx(p, m, nhel, nsf) creates an incoming spinor (column): u/v spinors
    /// - oxxxxx(p, m, nhel, nsf) creates an outgoing spinor (row): ū/v̄ spinors
    ///
    /// Both use the same momentum, helicity, and charge (nsf) parameters.
    /// The difference is:
    /// 1. Component indexing (which Weyl chiralities are where)
    /// 2. Complex conjugation of transverse phases (χ[1] uses -p[2] in oxxxxx)
    /// 3. Swapping of omega factors (sfomeg[0] ↔ sfomeg[1])
    ///
    /// These differences reflect the Dirac conjugate relationship ψ̄ = ψ† γ⁰,
    #[test]
    fn test_ixxxxx_oxxxxx() {
        for ((p, mass), nhel, nsf) in generate_test_cases(false) {
            let fi = Bispinor::ixxxxx(p, mass, nhel, nsf);
            let fo = Bispinor::oxxxxx(p, mass, nhel, nsf);

            // Both should produce non-zero spinors
            assert!(
                fi.0.iter().any(|c| c.norm() > 1e-10),
                "ixxxxx should produce non-zero spinor"
            );
            assert!(
                fo.0.iter().any(|c| c.norm() > 1e-10),
                "oxxxxx should produce non-zero spinor"
            );

            // The norms should be equal (both represent a spinor at momentum p)
            let fi_norm: f64 = fi.0.iter().map(|c| c.norm_sqr()).sum();
            let fo_norm: f64 = fo.0.iter().map(|c| c.norm_sqr()).sum();
            assert!(fi_norm > 0.0, "ixxxxx norm should be non-zero");
            assert!(fo_norm > 0.0, "oxxxxx norm should be non-zero");

            // The norms should be equal because both represent the same physical state
            assert!(
                (fi_norm - fo_norm).abs() / fi_norm < 1e-10,
                "ixxxxx and oxxxxx with same params should have equal norms"
            );

            // They should be dirac conjugates of each other
            assert_eq!(fi.dirac_adjoint(), fo);
        }
    }

    /// Test bilinear scalar norm for the spinors
    #[test]
    fn test_bilinear_scalar_norm() {
        for ((p, mass), nhel, nsf) in generate_test_cases(true) {
            let in_wf = InDiracWf::new(p, mass, nhel, nsf);
            let scalar_bilinear = in_wf.scalar_bilinear(&in_wf.to_outgoing(), Chirality::Both);
            // HELAS convention for the scalar bilinear norm is 2 * p[0] (twice the energy component of the momentum)
            let expected = 2.0 * p[0];
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

            // now check orthogonality
            let in_wf_oh = InDiracWf::new(p, mass, nhel.flip(), nsf);
            let scalar_bilinear_orthogonal =
                in_wf.scalar_bilinear(&in_wf_oh.to_outgoing(), Chirality::Both);
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
        for ((p, mass), nhel, nsf) in generate_test_cases(false) {
            let in_wf = InDiracWf::new(p, mass, nhel, nsf);
            let out_wf = OutDiracWf::new(p, mass, nhel, nsf);
            let in_wf_converted = out_wf.to_incoming();
            let out_wf_converted = in_wf.to_outgoing();

            // The converted wavefunction should match the original
            assert_eq!(in_wf, in_wf_converted);
            assert_eq!(out_wf, out_wf_converted);
        }
    }
}
