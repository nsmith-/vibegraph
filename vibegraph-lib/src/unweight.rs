//! Accept/reject unweighting of a converged multichannel integration.
//!
//! The integral is estimated channel by channel — one VEGAS grid per channel,
//! each trained on `∫ dΦ f·αⱼgⱼ/g` with the channel frozen — so unweighting is
//! done channel by channel too: a trial draws a channel, draws a point against
//! that channel's frozen grid, and is accepted with probability `wⱼ(x)/w_maxⱼ`.
//!
//! # Why the channel is drawn `∝ w_maxⱼ`
//!
//! The kept events must be distributed across channels `∝ σⱼ`, since that is what
//! the physical cross section is decomposed into. A trial in channel `j` is kept
//! with mean probability `σⱼ/w_maxⱼ`, so drawing the channel with probability
//! `qⱼ` keeps events at a rate `∝ qⱼ·σⱼ/w_maxⱼ` — which is `∝ σⱼ` exactly when
//! `qⱼ ∝ w_maxⱼ`. Any other choice needs a compensating per-event weight and so
//! stops being an unweighted sample; drawing `∝ σⱼ`, in particular, over-populates
//! the channels whose maximum is small relative to their integral. The overall
//! acceptance is then
//!
//! ```text
//! Σⱼ qⱼ·σⱼ/w_maxⱼ = (Σⱼ σⱼ) / (Σⱼ w_maxⱼ) = σ / Σⱼ w_maxⱼ
//! ```
//!
//! which is also the largest acceptance any channel-selection rule can reach, and
//! is what makes the largest channel's share of `Σⱼ w_maxⱼ` — not the channel
//! count — the predictor of what splitting the integral buys.
//!
//! # `w_max` is an extremum estimate, so overweights are expected
//!
//! Each `w_maxⱼ` is the largest weight a finite frozen scan on channel `j`'s grid
//! happened to see, and the maximum of a finite sample is biased low. Points above
//! it are therefore not an error condition: they are kept with a weight `> 1`,
//! which leaves the estimator unbiased, and are counted two ways —
//! [`overweight_fraction`](UnweightStats::overweight_fraction) and
//! [`overweight_weight_share`](UnweightStats::overweight_weight_share). The share
//! is the load-bearing one: a handful of points carrying a large part of the cross
//! section is the classic silent unweighting failure, and a rate alone cannot see
//! it.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::phasespace::rng::{SubStream, SCALE_DRAW_STREAM_BASE};
use crate::select::select_index;
use crate::vegas::VegasGrid;

/// The RNG stream the per-channel weight scans run on, offset by channel index so
/// each channel's scan is independent of the others and of the event generation.
const SCAN_STREAM_BASE: u64 = 0x0057_4D41;

/// How many points the frozen scan spends estimating one channel's maximum.
///
/// The scan's budget is not the integration's. `w_max` is an extremum estimate,
/// and how many draws an extremum needs is set by the tail of the weight
/// distribution over the channel's own grid, not by how much of the cross section
/// the channel carries — so a budget that gives a good `σ` says nothing about how
/// good the maxima are.
///
/// What a larger budget buys is a trade, not a convergence. Every extra draw can
/// only raise a maximum, which moves cross section from above `w_max` to below it
/// — fewer and smaller overweight events — while raising `Σⱼ w_maxⱼ` and so
/// lowering the acceptance `σ / Σⱼ w_maxⱼ` that sets what an event costs. On the
/// 24-channel `p p → ℓ⁺ℓ⁻ j` grids the two move together over more than two
/// decades of budget: `Σⱼ w_maxⱼ` grows as `n^0.51` from 10³ to 2.6·10⁵ draws per
/// channel with no plateau, which is the signature of a Pareto weight tail of
/// index ≈ 2, and the share of `σ` above the maxima falls only as `n^-0.46`. There
/// is no budget at which the maxima settle; there is a curve, and a budget picks a
/// point on it.
///
/// The two variants choose the *allocation* across channels rather than the point
/// on the curve. [`PerChannel`](Self::PerChannel) costs `channels × points` and is
/// a function of the decomposition alone;
/// [`IntegrationShare`](Self::IntegrationShare) gives a channel holding a
/// per-mille of `σ` a per-mille of the points, which starves the narrow channels
/// whose maxima are hardest to find. On those same grids the two land on the same
/// curve to within the scan's own seed-to-seed spread (±20% on `Σⱼ w_maxⱼ` over
/// five seeds), so the allocation is the much weaker lever of the two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanBudget {
    /// Every channel scans on the same number of points, whatever the integration
    /// budget was and whatever share of it the channel received.
    PerChannel(usize),
    /// Each channel scans on the points the integration spent on it per iteration.
    IntegrationShare,
}

