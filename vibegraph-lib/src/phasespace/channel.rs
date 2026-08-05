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
//! the seam such a combiner plugs into; [`MultiChannel`] is the one built here.
//!
//! Two maps sit behind the seam today: flat [`RamboChannel`] over the scalar
//! field `F`, and the massless 2-body [`Lips2Channel`].

use crate::helas::repr::lorentz::LorentzVector;
use crate::helas::repr::Real;

use super::rambo::{flat_weight, rambo};
use super::rng::SubStream;
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
///
/// A channel is a fixed structure that answers questions about a point — masks,
/// masses, propagator poles — and holds no evaluation state, so a combiner can be
/// read from several threads at once. `Send + Sync` states that here rather than
/// at each `Box<dyn Channel>`: a channel that needed interior mutability would be
/// a different kind of object, and the parallel integrator's channel maps are
/// shared, not copied per thread.
pub trait Channel<F: Real>: PhaseSpaceMap<F> + Send + Sync {
    /// The sampling density `g` this channel assigns to `momenta`. Equal to
    /// `1 / weight` at any point the channel itself generated.
    fn density(&self, momenta: &[LorentzVector<F>]) -> F;
}

/// A channel whose collision energy is supplied per draw instead of baked in.
///
/// A channel map is a `√ŝ`-independent structure — masks, masses, propagator poles
/// — evaluated at whatever energy the event has. [`Channel`] is the fixed-energy
/// specialisation a partonic run wants; this is what a hadronic run needs, where
/// `ŝ = τ s` changes every event and rebuilding the channel set per point would be
/// the only alternative.
/// The coordinate count is [`PhaseSpaceMap::ndim`] — a channel's dimensionality
/// does not move with the energy, so it is not restated here.
/// Shared across threads for the same reason [`Channel`] is, and stating it the
/// same way.
pub trait ScaledChannel<F: Real>: PhaseSpaceMap<F> + Send + Sync {
    /// Map uniforms to a phase-space point at CM energy `sqrt_s`, with the weight
    /// `1/g` this channel alone assigns.
    fn sample_at(&self, sqrt_s: F, u: &[F]) -> PhaseSpacePoint<F>;

    /// The sampling density this channel assigns to `momenta` at CM energy
    /// `sqrt_s`.
    fn density_at(&self, sqrt_s: F, momenta: &[LorentzVector<F>]) -> F;
}

/// One Kleiss–Pittau reallocation of the selection weights: `αⱼ ← αⱼ·Wⱼ^damping`,
/// floored strictly positive and renormalised.
///
/// `None` when the survey found no weight to reallocate (an identically zero
/// integrand, or a non-finite sum), in which case the caller keeps the `alphas` it
/// has. Shared so the rule is written once for every combiner that applies it.
pub fn kleiss_pittau_step<F: Real>(alphas: &[F], variance: &[F], damping: F) -> Option<Vec<F>> {
    let floor_frac = F::from(1e-12).expect("floor fits the scalar field");
    let mut raw: Vec<F> = alphas
        .iter()
        .zip(variance)
        .map(|(&a, &wj)| a * wj.powf(damping))
        .collect();
    let sum = raw.iter().fold(F::zero(), |acc, &x| acc + x);
    if !(sum > F::zero()) || !sum.is_finite() {
        return None;
    }
    let floor = floor_frac * sum;
    for r in &mut raw {
        *r = r.max(floor);
    }
    let renorm = F::one() / raw.iter().fold(F::zero(), |acc, &x| acc + x);
    for r in &mut raw {
        *r = *r * renorm;
    }
    Some(raw)
}

/// The channel `u0 ∈ [0,1)` selects from a normalised weight vector, by cumulative
/// weight. The last channel absorbs the rounding at the top of the interval.
pub fn select_channel<F: Real>(alphas: &[F], u0: F) -> usize {
    let mut acc = F::zero();
    for (j, alpha) in alphas.iter().enumerate() {
        acc = acc + *alpha;
        if u0 < acc {
            return j;
        }
    }
    alphas.len() - 1
}

/// The seam a multichannel combiner plugs into: a set of [`Channel`]s presented
/// as one [`PhaseSpaceMap`].
///
/// A combiner selects a channel per draw (with per-channel weights `αᵢ`) and
/// reweights each generated point by `1/Σⱼ(1/Jⱼ) = 1/Σⱼ gⱼ`, gathering every
/// `gⱼ` from [`Channel::density`] at the generated configuration. It exposes its
/// channels so an integrator or a weight-adaptation pass can inspect them in a
/// fixed order matching the `αᵢ`. [`MultiChannel`] satisfies this interface.
pub trait Combiner<F: Real>: PhaseSpaceMap<F> {
    /// The channels this combiner draws from, in the order its per-channel
    /// weights index.
    fn channels(&self) -> &[Box<dyn Channel<F>>];
}

/// A multichannel combiner: a fixed set of [`Channel`]s on a common `√ŝ` and
/// external-mass set, presented as one [`PhaseSpaceMap`].
///
/// # Estimator
///
/// Each channel `j` carries per-draw selection weight `αⱼ ≥ 0`, with `Σⱼ αⱼ = 1`.
/// A draw selects channel `i` with probability `αᵢ`, generates a point `p` from
/// channel `i`, and weights it by
///
/// ```text
/// w(p) = 1 / g(p),   g(p) = Σⱼ αⱼ gⱼ(p)
/// ```
///
/// where `gⱼ = channelⱼ.density` is evaluated at the *same* `p` for every channel
/// (the variance-minimising "recycling" combination `1/Σⱼ(1/Jⱼ)` with the `αⱼ`
/// folded in). Because `g` is the pushforward density of the selection-plus-draw
/// procedure over phase space, the estimator `w(p)·f(p)` is unbiased for
/// `∫ dΦ f` for any fixed `αⱼ`, and its variance is minimised when a single
/// channel's peak dominates `g` there — so a point drawn near one channel's
/// resonance is not over-counted by the others.
///
/// # Channel selection and VEGAS
///
/// One unit-hypercube coordinate is reserved for the discrete channel draw:
/// `ndim = 1 + channel_ndim`. Given `u ∈ [0,1]^ndim`, `u[0]` picks the channel by
/// its cumulative `αⱼ` and `u[1..]` feeds the chosen channel. Selection is thus a
/// deterministic, replayable function of `u`, so the whole map is a pure
/// `sample(u)` a VEGAS grid composes in front of: VEGAS refines every coordinate,
/// including the selection coordinate, and stays unbiased because the integrand
/// map `u ↦ w(p(u))·f(p(u))` integrates to `∫ dΦ f` over the fixed unit hypercube
/// regardless of how VEGAS remaps it. The `αⱼ` are held fixed here; adapting them
/// toward each channel's variance share is a separate concern that reads and
/// rewrites [`alphas`](MultiChannel::alphas)/[`set_alphas`](MultiChannel::set_alphas)
/// without touching the estimator.
///
/// # Splitting the estimator by channel
///
/// The mixture above is one integral over `1 + channel_ndim` coordinates. The same
/// estimator also splits into one integral per channel,
///
/// ```text
/// ∫ dΦ f = Σⱼ ∫ dΦ f·αⱼgⱼ/g = Σⱼ E_{p∼gⱼ}[ αⱼ·f(p)/g(p) ]
/// ```
///
/// whose `j`-th term is [`sample_channel`](Self::sample_channel): draw from channel
/// `j` alone over `channel_ndim` coordinates — no selection coordinate — and weight
/// by `αⱼ/g` with the *same* combined `g` the mixture uses. Summing the terms
/// recovers the same integral, so the two arrangements differ only in how sampling
/// effort and any importance grid in front of them are organised: a grid per term
/// can learn a density conditional on the channel, which a single separable grid
/// over the mixture cannot express.
pub struct MultiChannel<F: Real> {
    channels: Vec<Box<dyn Channel<F>>>,
    alphas: Vec<F>,
    channel_ndim: usize,
}

