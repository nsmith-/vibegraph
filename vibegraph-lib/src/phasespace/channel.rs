//! Phase-space abstraction seam: sampler, channel map, and integrator meet here.
//!
//! The integrator (VEGAS) only ever sees a [`PhaseSpaceMap`]: unit-hypercube
//! uniforms in, `n` momenta plus a phase-space weight out. Keeping that shape —
//! `u ∈ [0,1]^d → point` — lets a VEGAS grid compose *in front of* any map,
//! refining its per-channel hypercube without knowing what the map is.
//!
//! A single [`Channel`] is one such map on a fixed `√ŝ` and external-mass set. It
//! additionally reports the sampling *density* it assigns to an arbitrary
//! on-shell configuration, not only to points it generated. A multichannel
//! combiner needs exactly that: to reweight a point drawn from channel `i` by the
//! variance-minimising factor `1/Σⱼ(1/Jⱼ) = 1/Σⱼ gⱼ`, it evaluates every
//! channel's density `gⱼ` at the *same* momentum configuration. [`Combiner`] is
//! the seam such a combiner plugs into; assembling one is a later concern.
//!
//! Two maps sit behind the seam today: flat [`RamboChannel`] over the scalar
//! field `F`, and the massless 2-body [`Lips2Channel`].

use crate::helas::repr::lorentz::LorentzVector;
use crate::helas::repr::Real;

use super::rambo::{flat_weight, rambo};
use super::{lips2_jacobian_u, u_to_costheta};

/// A generated phase-space point: `n` on-shell momenta in the CM frame (total
/// four-momentum `(√ŝ, 0, 0, 0)`), and the phase-space weight `J = 1/g` the
/// producing map assigns, such that a flat average of `J · f` over the uniforms
/// estimates `∫ dΦ f` (the `(2π)` measure factors live in the cross-section
/// prefactor, not here).
#[derive(Clone, Debug)]
pub struct PhaseSpacePoint<F: Real> {
    pub momenta: Vec<LorentzVector<F>>,
    pub weight: F,
}

/// The unit-hypercube → phase-space map an integrator composes on top of.
///
/// Both a single [`Channel`] and a future multichannel [`Combiner`] implement
/// this, so the integrator stays agnostic to how a point was produced. `sample`
/// consumes exactly [`ndim`](PhaseSpaceMap::ndim) uniforms.
pub trait PhaseSpaceMap<F: Real> {
    /// Number of unit-hypercube coordinates consumed per point — the
    /// dimensionality a VEGAS grid is built over.
    fn ndim(&self) -> usize;

    /// Map uniforms `u ∈ [0,1]^ndim` to a phase-space point.
    fn sample(&self, u: &[F]) -> PhaseSpacePoint<F>;
}

/// A single phase-space channel on a fixed `√ŝ` and external-mass set.
///
/// Beyond generating points ([`PhaseSpaceMap::sample`]), a channel evaluates the
/// sampling density `g` it would assign to an *arbitrary* on-shell,
/// momentum-conserving configuration — the reciprocal of the weight it would
/// return had it generated that point. A multichannel combiner sums these as
/// `Σⱼ gⱼ` to form the variance-minimising weight `1/Σⱼ(1/Jⱼ)`, so it needs each
/// channel's density at the *same* point, not only where the channel drew one.
pub trait Channel<F: Real>: PhaseSpaceMap<F> {
    /// The sampling density `g` this channel assigns to `momenta`. Equal to
    /// `1 / weight` at any point the channel itself generated.
    fn density(&self, momenta: &[LorentzVector<F>]) -> F;
}

/// The seam a multichannel combiner plugs into: a set of [`Channel`]s presented
/// as one [`PhaseSpaceMap`].
///
/// A combiner selects a channel per draw (with per-channel weights `αᵢ`) and
/// reweights each generated point by `1/Σⱼ(1/Jⱼ) = 1/Σⱼ gⱼ`, gathering every
/// `gⱼ` from [`Channel::density`] at the generated configuration. It exposes its
/// channels so an integrator or a weight-adaptation pass can inspect them in a
/// fixed order matching the `αᵢ`. No combiner is built here — this is the
/// interface one will satisfy.
pub trait Combiner<F: Real>: PhaseSpaceMap<F> {
    /// The channels this combiner draws from, in the order its per-channel
    /// weights index.
    fn channels(&self) -> &[Box<dyn Channel<F>>];
}

/// Flat RAMBO as a [`Channel`]: `4n` uniforms → `n` on-shell momenta of the
/// configured `masses` at `√ŝ`, carrying the Kleiss–Stirling–Ellis weight.
///
/// The density is flat in the invariant volume: constant `1/R_n` in the massless
/// case, and `1/(R_n · massive_jacobian)` once any mass is non-zero.
#[derive(Clone, Debug)]
pub struct RamboChannel<F: Real> {
    sqrt_s: F,
    masses: Vec<F>,
}

impl<F: Real> RamboChannel<F> {
    /// A flat RAMBO channel producing `masses.len()` final-state momenta at CM
    /// energy `sqrt_s`. Requires `masses.len() >= 2` and `sqrt_s > Σ mᵢ`, checked
    /// when a point is sampled.
    pub fn new(sqrt_s: F, masses: Vec<F>) -> Self {
        RamboChannel { sqrt_s, masses }
    }