impl ScanBudget {
    /// The draws a channel gets, given the `channel_neval` the integration spent
    /// on it per iteration.
    pub fn draws_for(self, channel_neval: usize) -> usize {
        match self {
            ScanBudget::PerChannel(n) => n,
            ScanBudget::IntegrationShare => channel_neval,
        }
    }

    /// What the scan will cost in integrand evaluations, over channels whose
    /// per-iteration integration budgets are `channel_nevals`.
    pub fn total_draws(self, channel_nevals: impl IntoIterator<Item = usize>) -> usize {
        channel_nevals.into_iter().map(|n| self.draws_for(n)).sum()
    }
}

/// An integrand whose integral is split into channels, each sampled over its own
/// coordinates with the channel frozen.
///
/// This is the seam the unweighting pass needs and nothing more: it never sees
/// momenta, cuts or matrix elements, so the same accept/reject loop drives any
/// channel decomposition (and a single-channel integrand is the one-term case).
pub trait ChannelIntegrand {
    /// The number of channels the integral is split across.
    fn channel_count(&self) -> usize;

    /// The dimension of one channel's grid — the channel being frozen, this
    /// excludes any channel-selection coordinate.
    fn channel_grid_ndim(&self) -> usize;

    /// Uniforms the integrand consumes *after* its channel's grid coordinates,
    /// which the grid therefore does not adapt over. Zero for an integrand whose
    /// value is a function of the map's coordinates alone.
    ///
    /// The pass fills them from a stream of its own and carries them in the
    /// accepted point, so a reconstruction at the same coordinates reproduces the
    /// value the trial was accepted on — including whatever the integrand did
    /// with them.
    fn scale_draw_ndim(&self) -> usize {
        0
    }

    /// The `channel`-th term's integrand at
    /// `u ∈ [0,1]^(channel_grid_ndim + scale_draw_ndim)`, weighted by that
    /// channel's `αⱼ`, so the terms sum to the full integral. Points the cuts
    /// reject return exactly `0.0`.
    fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64;
}

/// What a frozen scan of one channel's grid found.
#[derive(Debug, Clone)]
pub struct ChannelScan {
    /// The largest weight seen, in the integrand's own units. Zero when the scan
    /// found no point passing the cuts.
    pub w_max: f64,
    /// Points drawn.
    pub draws: usize,
    /// Points with a non-zero weight — a channel whose grid mostly lands outside
    /// the cuts shows up here.
    pub nonzero: usize,
    /// The scan's own mean weight: an independent (and much cruder) estimate of
    /// this channel's `σⱼ`, useful for spotting a channel whose banked term and
    /// whose grid disagree.
    pub mean: f64,
}