/// The record of a survey→refine α-adaptation pass ([`MultiChannel::adapt_alphas`]).
///
/// The channel selection weights `αⱼ` are driven toward the variance-minimising
/// mixture by the Kleiss–Pittau reallocation rule (R. Kleiss, R. Pittau,
/// *Weight optimization in multichannel Monte Carlo*, Comput. Phys. Commun. 83
/// (1994) 141; the survey/refine "job strategy" of MG's phase-space appendix,
/// note 01 §A). Each survey estimates every channel's contribution to the
/// estimator variance,
///
/// ```text
/// Wⱼ = ∫ f(p)²·gⱼ(p)/g(p)² dΦ = E_g[(f/g)²·gⱼ/g],   g = Σₖ αₖ gₖ,
/// ```
///
/// and reallocates weight toward the channels carrying more of it,
///
/// ```text
/// αⱼ ← αⱼ·Wⱼ^β / Σₖ αₖ·Wₖ^β   (β = ½, the standard exponent).
/// ```
///
/// The stationary point of this map is `Wⱼ = const` over channels with `αⱼ > 0`
/// — exactly the condition `∂(∫f²/g)/∂αⱼ = −Wⱼ = −λ` that minimises the estimator
/// variance `∫ f²/g dΦ − I²` under `Σⱼ αⱼ = 1`. So a converged α *equalises the
/// per-channel variance share*: no reallocation of samples across channels lowers
/// the variance further.
#[derive(Clone, Debug)]
pub struct AlphaAdaptation<F: Real> {
    /// The α vector before each survey, plus the converged vector installed on the
    /// combiner: `trajectory[0]` is the starting α, `trajectory[k]` the α used in
    /// survey `k`, and the last entry the final α. Reports the whole refinement
    /// path so convergence can be read off directly.
    pub trajectory: Vec<Vec<F>>,
    /// The per-channel variance contribution `Wⱼ` estimated on the final survey. At
    /// the variance-minimising fixed point these are equal across channels carrying
    /// weight — the "variance share" the reallocation equalises.
    pub variance_shares: Vec<F>,
}

impl<F: Real> MultiChannel<F> {
    /// Combine `channels` under explicit selection weights `alphas` (`Σ αⱼ = 1`,
    /// each `> 0`). All channels must share one [`ndim`](PhaseSpaceMap::ndim) — they
    /// parametrise the same `n`-body final state at the same `√ŝ`.
    pub fn new(channels: Vec<Box<dyn Channel<F>>>, alphas: Vec<F>) -> Self {
        assert!(
            !channels.is_empty(),
            "a combiner needs at least one channel"
        );
        assert_eq!(
            channels.len(),
            alphas.len(),
            "one selection weight per channel"
        );
        let channel_ndim = channels[0].ndim();
        assert!(
            channels.iter().all(|c| c.ndim() == channel_ndim),
            "all channels must share one ndim"
        );
        let mc = MultiChannel {
            channels,
            alphas,
            channel_ndim,
        };
        mc.assert_normalized();
        mc
    }

    /// Combine `channels` with uniform selection weights `αⱼ = 1/N`.
    pub fn uniform(channels: Vec<Box<dyn Channel<F>>>) -> Self {
        let n = channels.len();
        assert!(n > 0, "a combiner needs at least one channel");
        let alpha = F::one() / F::from(n).expect("channel count fits the scalar field");
        let alphas = vec![alpha; n];
        MultiChannel::new(channels, alphas)
    }

    /// The current per-channel selection weights, in channel order.
    pub fn alphas(&self) -> &[F] {
        &self.alphas
    }

    /// Replace the selection weights (`Σ αⱼ = 1`, each `> 0`), keeping the channel
    /// set. The reweighting hook a channel-weight adaptation drives; the estimator
    /// and [`density`](Self::density) pick the new `αⱼ` up unchanged.
    pub fn set_alphas(&mut self, alphas: Vec<F>) {
        assert_eq!(
            alphas.len(),
            self.channels.len(),
            "one selection weight per channel"
        );
        self.alphas = alphas;
        self.assert_normalized();
    }

