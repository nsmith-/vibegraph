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
pub struct MultiChannel<F: Real> {
    channels: Vec<Box<dyn Channel<F>>>,
    alphas: Vec<F>,
    channel_ndim: usize,
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

    /// The combined sampling density `g(p) = Σⱼ αⱼ gⱼ(p)` at `momenta` — the
    /// reciprocal of the weight the combiner assigns to a point generated there.
    pub fn density(&self, momenta: &[LorentzVector<F>]) -> F {
        let mut g = F::zero();
        for (alpha, ch) in self.alphas.iter().zip(&self.channels) {
            g = g + *alpha * ch.density(momenta);
        }
        g
    }

    /// The channel `u0 ∈ [0,1)` selects, by cumulative selection weight.
    fn select(&self, u0: F) -> usize {
        let mut acc = F::zero();
        for (j, alpha) in self.alphas.iter().enumerate() {
            acc = acc + *alpha;
            if u0 < acc {
                return j;
            }
        }
        self.channels.len() - 1
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
        let idx = self.select(u[0]);
        let pt = self.channels[idx].sample(&u[1..]);
        // g ≥ α_idx · g_idx = α_idx / pt.weight > 0: the generating channel's own
        // positive density keeps the denominator off zero, so `1/g` is the exact
        // reciprocal the reciprocity contract requires.
        let g = self.density(&pt.momenta);
        debug_assert!(g > F::zero(), "combined density must be positive");
        PhaseSpacePoint {
            momenta: pt.momenta,
            weight: F::one() / g,
        }
    }
}

impl<F: Real> Combiner<F> for MultiChannel<F> {
    fn channels(&self) -> &[Box<dyn Channel<F>>] {
        &self.channels
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
        let cases: Vec<(f64, usize, Vec<Vec<Vec<usize>>>)> = vec![
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
}