/// Running counts of an accept/reject pass.
///
/// Weights are counted as the dimensionless ratio `r = w/w_maxⱼ` of the channel
/// the trial was taken in, which is what makes trials in different channels
/// comparable: with the channel drawn `∝ w_maxⱼ`, `E[r] = σ/Σⱼ w_maxⱼ`, so sums of
/// `r` are cross sections in units of `Σⱼ w_maxⱼ`.
#[derive(Debug, Clone, Default)]
pub struct UnweightStats {
    pub trials: u64,
    pub accepted: u64,
    /// Trials whose weight was exactly zero — outside the cuts, or a vanishing
    /// matrix element.
    pub vanishing: u64,
    /// `Σ r` over every trial.
    pub ratio_sum: f64,
    /// `Σ max(1, r)` over the accepted trials — the event weights the sample
    /// carries.
    pub event_weight_sum: f64,
    /// Trials with `r > 1`.
    pub overweight: u64,
    /// `Σ r` restricted to those trials.
    pub overweight_ratio_sum: f64,
    /// `Σ (r − 1)` over them: the part of the cross section that would be lost by
    /// truncating events at `w_max` instead of keeping them overweight.
    pub excess_sum: f64,
    /// The largest `r` seen, i.e. how far past its channel's maximum the pass got.
    pub ratio_max: f64,
}

impl UnweightStats {
    /// Accepted events per trial.
    pub fn efficiency(&self) -> f64 {
        ratio(self.accepted as f64, self.trials as f64)
    }

    /// Trials that exceeded their channel's maximum, as a fraction of all trials.
    pub fn overweight_fraction(&self) -> f64 {
        ratio(self.overweight as f64, self.trials as f64)
    }

    /// The share of the cross section carried by those trials.
    ///
    /// This is the number that reveals a silent overweight tail: a rate can be
    /// negligible while a few points carry a large fraction of the integral.
    pub fn overweight_weight_share(&self) -> f64 {
        ratio(self.overweight_ratio_sum, self.ratio_sum)
    }

    /// The share of the cross section that lives *above* `w_max` — what capping
    /// event weights at 1 would discard.
    pub fn excess_share(&self) -> f64 {
        ratio(self.excess_sum, self.ratio_sum)
    }

    /// The mean weight of a kept event; `1` when no trial ever went overweight.
    pub fn mean_event_weight(&self) -> f64 {
        ratio(self.event_weight_sum, self.accepted as f64)
    }
}

fn ratio(num: f64, den: f64) -> f64 {
    if den > 0.0 {
        num / den
    } else {
        0.0
    }
}

/// One accepted point: the channel it was drawn in, the grid coordinates it was
/// drawn at, and the weight it carries.
///
/// The coordinates are what regenerates the point's momenta, at the same `u` and
/// so with the same weight the trial was accepted on. The weight is `1` for an
/// ordinary event and `w/w_max > 1` for one above its channel's maximum.
#[derive(Debug, Clone)]
pub struct AcceptedPoint {
    pub channel: usize,
    pub u: Vec<f64>,
    pub weight: f64,
}

/// One channel's frozen grid and the maximum weight the accept/reject pass
/// normalises it against.
#[derive(Debug, Clone)]
struct UnweightChannel {
    grid: VegasGrid,
    w_max: f64,
    scan: ChannelScan,
}

/// Accept/reject event generation over frozen per-channel VEGAS grids.
///
/// Built by [`scan`](Self::scan), which estimates every channel's maximum weight,
/// then driven one trial at a time by [`trial`](Self::trial) or one event at a
/// time by [`next_event`](Self::next_event). The statistics accumulate across
/// calls and are read from [`stats`](Self::stats).
#[derive(Debug, Clone)]
pub struct Unweighter {
    channels: Vec<UnweightChannel>,
    /// The channel-selection weights, `∝ w_maxⱼ`.
    select_weights: Vec<f64>,
    total_w_max: f64,
    stats: UnweightStats,
    /// Reused coordinate buffer for a trial draw: the drawn channel's grid
    /// coordinates followed by the integrand's trailing uniforms.
    u: Vec<f64>,
    /// How many of `u`'s trailing coordinates the grid does not supply.
    scale_ndim: usize,
    /// One stream per channel for those trailing uniforms, opened at the scan and
    /// carried on into the trials, so no trailing draw is ever replayed and the
    /// caller's own generator supplies none of them.
    scale_draw: Vec<SubStream>,
}