    /// Refine the selection weights `αⱼ` toward the variance-minimising channel
    /// mixture by a survey→refine loop, installing the converged weights on the
    /// combiner and returning the refinement path ([`AlphaAdaptation`]).
    ///
    /// Each of `n_iter` iterations opens an independent, replayable substream
    /// (`stream + iteration` of `seed`), draws `n_survey` points from the combiner
    /// under the *current* α, and estimates every channel's variance contribution
    /// `Wⱼ = E_g[(f/g)²·gⱼ/g]` from those points (each point informs every channel,
    /// so the α estimate itself is low-variance). It then reallocates weight by the
    /// Kleiss–Pittau rule `αⱼ ← αⱼ·Wⱼ^damping`, renormalised, with `damping = ½`
    /// the standard exponent. The per-point cost and the estimator are unchanged —
    /// α-adaptation reshapes only the channel mixture, not the integral — so a
    /// converged combiner is handed straight to VEGAS as its integrand map (VEGAS
    /// refines the per-channel unit hypercube with α frozen).
    ///
    /// A survey that finds the integrand identically zero (or leaves the weights
    /// non-finite) leaves α untouched and stops. Otherwise every weight is floored
    /// strictly positive before reinstalling, so one under-sampled survey never
    /// permanently kills a channel.
    ///
    /// `integrand` is evaluated at the drawn point *and* the channel that drew
    /// it, because an integrand carrying a per-event scale prescription is not a
    /// function of the momenta alone: the cluster scale reads the integration
    /// channel. The survey therefore sees the same integrand the integration
    /// will.
    pub fn adapt_alphas(
        &mut self,
        integrand: impl Fn(&[LorentzVector<F>], usize) -> F,
        seed: u64,
        stream: u64,
        n_survey: usize,
        n_iter: usize,
        damping: F,
    ) -> AlphaAdaptation<F> {
        let n = self.channels.len();
        let ndim = self.ndim();
        let inv_survey = F::one() / F::from(n_survey).expect("survey size fits the scalar field");

        let mut trajectory = vec![self.alphas.clone()];
        let mut variance_shares = vec![F::zero(); n];

        for it in 0..n_iter {
            let mut s = SubStream::from_stream(seed, stream + it as u64);
            let mut w = vec![F::zero(); n];
            for _ in 0..n_survey {
                let u = s.uniforms::<F>(ndim);
                let (drawn, pt) = self.sample_from(&u);
                // g(p) = 1/weight is the combined density; est = weight·f = f/g.
                let g = F::one() / pt.weight;
                let est = pt.weight * integrand(&pt.momenta, drawn);
                let est2 = est * est;
                for (wj, ch) in w.iter_mut().zip(&self.channels) {
                    *wj = *wj + est2 * ch.density(&pt.momenta) / g;
                }
            }
            for wj in &mut w {
                *wj = *wj * inv_survey;
            }
            variance_shares = w.clone();

            let Some(raw) = kleiss_pittau_step(&self.alphas, &w, damping) else {
                break;
            };
            self.set_alphas(raw.clone());
            trajectory.push(raw);
        }

        AlphaAdaptation {
            trajectory,
            variance_shares,
        }
    }

    /// The combined sampling density `g(p) = Σⱼ αⱼ gⱼ(p)` at `momenta` — the
    /// reciprocal of the weight the combiner assigns to a point generated there.
    pub fn density(&self, momenta: &[LorentzVector<F>]) -> F {
        let mut g = F::zero();
        for (alpha, ch) in self.alphas.iter().zip(&self.channels) {
            g = g + *alpha * ch.density(momenta);
        }
        g
    }

    /// The uniforms one channel consumes on its own — [`ndim`](PhaseSpaceMap::ndim)
    /// less the coordinate the mixture reserves for its channel draw.
    pub fn channel_ndim(&self) -> usize {
        self.channel_ndim
    }

    /// Draw from channel `j` alone: `u ∈ [0,1]^channel_ndim` (no selection
    /// coordinate), weighted by `αⱼ/g(p)` with `g = Σₖ αₖ gₖ` the same combined
    /// density the mixture divides by.
    ///
    /// The flat average of `weight · f` over the uniforms estimates the `j`-th term
    /// of the channel-split estimator, `∫ dΦ f·αⱼgⱼ/g`, so summing that average over
    /// all channels estimates `∫ dΦ f` — the same integral
    /// [`sample`](PhaseSpaceMap::sample) estimates from the mixture, at the same
    /// per-point cost.
    ///
    /// # Panics
    ///
    /// If `j` is not a channel index.
    pub fn sample_channel(&self, j: usize, u: &[F]) -> PhaseSpacePoint<F> {
        assert!(j < self.channels.len(), "channel index out of range");
        let pt = self.channels[j].sample(u);
        let g = self.density(&pt.momenta);
        debug_assert!(g > F::zero(), "combined density must be positive");
        PhaseSpacePoint {
            momenta: pt.momenta,
            weight: self.alphas[j] / g,
        }
    }

    /// [`PhaseSpaceMap::sample`] with the channel that drew the point reported
    /// alongside it.
    ///
    /// The point and its `1/g` weight are the mixture's own either way; what the
    /// index adds is the channel an integrand needs when it is not a pure
    /// function of the momenta — a per-event scale prescription that reads the
    /// integration channel, for one.
    pub fn sample_from(&self, u: &[F]) -> (usize, PhaseSpacePoint<F>) {
        let idx = self.select(u[0]);
        let pt = self.channels[idx].sample(&u[1..]);
        // g ≥ α_idx · g_idx = α_idx / pt.weight > 0: the generating channel's own
        // positive density keeps the denominator off zero, so `1/g` is the exact
        // reciprocal the reciprocity contract requires.
        let g = self.density(&pt.momenta);
        debug_assert!(g > F::zero(), "combined density must be positive");
        (
            idx,
            PhaseSpacePoint {
                momenta: pt.momenta,
                weight: F::one() / g,
            },
        )
    }

    /// The channel `u0 ∈ [0,1)` selects, by cumulative selection weight.
    fn select(&self, u0: F) -> usize {
        select_channel(&self.alphas, u0)
    }

    fn assert_normalized(&self) {
        let sum = self.alphas.iter().fold(F::zero(), |a, &x| a + x);
        let eps = F::from(1e-9).expect("tolerance fits the scalar field");
        assert!(
            self.alphas.iter().all(|&a| a > F::zero()),
            "selection weights must be positive"
        );
        assert!(
            (sum - F::one()).abs() < eps,
            "selection weights must sum to 1"
        );
    }
}

impl<F: Real> PhaseSpaceMap<F> for MultiChannel<F> {
    fn ndim(&self) -> usize {
        1 + self.channel_ndim
    }

    fn sample(&self, u: &[F]) -> PhaseSpacePoint<F> {
        self.sample_from(u).1
    }
}

impl<F: Real> Combiner<F> for MultiChannel<F> {
    fn channels(&self) -> &[Box<dyn Channel<F>>] {
        &self.channels
    }
}

/// [`MultiChannel`]'s per-event-energy counterpart: a fixed set of
/// [`ScaledChannel`]s combined under selection weights, with the collision energy
/// supplied per draw.
///
/// The estimator, the combined density `g = Σⱼ αⱼ gⱼ` and the channel-split
/// arrangement are exactly [`MultiChannel`]'s, read at the draw's own `√ŝ`. What it
/// deliberately does *not* carry is a [`PhaseSpaceMap`] impl: a hadronic integrand
/// prepends its own `(τ, y)` coordinates, so the unit hypercube the integrator
/// sees is not this combiner's, and the α-adaptation survey has to be driven by
/// the integrand that owns the outer map. [`kleiss_pittau_step`] is the shared
/// reallocation rule such a driver applies.
pub struct ScaledMultiChannel<F: Real> {
    channels: Vec<Box<dyn ScaledChannel<F>>>,
    alphas: Vec<F>,
    channel_ndim: usize,
}

