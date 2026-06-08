use crate::helas::repr::lorentz::{Bispinor, Charge, ComplexVector, LorentzVector, SpinorHelicity};
use crate::helas::repr::{Real, C};
use std::marker::PhantomData;

// ──────────────────────────────────────────────────────────────────────────────
// Spinor wavefunction
// ──────────────────────────────────────────────────────────────────────────────

/// Marker for flowing-IN spinors (`u`/`v` columns).
#[derive(Clone, Copy, Debug)]
pub struct FlowIn;

/// Marker for flowing-OUT spinors (`ū`/`v̄` rows).
#[derive(Clone, Copy, Debug)]
pub struct FlowOut;

/// A Dirac spinor wavefunction together with its (signed) 4-momentum.
///
/// `momentum` stores `p * nsf.sign()`: positive for particles, negative for
/// antiparticles.  This matches the HELAS convention used when building
/// currents and computing the s-channel propagator momentum.
///
/// `spinor` has type `B::Fiber` (= `[C<F>; 4]` for any `B: SpinorRepr<F>`),
/// since [`SpinorRepr<F>`] is a subtrait of [`crate::helas::repr::LorentzRepr<F>`]
/// with `Fiber = [C<F>; 4]`.
#[derive(Clone, Copy, Debug)]
pub struct DiracWf<F: Real, Flow = FlowIn> {
    pub spinor: Bispinor<F>,
    /// Signed momentum: particle → +p, antiparticle → −p
    pub momentum: LorentzVector<F>,
    _flow: PhantomData<Flow>,
}

/// Flowing-IN typed spinor wavefunction.
pub type InDiracWf<F> = DiracWf<F, FlowIn>;

/// Flowing-OUT typed spinor wavefunction.
pub type OutDiracWf<F> = DiracWf<F, FlowOut>;

impl<F: Real, Flow> DiracWf<F, Flow> {
    #[inline(always)]
    fn from_parts(spinor: Bispinor<F>, momentum: LorentzVector<F>) -> Self {
        Self {
            spinor,
            momentum,
            _flow: PhantomData,
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
        Self {
            spinor,
            momentum,
            _flow: PhantomData,
        }
    }

    /// Convert to a flowing-OUT wavefunction by taking the Dirac conjugate of the spinor
    /// and flipping the momentum sign.
    pub fn to_outgoing(self) -> OutDiracWf<F> {
        OutDiracWf::from_spinor(self.spinor.dirac_conjugate(), -self.momentum)
    }
}

impl<F: Real> OutDiracWf<F> {
    /// Construct a flowing-OUT wavefunction.
    pub fn new(p: LorentzVector<F>, mass: F, nhel: SpinorHelicity, nsf: Charge) -> Self {
        let spinor = Bispinor::oxxxxx(p, mass, nhel, nsf);
        Self::from_parts(
            spinor,
            match nsf {
                Charge::Particle => p,
                Charge::Antiparticle => -p,
            },
        )
    }

    /// Construct an off-shell flowing-OUT fermion from an arbitrary spinor and momentum.
    ///
    /// Used by `fioxxx` and similar off-shell vertex routines.
    pub fn from_spinor(spinor: Bispinor<F>, momentum: LorentzVector<F>) -> Self {
        Self {
            spinor,
            momentum,
            _flow: PhantomData,
        }
    }

    /// Convert to a flowing-IN wavefunction by taking the Dirac conjugate of the spinor
    /// and flipping the momentum sign.
    /// This is the inverse of [`InDiracWf::to_outgoing`].
    pub fn to_incoming(self) -> InDiracWf<F> {
        InDiracWf::from_spinor(self.spinor.dirac_conjugate(), -self.momentum)
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
}

#[cfg(test)]
mod ixxxxx_oxxxxx_tests {
    use super::*;
    use crate::helas::repr::lorentz::Bispinor;

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
        let p1 = LorentzVector([2.0, 0.5, -0.3, 1.2]);
        let p2 = LorentzVector([(3.5_f64).sqrt(), 0.5, -1.0, 1.5]);
        assert!(p2.m() < 1e-10, "p2 should be massless for this test");
        let lorentz_cases = vec![
            (p1, 0.5),    // off-shell massive
            (p1, 0.0),    // off-shell massless
            (p1, p1.m()), // on-shell massive
            (p2, 0.0),    // on-shell massless
        ];
        let cases = itertools::iproduct!(
            lorentz_cases,
            [SpinorHelicity::Up, SpinorHelicity::Down],
            [Charge::Particle, Charge::Antiparticle]
        );

        for ((p, mass), nhel, nsf) in cases {
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
            assert_eq!(fi.dirac_conjugate(), fo);
        }
    }
}