impl Unweighter {
    /// Estimate each channel's maximum weight by a frozen scan on its own grid.
    ///
    /// `channels` supplies each channel's trained grid together with the number of
    /// scan draws to spend on it; [`ScanBudget`] is the vocabulary for choosing
    /// those counts and carries what a budget buys. Each channel scans on its own
    /// RNG stream off `seed`, so a channel's estimate does not depend on how many
    /// draws its neighbours got.
    ///
    /// A channel whose scan finds nothing (every draw cut away) gets `w_max = 0`
    /// and is never selected; [`empty_channels`](Self::empty_channels) reports it,
    /// because its term is then missing from the generated sample even though it is
    /// present in the banked cross section.
    pub fn scan<'g, I: ChannelIntegrand>(
        integrand: &I,
        channels: impl IntoIterator<Item = (&'g VegasGrid, usize)>,
        seed: u64,
    ) -> Self {
        let ndim = integrand.channel_grid_ndim();
        let scale_ndim = integrand.scale_draw_ndim();
        let mut built = Vec::with_capacity(integrand.channel_count());
        let mut u = vec![0.0; ndim + scale_ndim];
        let mut scale_draw: Vec<SubStream> = Vec::with_capacity(integrand.channel_count());
        for (j, (grid, draws)) in channels.into_iter().enumerate() {
            assert_eq!(
                grid.ndim(),
                ndim,
                "channel {j}'s grid is over {} coordinates, the integrand's channels over {ndim}",
                grid.ndim()
            );
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            rng.set_stream(SCAN_STREAM_BASE + j as u64);
            let mut trailing = SubStream::from_stream(seed, SCALE_DRAW_STREAM_BASE + j as u64);
            let mut w_max = 0.0f64;
            let mut sum = 0.0f64;
            let mut nonzero = 0usize;
            for _ in 0..draws {
                let jac = grid.draw(&mut rng, &mut u[..ndim]);
                trailing.fill_uniforms(&mut u[ndim..]);
                let w = jac * integrand.value_in_channel(j, &u);
                if w > 0.0 {
                    nonzero += 1;
                    sum += w;
                    w_max = w_max.max(w);
                }
            }
            built.push(UnweightChannel {
                grid: grid.clone(),
                w_max,
                scan: ChannelScan {
                    w_max,
                    draws,
                    nonzero,
                    mean: ratio(sum, draws as f64),
                },
            });
            scale_draw.push(trailing);
        }
        assert_eq!(
            built.len(),
            integrand.channel_count(),
            "one grid per channel is required"
        );
        let select_weights: Vec<f64> = built.iter().map(|c| c.w_max).collect();
        let total_w_max: f64 = select_weights.iter().sum();
        assert!(
            total_w_max > 0.0 && total_w_max.is_finite(),
            "no channel produced a usable weight: nothing can be generated"
        );
        Unweighter {
            channels: built,
            select_weights,
            total_w_max,
            stats: UnweightStats::default(),
            u,
            scale_ndim,
            scale_draw,
        }
    }

    /// `Σⱼ w_maxⱼ` — the unit every accumulated weight ratio is measured in, and
    /// the denominator of the predicted efficiency `σ / Σⱼ w_maxⱼ`.
    pub fn total_w_max(&self) -> f64 {
        self.total_w_max
    }

    /// Per-channel maxima, in channel order.
    pub fn w_max(&self) -> Vec<f64> {
        self.channels.iter().map(|c| c.w_max).collect()
    }

    /// What the scan saw in each channel.
    pub fn scans(&self) -> Vec<&ChannelScan> {
        self.channels.iter().map(|c| &c.scan).collect()
    }

    /// The largest channel's share of `Σⱼ w_maxⱼ` — the predictor of how much a
    /// per-channel split buys over one global maximum. At `1` the mixture is
    /// effectively a single channel and there is nothing to gain.
    pub fn largest_channel_share(&self) -> f64 {
        let hi = self.channels.iter().map(|c| c.w_max).fold(0.0, f64::max);
        ratio(hi, self.total_w_max)
    }