impl<F: Real> ScaledMultiChannel<F> {
    /// Combine `channels` with uniform selection weights `αⱼ = 1/N`. All channels
    /// must share one [`ndim`](ScaledChannel::ndim) — they parametrise the same
    /// `n`-body final state.
    pub fn uniform(channels: Vec<Box<dyn ScaledChannel<F>>>) -> Self {
        assert!(
            !channels.is_empty(),
            "a combiner needs at least one channel"
        );
        let channel_ndim = channels[0].ndim();
        assert!(
            channels.iter().all(|c| c.ndim() == channel_ndim),
            "all channels must share one ndim"
        );
        let alpha =
            F::one() / F::from(channels.len()).expect("channel count fits the scalar field");
        let alphas = vec![alpha; channels.len()];
        ScaledMultiChannel {
            channels,
            alphas,
            channel_ndim,
        }
    }

    pub fn channels(&self) -> &[Box<dyn ScaledChannel<F>>] {
        &self.channels
    }

    pub fn alphas(&self) -> &[F] {
        &self.alphas
    }

    /// Replace the selection weights (`Σ αⱼ = 1`, each `> 0`), keeping the channels.
    pub fn set_alphas(&mut self, alphas: Vec<F>) {
        assert_eq!(
            alphas.len(),
            self.channels.len(),
            "one selection weight per channel"
        );
        let sum = alphas.iter().fold(F::zero(), |a, &x| a + x);
        let eps = F::from(1e-9).expect("tolerance fits the scalar field");
        assert!(
            alphas.iter().all(|&a| a > F::zero()),
            "selection weights must be positive"
        );
        assert!(
            (sum - F::one()).abs() < eps,
            "selection weights must sum to 1"
        );
        self.alphas = alphas;
    }

    /// The uniforms one channel consumes on its own.
    pub fn channel_ndim(&self) -> usize {
        self.channel_ndim
    }

    /// The combined sampling density `g(p) = Σⱼ αⱼ gⱼ(p)` at CM energy `sqrt_s`.
    pub fn density_at(&self, sqrt_s: F, momenta: &[LorentzVector<F>]) -> F {
        let mut g = F::zero();
        for (alpha, ch) in self.alphas.iter().zip(&self.channels) {
            g = g + *alpha * ch.density_at(sqrt_s, momenta);
        }
        g
    }

    /// Draw from channel `j` alone at CM energy `sqrt_s`, weighted by `αⱼ/g`.
    ///
    /// # Panics
    ///
    /// If `j` is not a channel index.
    pub fn sample_channel_at(&self, j: usize, sqrt_s: F, u: &[F]) -> PhaseSpacePoint<F> {
        assert!(j < self.channels.len(), "channel index out of range");
        let pt = self.channels[j].sample_at(sqrt_s, u);
        let g = self.density_at(sqrt_s, &pt.momenta);
        PhaseSpacePoint {
            momenta: pt.momenta,
            weight: self.alphas[j] / g,
        }
    }

    /// The channel `u0 ∈ [0,1)` selects, by cumulative selection weight.
    pub fn select(&self, u0: F) -> usize {
        select_channel(&self.alphas, u0)
    }
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

    // ── Multichannel combiner ────────────────────────────────────────────────

    use crate::phasespace::diagram_channel::{DiagramChannel, Resonance};
    use crate::phasespace::rambo::massless_volume;

    const M_Z: f64 = 91.1876;
    const G_Z: f64 = 2.4952;

    fn z_res() -> Resonance<f64> {
        Resonance {
            mass: M_Z,
            width: G_Z,
        }
    }

    /// Invariant mass² of the outgoing pair `(i, j)`.
    fn s_pair(p: &[LorentzVector<f64>], i: usize, j: usize) -> f64 {
        let (a, b) = (&p[i], &p[j]);
        let e = a.e() + b.e();
        let px = a.px() + b.px();
        let py = a.py() + b.py();
        let pz = a.pz() + b.pz();
        e * e - px * px - py * py - pz * pz
    }

    /// Monte-Carlo mean and per-point estimator variance of `weight·f` over a map,
    /// drawing `ndim` uniforms per point from a fixed substream.
    fn mc_estimate(
        map: &dyn PhaseSpaceMap<f64>,
        seed: u64,
        stream: u64,
        n: usize,
        f: impl Fn(&[LorentzVector<f64>]) -> f64,
    ) -> (f64, f64) {
        let mut s = SubStream::from_stream(seed, stream);
        let ndim = map.ndim();
        let (mut sum, mut sum_sq) = (0.0, 0.0);
        for _ in 0..n {
            let u = s.uniforms::<f64>(ndim);
            let pt = map.sample(&u);
            let v = pt.weight * f(&pt.momenta);
            sum += v;
            sum_sq += v * v;
        }
        let mean = sum / n as f64;
        let var = (sum_sq / n as f64 - mean * mean).max(0.0);
        (mean, var)
    }

    /// The combiner reserves one coordinate for the channel draw, its selection
    /// reaches every channel, and its density is the reciprocal of the weight it
    /// assigns to a generated point — the reciprocity contract for the combiner.
    #[test]
    fn multichannel_reciprocity_and_selection() {
        let a = DiagramChannel::from_topology(500.0, vec![0.0; 3], &[vec![0, 1]]);
        let b = DiagramChannel::from_topology(500.0, vec![0.0; 3], &[vec![1, 2]]);
        let multi = MultiChannel::uniform(vec![Box::new(a), Box::new(b)]);

        assert_eq!(multi.ndim(), 1 + (3 * 3 - 4));
        assert_eq!(multi.channels().len(), 2);
        // Uniform weights split the unit interval evenly and both ends resolve.
        assert_eq!(multi.select(0.0), 0);
        assert_eq!(multi.select(0.49), 0);
        assert_eq!(multi.select(0.51), 1);
        assert_eq!(multi.select(0.999_999), 1);

        let mut s = SubStream::from_stream(0x11CE, 21);
        for _ in 0..1000 {
            let u = s.uniforms::<f64>(multi.ndim());
            let pt = multi.sample(&u);
            let recip = 1.0 / pt.weight;
            assert!(
                (multi.density(&pt.momenta) - recip).abs() <= 1e-9 * recip,
                "combiner density {} not reciprocal of weight {}",
                multi.density(&pt.momenta),
                pt.weight
            );
            assert!(pt.weight > 0.0 && pt.weight.is_finite());
        }
    }