    /// The final-state masses in outgoing-leg order.
    pub fn masses(&self) -> &[F] {
        &self.masses
    }
}

impl<F: Real> PhaseSpaceMap<F> for RamboChannel<F> {
    fn ndim(&self) -> usize {
        4 * self.masses.len()
    }

    fn sample(&self, u: &[F]) -> PhaseSpacePoint<F> {
        let point = rambo(self.sqrt_s, &self.masses, u);
        PhaseSpacePoint {
            momenta: point.momenta,
            weight: point.weight,
        }
    }
}

impl<F: Real> Channel<F> for RamboChannel<F> {
    fn density(&self, momenta: &[LorentzVector<F>]) -> F {
        F::one() / flat_weight(self.sqrt_s, &self.masses, momenta)
    }
}

/// The massless 2-body LIPS map as a [`Channel`]: one uniform → two back-to-back
/// massless momenta in the CM frame, in the `x`–`z` plane.
///
/// The single degree of freedom is the CM polar angle, `cosθ = 2u − 1`, with the
/// azimuth integrated out into the weight. The weight is flat in `cosθ`, so the
/// density is the constant reciprocal of the LIPS Jacobian. Massive endpoints and
/// resonance-shaped invariants are a separate mapping, not this flat map.
#[derive(Clone, Debug)]
pub struct Lips2Channel {
    sqrt_s: f64,
}

impl Lips2Channel {
    /// A massless 2-body channel at CM energy `sqrt_s`.
    pub fn new(sqrt_s: f64) -> Self {
        Lips2Channel { sqrt_s }
    }
}

impl PhaseSpaceMap<f64> for Lips2Channel {
    fn ndim(&self) -> usize {
        1
    }

    fn sample(&self, u: &[f64]) -> PhaseSpacePoint<f64> {
        let cos_theta = u_to_costheta(u[0]);
        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
        let p = self.sqrt_s / 2.0;
        let momenta = vec![
            LorentzVector::new(p, p * sin_theta, 0.0, p * cos_theta),
            LorentzVector::new(p, -p * sin_theta, 0.0, -p * cos_theta),
        ];
        PhaseSpacePoint {
            momenta,
            weight: lips2_jacobian_u(self.sqrt_s),
        }
    }
}

impl Channel<f64> for Lips2Channel {
    fn density(&self, _momenta: &[LorentzVector<f64>]) -> f64 {
        1.0 / lips2_jacobian_u(self.sqrt_s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phasespace::rambo::rambo;
    use crate::phasespace::rng::SubStream;

    /// The RAMBO channel delegates to the free [`rambo`] map bit-for-bit, so the
    /// seam introduces no numeric change on the point it generates.
    #[test]
    fn rambo_channel_matches_free_map() {
        let mut s = SubStream::from_stream(0x0C7A, 3);
        let sqrt_s = 500.0;
        for masses in [vec![0.0; 4], vec![10.0, 20.0, 5.0], vec![80.4, 80.4]] {
            let ch = RamboChannel::new(sqrt_s, masses.clone());
            let u = s.uniforms::<f64>(4 * masses.len());
            let pt = ch.sample(&u);
            let direct = rambo(sqrt_s, &masses, &u);
            assert_eq!(pt.weight, direct.weight);
            for (a, b) in pt.momenta.iter().zip(&direct.momenta) {
                assert_eq!(a.e(), b.e());
                assert_eq!(a.px(), b.px());
                assert_eq!(a.py(), b.py());
                assert_eq!(a.pz(), b.pz());
            }
        }
    }

    /// A channel's density is the exact reciprocal of the weight it assigns to a
    /// point it generated — the identity a multichannel combiner relies on.
    #[test]
    fn rambo_density_is_reciprocal_weight() {
        let mut s = SubStream::from_stream(0x0C7B, 4);
        let sqrt_s = 500.0;
        for masses in [vec![0.0; 6], vec![10.0, 20.0, 5.0]] {
            let ch = RamboChannel::new(sqrt_s, masses.clone());
            let u = s.uniforms::<f64>(4 * masses.len());
            let pt = ch.sample(&u);
            assert_eq!(ch.density(&pt.momenta), 1.0 / pt.weight);
        }
    }

    /// The 2-body channel produces back-to-back, on-shell, energy-conserving
    /// massless momenta, and carries the free LIPS Jacobian as its weight.
    #[test]
    fn lips2_channel_kinematics_and_weight() {
        let sqrt_s = 91.2;
        let ch = Lips2Channel::new(sqrt_s);
        assert_eq!(ch.ndim(), 1);
        for &u0 in &[0.0, 0.25, 0.5, 0.9] {
            let pt = ch.sample(&[u0]);
            let (a, b) = (&pt.momenta[0], &pt.momenta[1]);
            assert!((a.e() + b.e() - sqrt_s).abs() < 1e-12);
            assert!((a.px() + b.px()).abs() < 1e-12);
            assert!((a.py() + b.py()).abs() < 1e-12);
            assert!((a.pz() + b.pz()).abs() < 1e-12);
            assert!(a.m2().abs() < 1e-9);
            assert_eq!(pt.weight, lips2_jacobian_u(sqrt_s));
        }
        assert_eq!(ch.density(&[]), 1.0 / lips2_jacobian_u(sqrt_s));
    }
}