    /// Channels whose scan found no point at all, and which are therefore never
    /// drawn from.
    pub fn empty_channels(&self) -> Vec<usize> {
        self.channels
            .iter()
            .enumerate()
            .filter(|(_, c)| c.scan.nonzero == 0)
            .map(|(j, _)| j)
            .collect()
    }

    /// The statistics accumulated so far.
    pub fn stats(&self) -> &UnweightStats {
        &self.stats
    }

    /// The cross section estimated from every trial, accepted or not — the plain
    /// weighted estimator over the same draws, in the integrand's own units.
    pub fn sigma_from_trials(&self) -> f64 {
        self.total_w_max * ratio(self.stats.ratio_sum, self.stats.trials as f64)
    }

    /// The cross section carried by the *kept* events alone: `Σⱼ w_maxⱼ` times the
    /// mean event weight per trial.
    ///
    /// This is what an unweighted sample is worth, and comparing it against the
    /// integration's own `σ` is the check that accept/reject preserved the
    /// normalisation. It has the same expectation as
    /// [`sigma_from_trials`](Self::sigma_from_trials) but a larger variance — the
    /// rejected trials are exactly the information unweighting throws away.
    pub fn sigma_from_events(&self) -> f64 {
        self.total_w_max * ratio(self.stats.event_weight_sum, self.stats.trials as f64)
    }

    /// Draw one trial: pick a channel `∝ w_maxⱼ`, draw a point on its grid, and
    /// accept it with probability `w/w_maxⱼ`. Returns the accepted point, or
    /// `None` when the trial was rejected.
    ///
    /// Three uniforms' worth of the caller's stream are consumed per trial in a
    /// fixed order — channel, point, acceptance — so a run is reproducible from
    /// the RNG alone. The integrand's trailing uniforms are *not* among them: they
    /// come off this pass's own per-channel streams, which is what leaves the
    /// caller's sequence identical to a run with no such coordinate.
    pub fn trial<I: ChannelIntegrand>(
        &mut self,
        integrand: &I,
        rng: &mut impl Rng,
    ) -> Option<AcceptedPoint> {
        let j = select_index(&self.select_weights, rng.random::<f64>())
            .expect("the summed maximum is positive, so some channel carries weight");
        let channel = &self.channels[j];
        let grid_ndim = self.u.len() - self.scale_ndim;
        let jac = channel.grid.draw(rng, &mut self.u[..grid_ndim]);
        self.scale_draw[j].fill_uniforms(&mut self.u[grid_ndim..]);
        let w = jac * integrand.value_in_channel(j, &self.u);
        let r = w / channel.w_max;
        let accept: f64 = rng.random();

        self.stats.trials += 1;
        if !(w > 0.0) {
            self.stats.vanishing += 1;
            return None;
        }
        self.stats.ratio_sum += r;
        self.stats.ratio_max = self.stats.ratio_max.max(r);
        if r > 1.0 {
            self.stats.overweight += 1;
            self.stats.overweight_ratio_sum += r;
            self.stats.excess_sum += r - 1.0;
        }
        if accept >= r {
            return None;
        }
        let weight = r.max(1.0);
        self.stats.accepted += 1;
        self.stats.event_weight_sum += weight;
        Some(AcceptedPoint {
            channel: j,
            u: self.u.clone(),
            weight,
        })
    }