    /// Combining channels does not bias the integral: a uniform combiner over two
    /// distinct flat decomposition trees reproduces the analytic massless
    /// phase-space volume `V_n` for a spread of multiplicities.
    #[test]
    fn multichannel_unbiased_volume() {
        // (√s, n_out, one decomposition tree per channel).
        type Case = (f64, usize, Vec<Vec<Vec<usize>>>);
        let cases: Vec<Case> = vec![
            // 2→3: pair {0,1} vs pair {1,2}.
            (500.0, 3, vec![vec![vec![0, 1]], vec![vec![1, 2]]]),
            // 2→4: balanced {0,1}{2,3} vs crossed {0,2}{1,3}.
            (
                600.0,
                4,
                vec![vec![vec![0, 1], vec![2, 3]], vec![vec![0, 2], vec![1, 3]]],
            ),
        ];
        for (sqrt_s, n, trees) in cases {
            let channels: Vec<Box<dyn Channel<f64>>> = trees
                .iter()
                .map(|subs| {
                    Box::new(DiagramChannel::from_topology(sqrt_s, vec![0.0; n], subs))
                        as Box<dyn Channel<f64>>
                })
                .collect();
            let multi = MultiChannel::uniform(channels);
            let (mean, var) = mc_estimate(&multi, 0xC0A1, 23, 600_000, |_| 1.0);
            let err = (var / 600_000.0).sqrt();
            let analytic = massless_volume(sqrt_s, n);
            eprintln!(
                "n={n}: multichannel V_n = {mean:.6e} ± {err:.2e}, analytic {analytic:.6e}, \
                 diff {:.2e}",
                (mean - analytic).abs()
            );
            assert!(
                (mean - analytic).abs() < 5.0 * err,
                "n={n}: combiner V_n {mean:.6e} ± {err:.2e} vs analytic {analytic:.6e}"
            );
        }
    }

    /// The headline: on a genuinely multi-peak integrand — one pole on `s₀₁`, one on
    /// `s₁₂`, reachable through channels that resonate on *different* pairings — the
    /// multichannel weight has variance strictly below every single channel alone
    /// and below flat RAMBO at fixed `N`, while all estimators agree on the integral.
    /// No single channel covers both peaks, so combining is what shrinks the
    /// variance; a wrong combined density would either bias the integral or fail to
    /// suppress the tail.
    #[test]
    fn multichannel_beats_baselines_on_multi_peak() {
        let (m2, mg) = (M_Z * M_Z, M_Z * G_Z);
        let bw = move |s: f64| 1.0 / ((s - m2).powi(2) + mg * mg);
        let f = move |p: &[LorentzVector<f64>]| bw(s_pair(p, 0, 1)) + bw(s_pair(p, 1, 2));
        let sqrt_s = 500.0;
        let masses = vec![0.0; 3];

        let a = DiagramChannel::from_topology_resonant(
            sqrt_s,
            masses.clone(),
            &[(vec![0, 1], Some(z_res()))],
        );
        let b = DiagramChannel::from_topology_resonant(
            sqrt_s,
            masses.clone(),
            &[(vec![1, 2], Some(z_res()))],
        );
        let multi = MultiChannel::uniform(vec![Box::new(a.clone()), Box::new(b.clone())]);
        let flat = RamboChannel::new(sqrt_s, masses.clone());

        let n = 400_000;
        let (mean_m, var_m) = mc_estimate(&multi, 0x5EED, 31, n, f);
        let (mean_a, var_a) = mc_estimate(&a, 0x5EE1, 33, n, f);
        let (mean_b, var_b) = mc_estimate(&b, 0x5EE2, 35, n, f);
        let (mean_f, var_f) = mc_estimate(&flat, 0x5EE3, 37, n, f);

        eprintln!(
            "multi-peak σ: multi {mean_m:.6e} (var {var_m:.3e}) | \
             chan-A {mean_a:.6e} (var {var_a:.3e}, ratio {:.1}×) | \
             chan-B {mean_b:.6e} (var {var_b:.3e}, ratio {:.1}×) | \
             flat {mean_f:.6e} (var {var_f:.3e}, ratio {:.1}×)",
            var_a / var_m,
            var_b / var_m,
            var_f / var_m
        );

        // Every estimator is unbiased for the same integral: the low-variance
        // combiner agrees with each baseline within their combined MC error (wide
        // where a baseline under-samples a peak, but a biased combiner would exceed
        // it).
        for (name, mean_x, var_x) in [
            ("chan-A", mean_a, var_a),
            ("chan-B", mean_b, var_b),
            ("flat", mean_f, var_f),
        ] {
            let err = ((var_m + var_x) / n as f64).sqrt();
            assert!(
                (mean_m - mean_x).abs() < 6.0 * err,
                "combiner {mean_m:.6e} disagrees with {name} {mean_x:.6e} (err {err:.2e})"
            );
        }

        // The deliverable: strictly below every single channel and below flat.
        assert!(
            var_m < var_a,
            "combiner variance {var_m:.3e} not below channel-A {var_a:.3e}"
        );
        assert!(
            var_m < var_b,
            "combiner variance {var_m:.3e} not below channel-B {var_b:.3e}"
        );
        assert!(
            var_m < var_f,
            "combiner variance {var_m:.3e} not below flat RAMBO {var_f:.3e}"
        );
    }

    /// Monte-Carlo mean and per-point variance of `weight·f` over one frozen
    /// channel of a combiner, drawing `channel_ndim` uniforms per point.
    fn mc_estimate_channel(
        multi: &MultiChannel<f64>,
        j: usize,
        seed: u64,
        stream: u64,
        n: usize,
        f: impl Fn(&[LorentzVector<f64>]) -> f64,
    ) -> (f64, f64) {
        let mut s = SubStream::from_stream(seed, stream);
        let (mut sum, mut sum_sq) = (0.0, 0.0);
        for _ in 0..n {
            let u = s.uniforms::<f64>(multi.channel_ndim());
            let pt = multi.sample_channel(j, &u);
            let v = pt.weight * f(&pt.momenta);
            sum += v;
            sum_sq += v * v;
        }
        let mean = sum / n as f64;
        let var = (sum_sq / n as f64 - mean * mean).max(0.0);
        (mean, var)
    }

