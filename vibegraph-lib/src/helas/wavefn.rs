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
        Self::from_parts(spinor, p.scaled(nsf.sign()))
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
}

impl<F: Real> OutDiracWf<F> {
    /// Construct a flowing-OUT wavefunction.
    pub fn new(p: LorentzVector<F>, mass: F, nhel: SpinorHelicity, nsf: Charge) -> Self {
        let spinor = Bispinor::oxxxxx(p, mass, nhel, nsf);
        Self::from_parts(spinor, p.scaled(nsf.sign()))
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
            momentum: p.scaled(nsv),
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
            momentum: p.scaled(nss),
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