    /// Trial until one point is accepted, giving up after `max_trials`.
    ///
    /// The budget is a guard against a pass that cannot accept anything (an
    /// integrand that has changed under the scan, say); a caller generating a
    /// fixed number of events should size it well above `1/efficiency`.
    pub fn next_event<I: ChannelIntegrand>(
        &mut self,
        integrand: &I,
        rng: &mut impl Rng,
        max_trials: usize,
    ) -> Option<AcceptedPoint> {
        (0..max_trials).find_map(|_| self.trial(integrand, rng))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::ChaCha8Rng;

    /// A closed-form channel decomposition: channel `j` is the density
    /// `(pⱼ+1)·x^pⱼ` on the unit interval scaled by `sⱼ`, so its term integrates to
    /// exactly `sⱼ` and its supremum over a flat grid is `sⱼ·(pⱼ+1)`.
    struct PowerChannels {
        sigma: Vec<f64>,
        power: Vec<f64>,
    }

    impl ChannelIntegrand for PowerChannels {
        fn channel_count(&self) -> usize {
            self.sigma.len()
        }
        fn channel_grid_ndim(&self) -> usize {
            1
        }
        fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
            let p = self.power[channel];
            self.sigma[channel] * (p + 1.0) * u[0].powf(p)
        }
    }

    fn flat_grids(n: usize) -> Vec<VegasGrid> {
        (0..n).map(|_| VegasGrid::new(1, 50, 0.0)).collect()
    }

    fn scan_of(integ: &PowerChannels, draws: usize, seed: u64) -> (Vec<VegasGrid>, Unweighter) {
        let grids = flat_grids(integ.channel_count());
        let uw = Unweighter::scan(integ, grids.iter().map(|g| (g, draws)), seed);
        (grids, uw)
    }

    /// [`ScanBudget::IntegrationShare`] must hand back exactly the count it was
    /// given: it is the spelling of the coupling, not a policy on top of it, and a
    /// caller that selects it gets the draws the integration allocated.
    #[test]
    fn the_integration_share_budget_is_the_integrations_own_count() {
        for n in [0usize, 1, 37, 12_345, 600_000] {
            assert_eq!(ScanBudget::IntegrationShare.draws_for(n), n);
            assert_eq!(ScanBudget::PerChannel(500).draws_for(n), 500);
        }
        let nevals = [10usize, 200, 3_000];
        assert_eq!(ScanBudget::IntegrationShare.total_draws(nevals), 3_210);
        assert_eq!(ScanBudget::PerChannel(500).total_draws(nevals), 1_500);
    }

    /// The mechanism that makes the scan's budget a question of its own: how many
    /// draws an extremum needs is set by the channel's weight distribution, not by
    /// its share of the cross section.
    ///
    /// Two channels with the same analytic supremum, `20`, reached from opposite
    /// ends — a wide one carrying essentially all of `σ` with a mild peak, and a
    /// narrow one carrying a thousandth of it with a very steep one. Under an
    /// allocation `∝ σⱼ` the narrow channel gets a thousandth of the points and
    /// its maximum stays far below the supremum; under a flat allocation of the
    /// *same total* it reaches it, and the wide channel, whose draws were halved
    /// to pay for it, barely notices — an extremum estimate improves
    /// logarithmically in the draw count.
    ///
    /// This is a statement about the allocation's reach, not about which
    /// allocation wins on a real decomposition: whether starved narrow channels
    /// matter there depends on how their maxima compare with the wide channels',
    /// and on `p p → ℓ⁺ℓ⁻ j` grids the two allocations measure the same.
    #[test]
    fn a_flat_budget_finds_a_narrow_channels_maximum_the_share_budget_misses() {
        let integ = PowerChannels {
            sigma: vec![10.0, 0.01],
            power: vec![1.0, 1999.0],
        };
        let supremum = [20.0, 20.0];
        let grids = flat_grids(2);
        // The share allocation over a 40 000-point budget, `αⱼ ∝ σⱼ`.
        let total = 40_000usize;
        let share = [
            (total as f64 * 10.0 / 10.01) as usize,
            (total as f64 * 0.01 / 10.01) as usize,
        ];
        let flat = [total / 2, total / 2];
        assert_eq!(
            ScanBudget::IntegrationShare.total_draws(share),
            share.iter().sum::<usize>()
        );

        let scan = |draws: [usize; 2]| {
            Unweighter::scan(
                &integ,
                grids.iter().zip(draws).map(|(g, n)| (g, n)),
                0x5CA7_B0D9,
            )
            .w_max()
        };
        let by_share = scan(share);
        let by_flat = scan(flat);

        // The narrow channel: starved to 39 draws by the share allocation, so its
        // steep peak is never approached; 20 000 flat draws reach it.
        assert!(
            by_share[1] / supremum[1] < 0.05,
            "the share allocation should leave the narrow channel far below its \
             supremum, got {:.4} of {}",
            by_share[1],
            supremum[1]
        );
        assert!(
            by_flat[1] / supremum[1] > 0.9,
            "a flat allocation should reach the narrow channel's peak, got {:.4} of {}",
            by_flat[1],
            supremum[1]
        );

        // The wide channel paid for it with half its draws and lost almost nothing.
        assert!(
            by_flat[0] / by_share[0] > 0.999,
            "halving the wide channel's scan cost it {:.4} of its maximum",
            by_flat[0] / by_share[0]
        );
    }