    /// The channel-split estimator sums to the same integral the mixture estimates:
    /// `Σⱼ E_{gⱼ}[αⱼf/g] = E_g[f/g]`. Pinned on a flat integrand against the analytic
    /// massless volume (which fixes the absolute normalisation, so a missing or
    /// double-counted `αⱼ` is visible) and on the resonant integrand against the
    /// mixture estimate.
    #[test]
    fn channel_split_sums_to_the_mixture_integral() {
        let sqrt_s = 500.0;
        let (m2, mg) = (M_Z * M_Z, M_Z * G_Z);
        let bw = move |s: f64| 1.0 / ((s - m2).powi(2) + mg * mg);
        let f = move |p: &[LorentzVector<f64>]| 4.0 * bw(s_pair(p, 0, 1)) + bw(s_pair(p, 1, 2));

        // A non-uniform α, so a term that forgot its αⱼ cannot pass by symmetry.
        let mut multi = two_peak_combiner(sqrt_s);
        multi.set_alphas(vec![0.7, 0.3]);

        let n = 400_000;
        for (name, integrand) in [
            (
                "flat",
                &(|_: &[LorentzVector<f64>]| 1.0) as &dyn Fn(&[LorentzVector<f64>]) -> f64,
            ),
            ("resonant", &f as &dyn Fn(&[LorentzVector<f64>]) -> f64),
        ] {
            let (mean_mix, var_mix) = mc_estimate(&multi, 0x5911, 41, n, |p| integrand(p));
            let mut split = 0.0;
            let mut var_split = 0.0;
            for j in 0..multi.channels().len() {
                let (mean_j, var_j) =
                    mc_estimate_channel(&multi, j, 0x5912, 43 + j as u64, n, |p| integrand(p));
                split += mean_j;
                var_split += var_j;
            }
            let err = ((var_mix + var_split) / n as f64).sqrt();
            eprintln!(
                "{name}: mixture {mean_mix:.6e} vs channel-split {split:.6e} (err {err:.2e})"
            );
            assert!(
                (split - mean_mix).abs() < 6.0 * err,
                "{name}: channel-split {split:.6e} disagrees with mixture {mean_mix:.6e} \
                 (err {err:.2e})"
            );
        }

        // Absolute normalisation: the split reproduces V₃ on the flat integrand.
        // Breit–Wigner-mapped channels sample a flat integrand at high variance, so
        // the comparison is made against the measured error rather than a fixed
        // relative band.
        let (mut vol, mut var_vol) = (0.0, 0.0);
        for j in 0..multi.channels().len() {
            let (mean_j, var_j) = mc_estimate_channel(&multi, j, 0x5913, 51 + j as u64, n, |_| 1.0);
            vol += mean_j;
            var_vol += var_j;
        }
        let err_vol = (var_vol / n as f64).sqrt();
        let analytic = massless_volume(sqrt_s, 3);
        eprintln!("channel-split V₃ = {vol:.6e} ± {err_vol:.2e}, analytic {analytic:.6e}");
        assert!(
            (vol - analytic).abs() < 5.0 * err_vol,
            "channel-split V₃ {vol:.6e} ± {err_vol:.2e} vs analytic {analytic:.6e}"
        );
    }

    /// A channel-frozen draw produces the channel's own momenta, and its weight is
    /// the mixture weight scaled by that channel's `αⱼ` — the identity the split
    /// estimator rests on.
    #[test]
    fn sample_channel_weight_is_alpha_times_mixture_weight() {
        let mut multi = two_peak_combiner(500.0);
        multi.set_alphas(vec![0.25, 0.75]);
        let mut s = SubStream::from_stream(0x5914, 61);
        for _ in 0..500 {
            let u = s.uniforms::<f64>(multi.channel_ndim());
            for j in 0..multi.channels().len() {
                let pt = multi.sample_channel(j, &u);
                let direct = multi.channels()[j].sample(&u);
                for (a, b) in pt.momenta.iter().zip(&direct.momenta) {
                    assert_eq!(a.e(), b.e());
                    assert_eq!(a.pz(), b.pz());
                }
                let mixture_weight = 1.0 / multi.density(&pt.momenta);
                let expected = multi.alphas()[j] * mixture_weight;
                assert!(
                    (pt.weight - expected).abs() <= 1e-12 * expected,
                    "channel {j}: weight {} != αⱼ·(1/g) {expected}",
                    pt.weight
                );
            }
        }
    }

    // ── α-adaptation (survey → refine of the channel mixture) ─────────────────

    /// The two-channel combiner for the asymmetric multi-peak process: channel A
    /// resonates on the `{0,1}` pair, channel B on `{1,2}`, both on the Z pole. A
    /// fresh set of boxed channels each call (the boxed channels are not `Clone`).
    fn two_peak_combiner(sqrt_s: f64) -> MultiChannel<f64> {
        let masses = vec![0.0; 3];
        let a = DiagramChannel::from_topology_resonant(
            sqrt_s,
            masses.clone(),
            &[(vec![0, 1], Some(z_res()))],
        );
        let b =
            DiagramChannel::from_topology_resonant(sqrt_s, masses, &[(vec![1, 2], Some(z_res()))]);
        MultiChannel::uniform(vec![Box::new(a), Box::new(b)])
    }

    /// The survey→refine loop drives α toward the variance-minimising mixture: the
    /// trajectory converges, the per-channel variance shares `Wⱼ` equalise, and on
    /// an integrand whose `{0,1}` peak is weighted four times heavier the weight
    /// shifts onto the channel that covers it. A wrong reallocation would neither
    /// converge nor equalise the shares.
    #[test]
    fn alpha_adaptation_converges_and_tracks_variance_share() {
        let (m2, mg) = (M_Z * M_Z, M_Z * G_Z);
        let bw = move |s: f64| 1.0 / ((s - m2).powi(2) + mg * mg);
        // Peak on the {0,1} pair (channel A) four times heavier than the {1,2}
        // peak (channel B): the optimum must shift weight onto A.
        let f = move |p: &[LorentzVector<f64>]| 4.0 * bw(s_pair(p, 0, 1)) + bw(s_pair(p, 1, 2));

        let mut multi = two_peak_combiner(500.0);
        let report = multi.adapt_alphas(|p, _| f(p), 0xA1FA, 61, 40_000, 10, 0.5);

        let traj = &report.trajectory;
        eprintln!("α trajectory:");
        for (k, a) in traj.iter().enumerate() {
            eprintln!("  iter {k}: [{:.4}, {:.4}]", a[0], a[1]);
        }
        let (wa, wb) = (report.variance_shares[0], report.variance_shares[1]);
        eprintln!("final variance shares W = [{wa:.4e}, {wb:.4e}]");

        // Converged: the last two α vectors barely move.
        let last = traj.last().unwrap();
        let prev = &traj[traj.len() - 2];
        let step = (last[0] - prev[0]).abs() + (last[1] - prev[1]).abs();
        assert!(step < 1e-2, "α not converged: last step {step:.3e}");

        // The variance-minimising fixed point equalises the per-channel shares.
        let share_gap = (wa - wb).abs() / wa.max(wb);
        assert!(
            share_gap < 0.15,
            "variance shares not equalised: W = [{wa:.4e}, {wb:.4e}], gap {share_gap:.3}"
        );

        // Weight shifted onto the channel covering the heavier peak.
        assert!(
            last[0] > 0.55 && last[0] > last[1],
            "α did not shift toward the heavier {{0,1}} peak: {last:?}"
        );
    }