    #[test]
    fn scan_approaches_the_analytic_supremum() {
        let integ = PowerChannels {
            sigma: vec![1.0, 4.0],
            power: vec![0.0, 3.0],
        };
        let (_g, uw) = scan_of(&integ, 200_000, 7);
        let w = uw.w_max();
        // Channel 0 is flat: its supremum is reached at every point.
        assert!((w[0] - 1.0).abs() < 1e-12, "flat channel maximum {}", w[0]);
        // Channel 1 peaks at u = 1 with 4*4 = 16; a finite scan lands just below.
        assert!(w[1] < 16.0, "a finite scan cannot exceed the supremum");
        assert!(w[1] > 15.9, "200k draws should come close: {}", w[1]);
    }

    /// The accepted events must be distributed over channels in proportion to the
    /// channels' cross sections. This is the property that fixes the
    /// channel-selection rule to `∝ w_maxⱼ`: drawing `∝ σⱼ` instead leaves the
    /// integral right and this distribution wrong.
    #[test]
    fn kept_events_follow_the_channel_cross_sections() {
        let integ = PowerChannels {
            // Equal integrals, very different maxima — the case a σ-proportional
            // channel draw gets wrong by a factor of the maximum ratio.
            sigma: vec![1.0, 1.0],
            power: vec![0.0, 9.0],
        };
        let (_g, mut uw) = scan_of(&integ, 100_000, 11);
        let mut rng = ChaCha8Rng::seed_from_u64(4242);
        let mut counts = [0usize; 2];
        let n = 40_000;
        for _ in 0..n {
            let ev = uw
                .next_event(&integ, &mut rng, 100_000)
                .expect("an event within the trial budget");
            counts[ev.channel] += 1;
        }
        let f0 = counts[0] as f64 / n as f64;
        // Both channels carry half the cross section, so half the events each.
        let sigma = (0.5 * 0.5 / n as f64).sqrt();
        assert!(
            (f0 - 0.5).abs() < 4.0 * sigma,
            "channel-0 share {f0:.4} vs 0.5 (4σ = {:.4})",
            4.0 * sigma
        );
    }

    #[test]
    fn the_unweighted_sample_reproduces_the_integral() {
        let integ = PowerChannels {
            sigma: vec![2.0, 0.5, 3.0],
            power: vec![0.0, 2.0, 6.0],
        };
        let sigma: f64 = integ.sigma.iter().sum();
        let (_g, mut uw) = scan_of(&integ, 100_000, 3);
        let mut rng = ChaCha8Rng::seed_from_u64(9);
        for _ in 0..2_000_000 {
            uw.trial(&integ, &mut rng);
        }
        let from_events = uw.sigma_from_events();
        let from_trials = uw.sigma_from_trials();
        assert!(
            (from_events / sigma - 1.0).abs() < 0.02,
            "sigma from events {from_events:.5} vs {sigma:.5}"
        );
        assert!(
            (from_trials / sigma - 1.0).abs() < 0.02,
            "sigma from trials {from_trials:.5} vs {sigma:.5}"
        );
        // The predicted acceptance is the integral over the summed maxima.
        let predicted = sigma / uw.total_w_max();
        let observed = uw.stats().efficiency();
        assert!(
            (observed / predicted - 1.0).abs() < 0.05,
            "efficiency {observed:.4e} vs predicted {predicted:.4e}"
        );
    }

    /// A maximum deliberately estimated too low puts a measurable part of the
    /// integral above it. Both counters must see it, the estimator must survive it,
    /// and the weight share must be far larger than the rate — which is exactly the
    /// asymmetry that makes the rate alone an unsafe diagnostic.
    #[test]
    fn overweights_are_counted_and_leave_the_integral_unbiased() {
        let integ = PowerChannels {
            sigma: vec![1.0],
            power: vec![9.0],
        };
        // A short scan on a steeply peaked channel stops short of its supremum of 10,
        // leaving a tail of the integrand above the estimated maximum.
        let (_g, mut uw) = scan_of(&integ, 200, 5);
        assert!(
            uw.w_max()[0] < 10.0,
            "a finite scan cannot reach the supremum"
        );
        let mut rng = ChaCha8Rng::seed_from_u64(17);
        for _ in 0..400_000 {
            uw.trial(&integ, &mut rng);
        }
        let s = uw.stats();
        assert!(s.overweight > 0, "the undershoot must produce overweights");
        assert!(
            s.overweight_weight_share() > 4.0 * s.overweight_fraction(),
            "share {:.3e} vs rate {:.3e}",
            s.overweight_weight_share(),
            s.overweight_fraction()
        );
        assert!(s.excess_share() > 0.0 && s.excess_share() < s.overweight_weight_share());
        assert!(s.ratio_max > 1.0);
        assert!(s.mean_event_weight() > 1.0);
        assert!(
            (uw.sigma_from_events() - 1.0).abs() < 0.02,
            "overweight events keep the integral unbiased: {}",
            uw.sigma_from_events()
        );
    }

    /// Truncating overweights instead of keeping them would bias the integral low by
    /// exactly the excess share, which is the reason the excess is tracked at all.
    #[test]
    fn the_excess_share_is_what_truncation_would_cost() {
        let integ = PowerChannels {
            sigma: vec![1.0],
            power: vec![9.0],
        };
        let (_g, mut uw) = scan_of(&integ, 200, 5);
        let mut rng = ChaCha8Rng::seed_from_u64(17);
        for _ in 0..400_000 {
            uw.trial(&integ, &mut rng);
        }
        let s = uw.stats();
        let truncated = uw.total_w_max() * (s.ratio_sum - s.excess_sum) / s.trials as f64;
        let deficit = 1.0 - truncated / uw.sigma_from_trials();
        assert!(
            (deficit - s.excess_share()).abs() < 1e-12,
            "truncation deficit {deficit:.6e} vs excess share {:.6e}",
            s.excess_share()
        );
    }

    #[test]
    fn a_channel_the_scan_never_reached_is_reported() {
        struct OneLive;
        impl ChannelIntegrand for OneLive {
            fn channel_count(&self) -> usize {
                2
            }
            fn channel_grid_ndim(&self) -> usize {
                1
            }
            fn value_in_channel(&self, channel: usize, _u: &[f64]) -> f64 {
                if channel == 0 {
                    1.0
                } else {
                    0.0
                }
            }
        }
        let grids = flat_grids(2);
        let uw = Unweighter::scan(&OneLive, grids.iter().map(|g| (g, 1000)), 1);
        assert_eq!(uw.empty_channels(), vec![1]);
        assert_eq!(uw.w_max()[1], 0.0);
        assert!((uw.largest_channel_share() - 1.0).abs() < 1e-12);
    }
}