    /// The figure of merit: on the asymmetric multi-peak process the adapted-α
    /// combiner (L5) has strictly lower per-point estimator variance than the
    /// fixed uniform-α combiner (L4) at the same `N`. Since α-adaptation leaves the
    /// per-point cost unchanged (same channels evaluated), the variance ratio *is*
    /// the variance×CPU improvement at fixed precision; the survey is a one-off
    /// pre-conditioning cost, reported separately. Both estimators agree on the
    /// integral.
    ///
    /// This integrand is a linear combination of the two channels' Breit–Wigner
    /// shapes, so the variance-matched α approaches the zero-variance
    /// importance-sampling optimum and the win is large (best case); a real `|M|²`
    /// with continuum and interference is only approximately channel-shaped, so the
    /// practical win is smaller. The load-bearing claim the test pins is the
    /// *direction and strict inequality*, not the magnitude.
    #[test]
    fn alpha_adaptation_beats_uniform_variance_cpu() {
        use std::time::Instant;

        let (m2, mg) = (M_Z * M_Z, M_Z * G_Z);
        let bw = move |s: f64| 1.0 / ((s - m2).powi(2) + mg * mg);
        let f = move |p: &[LorentzVector<f64>]| 4.0 * bw(s_pair(p, 0, 1)) + bw(s_pair(p, 1, 2));

        let uniform = two_peak_combiner(500.0);

        let mut adapted = two_peak_combiner(500.0);
        let t_survey = Instant::now();
        let report = adapted.adapt_alphas(|p, _| f(p), 0xA2FA, 71, 40_000, 10, 0.5);
        let survey_ns = t_survey.elapsed().as_secs_f64() * 1e9;
        let survey_pts = 40_000.0 * report.trajectory.len().saturating_sub(1) as f64;

        let n = 400_000;
        let t_u = Instant::now();
        let (mean_u, var_u) = mc_estimate(&uniform, 0xB1A5, 73, n, f);
        let ns_u = t_u.elapsed().as_secs_f64() * 1e9 / n as f64;
        let t_a = Instant::now();
        let (mean_a, var_a) = mc_estimate(&adapted, 0xB1A6, 75, n, f);
        let ns_a = t_a.elapsed().as_secs_f64() * 1e9 / n as f64;

        eprintln!(
            "variance×CPU (L4 uniform vs L5 adapted): \
             uniform σ={mean_u:.6e} var={var_u:.3e} ({ns_u:.0} ns/pt) | \
             adapted σ={mean_a:.6e} var={var_a:.3e} ({ns_a:.0} ns/pt) | \
             var ratio {:.2}× | survey {survey_pts:.0} pts in {survey_ns:.2e} ns \
             (α = {:?})",
            var_u / var_a,
            adapted.alphas(),
        );

        // Unbiased: adapted α integrates to the same σ as uniform α.
        let err = ((var_u + var_a) / n as f64).sqrt();
        assert!(
            (mean_u - mean_a).abs() < 6.0 * err,
            "adapted σ {mean_a:.6e} disagrees with uniform σ {mean_u:.6e} (err {err:.2e})"
        );
        // The deliverable: strictly lower variance at fixed N.
        assert!(
            var_a < var_u,
            "adapted variance {var_a:.3e} not below uniform variance {var_u:.3e}"
        );
    }

    /// α-adaptation reshapes variance, not the integral: the adapted-α combiner
    /// reproduces the analytic massless volume `V₃` on a flat integrand and agrees
    /// with the uniform-α combiner on the resonant integral, both within MC error.
    #[test]
    fn alpha_adaptation_preserves_integral() {
        let (m2, mg) = (M_Z * M_Z, M_Z * G_Z);
        let bw = move |s: f64| 1.0 / ((s - m2).powi(2) + mg * mg);
        let f = move |p: &[LorentzVector<f64>]| 4.0 * bw(s_pair(p, 0, 1)) + bw(s_pair(p, 1, 2));
        let sqrt_s = 500.0;

        let mut adapted = two_peak_combiner(sqrt_s);
        adapted.adapt_alphas(|p, _| f(p), 0xA3FA, 81, 40_000, 8, 0.5);

        // Volume neutrality: the adapted mixture still integrates dΦ₃ to V₃.
        let n = 600_000;
        let (vol, var_vol) = mc_estimate(&adapted, 0xC0DE, 83, n, |_| 1.0);
        let err_vol = (var_vol / n as f64).sqrt();
        let analytic = massless_volume(sqrt_s, 3);
        eprintln!(
            "adapted V₃ = {vol:.6e} ± {err_vol:.2e}, analytic {analytic:.6e}, diff {:.2e}",
            (vol - analytic).abs()
        );
        assert!(
            (vol - analytic).abs() < 5.0 * err_vol,
            "adapted-α combiner biased the volume: {vol:.6e} vs {analytic:.6e}"
        );

        // Integral neutrality: adapted and uniform agree on the resonant integrand.
        let uniform = two_peak_combiner(sqrt_s);
        let (mean_a, var_a) = mc_estimate(&adapted, 0xC0D1, 85, n, f);
        let (mean_u, var_u) = mc_estimate(&uniform, 0xC0D2, 87, n, f);
        let err = ((var_a + var_u) / n as f64).sqrt();
        assert!(
            (mean_a - mean_u).abs() < 6.0 * err,
            "adapted σ {mean_a:.6e} disagrees with uniform σ {mean_u:.6e} (err {err:.2e})"
        );
    }

    /// note-07 conflicting-resonance hazard: two nearby timelike poles on the *same*
    /// invariant `s₀₁` — a Z at `M_Z` and a heavier pole at 140 GeV, widths broad
    /// enough that their tails overlap. Each pole needs its own Breit–Wigner
    /// channel; the combiner must resolve *both* and sum their densities correctly
    /// in the overlap valley. Firing content: (1) the sampled double-peak line shape
    /// matches the analytic `(ŝ−s)·[BW₁+BW₂]` (a wrong combined density distorts the
    /// valley), and (2) a single channel mapping only the first pole starves the
    /// second peak of samples — dropping the conflicting channel is visible as a
    /// coverage collapse there.
    #[test]
    fn overlapping_resonances_double_peak_resolved() {
        let sqrt_s = 500.0;
        let s_hat = sqrt_s * sqrt_s;
        let masses = vec![0.0; 3];
        let (m1, g1) = (M_Z, 5.0_f64);
        let (m2r, g2) = (140.0_f64, 8.0_f64);
        let (m1sq, mg1) = (m1 * m1, m1 * g1);
        let (m2sq, mg2) = (m2r * m2r, m2r * g2);
        let bw1 = move |s: f64| 1.0 / ((s - m1sq).powi(2) + mg1 * mg1);
        let bw2 = move |s: f64| 1.0 / ((s - m2sq).powi(2) + mg2 * mg2);
        let f = move |p: &[LorentzVector<f64>]| {
            let s = s_pair(p, 0, 1);
            bw1(s) + bw2(s)
        };

        // Two channels, each Breit–Wigner-mapping the {0,1} invariant at its own pole.
        let ch1 = DiagramChannel::from_topology_resonant(
            sqrt_s,
            masses.clone(),
            &[(
                vec![0, 1],
                Some(Resonance {
                    mass: m1,
                    width: g1,
                }),
            )],
        );
        let ch2 = DiagramChannel::from_topology_resonant(
            sqrt_s,
            masses.clone(),
            &[(
                vec![0, 1],
                Some(Resonance {
                    mass: m2r,
                    width: g2,
                }),
            )],
        );
        let combiner = MultiChannel::uniform(vec![Box::new(ch1), Box::new(ch2)]);

        // Analytic antiderivative of `(ŝ−s)·BW(s; m², mΓ)`.
        let anti_one = move |s: f64, msq: f64, mg: f64| {
            (s_hat - msq) / mg * ((s - msq) / mg).atan() - 0.5 * ((s - msq).powi(2) + mg * mg).ln()
        };
        let anti = move |s: f64| anti_one(s, m1sq, mg1) + anti_one(s, m2sq, mg2);

        // Histogram the combiner's `weight·f` across a window spanning both peaks.
        let (win_lo, win_hi) = (m1sq - 40.0 * mg1, m2sq + 40.0 * mg2);
        let nbins = 30usize;
        let bin_w = (win_hi - win_lo) / nbins as f64;
        let mut hist = vec![0.0_f64; nbins];
        let mut hist_sq = vec![0.0_f64; nbins];
        let mut count = vec![0usize; nbins];

        // Coverage counters within a few widths of the second (conflicting) pole.
        let peak2_lo = m2sq - 3.0 * mg2;
        let peak2_hi = m2sq + 3.0 * mg2;
        let mut combiner_in_peak2 = 0usize;

        let n = 2_000_000;
        let mut stream = SubStream::from_stream(0x0FF5, 91);
        for _ in 0..n {
            let u = stream.uniforms::<f64>(combiner.ndim());
            let pt = combiner.sample(&u);
            let s = s_pair(&pt.momenta, 0, 1);
            if (peak2_lo..peak2_hi).contains(&s) {
                combiner_in_peak2 += 1;
            }
            if s < win_lo || s >= win_hi {
                continue;
            }
            let k = ((s - win_lo) / bin_w) as usize;
            let v = pt.weight * f(&pt.momenta);
            hist[k] += v;
            hist_sq[k] += v * v;
            count[k] += 1;
        }

        let (mut mc, mut mc_err, mut an) = (Vec::new(), Vec::new(), Vec::new());
        for k in 0..nbins {
            if count[k] < 200 {
                continue;
            }
            let mean = hist[k] / n as f64;
            let err = ((hist_sq[k] / n as f64 - mean * mean).max(0.0) / n as f64).sqrt();
            let lo = win_lo + k as f64 * bin_w;
            mc.push(mean);
            mc_err.push(err);
            an.push(anti(lo + bin_w) - anti(lo));
        }
        assert!(mc.len() >= 16, "too few populated bins: {}", mc.len());

        let (s_mc, s_an): (f64, f64) = (mc.iter().sum(), an.iter().sum());
        let mut chi2 = 0.0;
        for i in 0..mc.len() {
            let e = (mc_err[i] / s_mc).max(1e-12);
            chi2 += ((mc[i] / s_mc - an[i] / s_an) / e).powi(2);
        }
        let chi2_dof = chi2 / mc.len() as f64;
        eprintln!(
            "double-peak line shape: {} bins, χ²/dof = {:.2}",
            mc.len(),
            chi2_dof
        );
        assert!(
            chi2_dof < 3.0,
            "combiner double-peak line shape departs from analytic: χ²/dof = {chi2_dof:.2}"
        );

        // Firing: a single channel mapping only the first pole starves the second
        // peak. Its coverage there collapses relative to the two-channel combiner.
        let single = DiagramChannel::from_topology_resonant(
            sqrt_s,
            masses.clone(),
            &[(
                vec![0, 1],
                Some(Resonance {
                    mass: m1,
                    width: g1,
                }),
            )],
        );
        let mut sstream = SubStream::from_stream(0x0FF6, 93);
        let mut single_in_peak2 = 0usize;
        for _ in 0..n {
            let u = sstream.uniforms::<f64>(single.ndim());
            let pt = single.sample(&u);
            let s = s_pair(&pt.momenta, 0, 1);
            if (peak2_lo..peak2_hi).contains(&s) {
                single_in_peak2 += 1;
            }
        }
        eprintln!(
            "second-pole coverage: combiner {combiner_in_peak2} vs single-channel {single_in_peak2}"
        );
        assert!(
            combiner_in_peak2 > 20 * single_in_peak2.max(1),
            "dropping the second channel did not collapse coverage of the conflicting \
             pole: combiner {combiner_in_peak2} vs single {single_in_peak2}"
        );
    }

    /// α-adaptation composes with VEGAS: the adapted combiner is handed to a VEGAS
    /// grid as its integrand map (α frozen, VEGAS refining the per-channel unit
    /// hypercube). The composed integral agrees with the direct combiner MC estimate
    /// within the combined error — the two adaptations compose without bias.
    #[test]
    fn alpha_adapted_combiner_composes_with_vegas() {
        use crate::vegas::VegasGrid;
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        let (m2, mg) = (M_Z * M_Z, M_Z * G_Z);
        let bw = move |s: f64| 1.0 / ((s - m2).powi(2) + mg * mg);
        let f = move |p: &[LorentzVector<f64>]| 4.0 * bw(s_pair(p, 0, 1)) + bw(s_pair(p, 1, 2));

        let mut multi = two_peak_combiner(500.0);
        multi.adapt_alphas(|p, _| f(p), 0xA4FA, 101, 40_000, 8, 0.5);

        // Direct MC estimate from the adapted combiner, as the reference integral.
        let n = 400_000;
        let (mean_mc, var_mc) = mc_estimate(&multi, 0xD0E1, 103, n, f);
        let err_mc = (var_mc / n as f64).sqrt();

        // VEGAS refines the per-channel hypercube on top of the frozen mixture.
        let mut grid = VegasGrid::new(multi.ndim(), 40, 1.5);
        let mut rng = StdRng::seed_from_u64(0x5EEDBEEF);
        let integrand = |u: &[f64]| {
            let pt = multi.sample(u);
            pt.weight * f(&pt.momenta)
        };
        let res = grid.adapt(integrand, 40_000, 6, &mut rng);
        eprintln!(
            "VEGAS∘α-adapt σ = {:.6e} ± {:.2e} (χ²/dof {:.2}); combiner MC {mean_mc:.6e} ± {err_mc:.2e}",
            res.integral, res.std_dev, res.chi2_per_dof
        );
        let err = (err_mc * err_mc + res.std_dev * res.std_dev).sqrt();
        assert!(
            (res.integral - mean_mc).abs() < 6.0 * err,
            "VEGAS∘α-adapt σ {:.6e} disagrees with combiner MC {mean_mc:.6e} (err {err:.2e})",
            res.integral
        );
    }
}
