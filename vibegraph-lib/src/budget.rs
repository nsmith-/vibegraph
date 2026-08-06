//! How a multichannel integration's sample budget is split across its channels
//! and across its iterations, and when it is allowed to stop.
//!
//! The integral is a sum of per-channel terms, each sampled over its own grid
//! with the channel frozen — a **hard split**, deterministic point counts per
//! channel rather than a per-point channel draw. Two things then have to be
//! decided every iteration: how many points each channel gets, and whether to
//! run another iteration at all.
//!
//! # Allocation
//!
//! For a total-σ target the variance-minimising split of `N` points across terms
//! with per-point standard deviations `sⱼ` is Neyman's, `Nⱼ ∝ sⱼ` — where `sⱼ` is
//! the standard deviation of *this* estimator's term, which already carries the
//! channel's selection weight `αⱼ` (the mixture density puts it inside
//! [`ChannelIntegrand::value_in_channel`]). Written on the unweighted channel
//! integrand `hⱼ = f/g` the same rule reads `Nⱼ ∝ αⱼ · sd(hⱼ)`.
//!
//! [`BlockAllocation::ByAlpha`] instead spends `Nⱼ ∝ αⱼ`. That is not the naive
//! choice it looks like: the Kleiss–Pittau α-adaptation the combiner runs before
//! the grids are trained already sets `αⱼ` from a variance survey, so `αⱼ` is
//! itself most of a variance estimate. Measured on `p p > l+ l- j`, `αⱼ` spans a
//! factor ~100 across the 24 channels while the realised ratio `sⱼ/αⱼ` spans
//! only 2–4.5: the α split is already close to Neyman's, and re-deriving it from
//! the trained grids buys a factor 1.00–1.22 in variance at equal points.
//!
//! That number is not why [`BlockAllocation::Neyman`] is worth having, and the
//! difference between the two is the whole point of the stopping rule below —
//! see [`BlockAllocation::Neyman`].
//!
//! Both are floored at [`MIN_CHANNEL_NEVAL`]: allocation splits the spend, never
//! the coverage. A channel whose map is the only one with density on some
//! structure has to keep sampling it even when its variance estimate says it is
//! cheap, because a channel that stops covering its own region is how a
//! multichannel integral becomes confidently wrong.
//!
//! # Stopping
//!
//! [`Budget::Fixed`] is the reproducible mode: `neval × niter`, the same points
//! in the same order every run. [`Budget::Target`] iterates until the combined
//! estimate's relative uncertainty meets a target, subject to three
//! preconditions, each of which exists because a stop is a claim about an error
//! bar and error bars here are known to be optimistic:
//!
//! * **The combination must be unweighted.** Lepage's `1/σ²` weighting is biased
//!   by the correlation between an iteration's estimate and its own variance, and
//!   the bias comes with a *small* error bar — precisely the shape that makes a
//!   convergence test stop early on a wrong number. A `Target` budget refuses an
//!   [`IterationCombination::InverseVariance`] grid rather than reading its error.
//! * **A minimum iteration count**, so the χ²/dof below has degrees of freedom
//!   and the grid has been refined more than a couple of times.
//! * **An iteration-consistency scale factor.** The stopping test reads not the
//!   quoted error but the quoted error times `√max(1, χ²/dof)` per channel — the
//!   PDG scale-factor treatment, applied where the inconsistency lives. This is
//!   the precondition that does the most work. The per-point weight distribution
//!   of the σ-carrying channels of `p p > l+ l- j` has a Pareto tail index near
//!   2.1–2.4, i.e. it sits at the boundary where the variance exists at all, so
//!   empirical variances converge slowly and biased low; the iterations then
//!   scatter by more than their own quoted errors, and χ²/dof measures exactly
//!   that excess. Measured χ²/dof on that process at production budgets is 2–3.3,
//!   so the factor buys a 1.4–1.8× tighter error before a stop is granted, which
//!   is a 2–3× point cost — deliberately, since the alternative is stopping on an
//!   error bar that the seed-to-seed spread does not support.
//!
//! The χ² that factor comes from is formed over the iterations that *measured* a
//! variance. On a split wide enough to put most channels on
//! [`MIN_CHANNEL_NEVAL`], an allocation regularly loses every one of its points
//! to the cuts and comes back a constant zero, whose measured variance is exactly
//! zero — not a small error bar but no error bar at all. Dividing a residual by
//! it puts the channel's χ²/dof at ~1e250, and a control decision taken on that
//! number is a target nothing can satisfy. Such iterations stay in the integral
//! and in the quoted error, and the χ²/dof the run *reports* is still the one
//! they produce; what they are kept out of is the consistency test, which has
//! nothing to say about an iteration that quoted no error. See
//! [`ChannelHistory::stop_scale`].

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use tracing::{debug, info, info_span, warn};

use crate::hadronic::{combine_channels, ChannelIntegration, VEGAS_NBINS};
use crate::phasespace::rng::{SubStream, SCALE_DRAW_STREAM_BASE};
use crate::progress;
use crate::unweight::ChannelIntegrand;
use crate::vegas::{
    adapt_blocks_iteration, combine_iterations, BlockIteration, BlockPlan, IterationCombination,
    VegasGrid, VegasResult,
};

/// Floor on a channel's per-iteration evaluation count.
///
/// A channel whose selection weight rounds to nothing still gets a grid it can
/// refine and a term it can estimate. Coverage is not a thing an allocation rule
/// is allowed to trade away.
pub const MIN_CHANNEL_NEVAL: usize = 512;

/// How the per-iteration budget is split across channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlockAllocation {
    /// `Nⱼ = round(αⱼ · neval)`, floored — the split the channel weights already
    /// imply, fixed for the whole run.
    #[default]
    ByAlpha,
    /// Neyman: `Nⱼ ∝ sⱼ`, the running per-point standard deviation of channel
    /// `j`'s own term, floored, recomputed from every completed iteration.
    ///
    /// Re-splitting a *fixed* budget this way is worth almost nothing — a factor
    /// 1.00–1.22 in variance on `p p > l+ l- j`, 1.000 on the four channels of
    /// `p p > e+ e-` — because the α-survey has already done that work. What it
    /// is worth is measured against a *target*, where the channels it feeds are
    /// the ones whose iterations disagree with themselves: on `p p > l+ l- j` at
    /// a 0.179% target over 8 seeds it reaches the same accuracy in 2.94M
    /// evaluations against 6.41M, and the seed-to-seed spread of that spend
    /// narrows from 2.75M–12.4M to 1.97M–4.26M.
    ///
    /// The mechanism is the scale factor, not the split: a starved channel's
    /// χ²/dof is what widens the error the stopping rule reads, so feeding it
    /// relaxes the test by more than its own variance share ever could. Both
    /// arms agree on σ (0.14σ and 0.32σ from MadGraph's banked value).
    Neyman,
}

/// What an integration is asked to spend.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Budget {
    /// `neval` points per iteration split across the channels, `niter`
    /// iterations, and stop. Reproducible point-for-point from the seed.
    Fixed { neval: usize, niter: usize },
    /// Iterate until the scale-factor-corrected relative uncertainty of the
    /// combined estimate is at or below `target_rel`.
    Target {
        /// Relative uncertainty to reach, e.g. `3.6e-3` for 0.36%.
        target_rel: f64,
        /// Points per iteration, split across the channels as in `Fixed`.
        neval: usize,
        /// Iterations that must run before a stop is considered.
        min_iters: usize,
        /// Iterations after which the run stops whether or not it converged.
        max_iters: usize,
        /// Points the run will not exceed, converged or not. A bound rather than
        /// a threshold: an iteration that would breach it is not drawn, since
        /// what an iteration costs is set by the channel count and the
        /// [`MIN_CHANNEL_NEVAL`] floor as much as by `neval`, and a cap tested
        /// only afterwards is overshot by a whole one.
        max_points: u64,
    },
}

impl Budget {
    /// Points per iteration, before the per-channel floor.
    pub fn neval(&self) -> usize {
        match *self {
            Budget::Fixed { neval, .. } | Budget::Target { neval, .. } => neval,
        }
    }

    /// Iterations this budget can run at most.
    fn max_iters(&self) -> usize {
        match *self {
            Budget::Fixed { niter, .. } => niter,
            Budget::Target { max_iters, .. } => max_iters,
        }
    }
}

/// Why an integration stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// A [`Budget::Fixed`] run finished its iterations.
    Budget,
    /// The scale-factor-corrected relative uncertainty met the target.
    TargetMet,
    /// `max_iters` was reached with the target unmet.
    MaxIters,
    /// The target was unmet and another iteration would have breached
    /// `max_points`.
    MaxPoints,
    /// A [`StopSignal`] was raised: the run stopped at the first iteration
    /// boundary after the request, whatever its budget still allowed.
    Aborted,
}

impl StopReason {
    /// Whether the run reached the accuracy it was asked for. `true` for a fixed
    /// budget, which was asked for points rather than accuracy.
    pub fn converged(self) -> bool {
        matches!(self, StopReason::Budget | StopReason::TargetMet)
    }
}

/// A request, raisable from another thread, that a running integration stop early
/// and keep what it has.
///
/// The request is read once per iteration and nowhere else, so a signal that is
/// never raised changes neither the points an integration draws nor the order it
/// draws them in: the arithmetic of a run that is not stopped is the arithmetic of
/// a run that could not be. What a raised signal buys is that the grids and the
/// terms returned are those of the last *completed* iteration — an iteration
/// abandoned part-way would have sampled only some of its channels, and a term
/// combined over that is not an estimate of anything.
#[derive(Debug, Clone, Default)]
pub struct StopSignal(Arc<AtomicBool>);

impl StopSignal {
    /// A signal nobody has raised.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ask every integration reading this signal to stop after its current
    /// iteration.
    pub fn request(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Whether the request has been made.
    pub fn requested(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// What a completed integration spent and why it stopped.
#[derive(Debug, Clone)]
pub struct ConvergenceReport {
    pub stop: StopReason,
    /// Iterations run, warm-up included.
    pub iterations: usize,
    /// Iterations that entered the combination — those past the warm-up.
    ///
    /// `0` means the run holds no estimate at all: the terms it returns are
    /// combined over nothing and are not numbers. Only a run stopped early can
    /// end that way, since a budget always outlasts its own warm-up.
    pub kept_iterations: usize,
    /// Integrand evaluations over all channels and iterations.
    pub points: u64,
    /// Evaluations the last iteration spent over all channels.
    ///
    /// The price of one more iteration, and the number an iteration count has to
    /// be multiplied by to become a spend. It is not `neval`: the
    /// [`MIN_CHANNEL_NEVAL`] floor raises it above what the budget asked for
    /// whenever the channel count alone outweighs the request, and above roughly
    /// `neval / MIN_CHANNEL_NEVAL` channels it is the channel count, not the
    /// request, that decides what a run costs.
    pub points_per_iteration: usize,
    /// The target that was asked for, if any.
    pub target_rel: Option<f64>,
    /// `Δσ/σ` of the combined estimate as quoted.
    pub achieved_rel: f64,
    /// `Δσ/σ` after the per-channel `√max(1, χ²/dof)` scale factor — the number
    /// the stopping test compares against `target_rel`.
    pub scaled_rel: f64,
    /// Points each channel drew over the whole run, in channel order.
    pub channel_points: Vec<u64>,
    /// The smallest per-iteration allocation any channel received in any
    /// iteration — the floor guardrail as realised rather than as intended. A
    /// cumulative point count cannot see a channel starved after the first
    /// iteration, since the first iteration's α split dominates the sum.
    pub min_channel_neval: usize,
}

/// One channel's running record over the iterations of a block-split adaptation.
struct ChannelHistory {
    alpha: f64,
    /// Points drawn so far — the block's generator position.
    drawn: u64,
    /// `(integral, variance, neval)` of every iteration past the warm-up, the
    /// variance exactly as measured. Zero is a value it really takes — an
    /// iteration all of whose points came back equal, which on a wide split is
    /// routinely an allocation every point of which failed the cuts — and the
    /// stopping test needs to see that rather than a floored stand-in.
    kept: Vec<(f64, f64, usize)>,
    /// Running sum of the per-point variances measured by *every* iteration, and
    /// the count behind it: the Neyman input. Warm-up iterations are included
    /// because before the warm-up is over they are the only measurement there is.
    point_var_sum: f64,
    point_var_n: usize,
}

impl ChannelHistory {
    /// The per-point standard deviation this channel's iterations have measured,
    /// or `None` before any iteration has run.
    fn point_sd(&self) -> Option<f64> {
        (self.point_var_n > 0).then(|| (self.point_var_sum / self.point_var_n as f64).sqrt())
    }

    /// This channel's term, combined over its kept iterations.
    ///
    /// Iterations that drew the same number of points combine under `rule` —
    /// the same arithmetic, in the same order, that a run of
    /// [`VegasGrid::adapt_parallel_seeded`] would have performed, so a fixed
    /// budget's numbers do not move.
    ///
    /// Iterations that drew *different* numbers are averaged weighted by their
    /// point counts. Those weights are what Lepage's are not: an iteration's
    /// point count is decided before any of its points exist, so it cannot
    /// correlate with the estimate it weights, which is the correlation that
    /// makes the `1/σ²` rule biased. What survives is a weaker dependence — a
    /// count is chosen from *earlier* iterations' variances, and those are
    /// correlated with earlier iterations' estimates. Bounding it needs a
    /// measurement rather than an argument: over 8 seeds at four
    /// process/target settings, this weighted mean and the plain unweighted one
    /// (which is what a fixed split takes, its counts never varying) differ by
    /// 0.26–0.40 σ with no common sign — no bias at the 0.02–0.04% these
    /// resolve.
    fn combine(&self, rule: IterationCombination) -> VegasResult {
        Self::combine_kept(&self.kept, rule)
    }

    /// [`ChannelHistory::combine`] over an arbitrary set of this channel's
    /// iterations. Every quoted variance is floored at [`f64::MIN_POSITIVE`],
    /// which is what keeps a zero-variance iteration from dividing the χ² sum by
    /// nothing; the floor makes that term enormous instead of undefined, so a
    /// caller that cares about the difference filters before it calls.
    fn combine_kept(kept: &[(f64, f64, usize)], rule: IterationCombination) -> VegasResult {
        let uniform = kept
            .first()
            .is_some_and(|&(_, _, n0)| kept.iter().all(|&(_, _, n)| n == n0));
        if uniform {
            let pairs: Vec<(f64, f64)> = kept
                .iter()
                .map(|&(i, v, _)| (i, v.max(f64::MIN_POSITIVE)))
                .collect();
            return combine_iterations(&pairs, rule);
        }
        let wtot: f64 = kept.iter().map(|&(_, _, n)| n as f64).sum();
        let integral: f64 = kept.iter().map(|&(i, _, n)| i * n as f64).sum::<f64>() / wtot;
        let var: f64 = kept
            .iter()
            .map(|&(_, v, n)| v.max(f64::MIN_POSITIVE) * (n as f64) * (n as f64))
            .sum::<f64>()
            / (wtot * wtot);
        let chi2_per_dof = if kept.len() > 1 {
            kept.iter()
                .map(|&(i, v, _)| (i - integral).powi(2) / v.max(f64::MIN_POSITIVE))
                .sum::<f64>()
                / (kept.len() - 1) as f64
        } else {
            0.0
        };
        VegasResult {
            integral,
            std_dev: var.sqrt(),
            chi2_per_dof,
        }
    }

    /// The iteration-consistency scale factor the stopping test widens this
    /// channel's quoted error by: `max(1, χ²/dof)`.
    ///
    /// The χ² is formed over the iterations that measured a variance. An
    /// iteration whose points all came back equal measured a variance of exactly
    /// zero, and zero is not a small error bar — it is no error bar at all. A
    /// residual divided by it is arbitrarily large, so on a split wide enough
    /// that most channels get the [`MIN_CHANNEL_NEVAL`] floor and some of their
    /// allocations lose every point to the cuts, the resulting χ²/dof reaches
    /// ~1e250 and no target is satisfiable. Those iterations are dropped from
    /// the consistency test and from nothing else: they still set the channel's
    /// integral and its quoted error, and the report still prints the χ²/dof
    /// they produce.
    ///
    /// Dropping them costs the test nothing it could measure. A channel
    /// populated in a minority of its iterations already quotes a relative error
    /// of order one — the empty iterations drag its mean down while contributing
    /// no variance — and that error enters the stopping sum in quadrature, where
    /// a channel too small to matter is weighted by how small it is. What the
    /// scale factor exists to catch is a channel whose iterations disagree by
    /// more than error bars they *did* quote, and every such iteration is still
    /// in the sum.
    ///
    /// A channel with fewer than two informative iterations has no scatter to
    /// measure and gets no scale factor. Where every iteration is informative
    /// this is the χ²/dof of the whole channel, unchanged.
    fn stop_scale(&self, rule: IterationCombination) -> f64 {
        if self.kept.iter().all(|&(_, v, _)| v > 0.0) {
            return self.combine(rule).chi2_per_dof.max(1.0);
        }
        let informative: Vec<(f64, f64, usize)> = self
            .kept
            .iter()
            .copied()
            .filter(|&(_, v, _)| v > 0.0)
            .collect();
        if informative.len() < 2 {
            return 1.0;
        }
        Self::combine_kept(&informative, rule).chi2_per_dof.max(1.0)
    }
}

/// Points one rayon task evaluates before its results are reduced.
///
/// `CHUNKS_PER_THREAD` several rather than one so a straggler costs a fraction of
/// an iteration rather than all of it, and `MIN_CHUNK` so per-chunk setup (a
/// generator seek, a substream, two small allocations) stays far below the cost
/// of the points in it.
///
/// The result is a scheduling knob only — [`adapt_blocks_iteration`] is
/// bit-identical at any chunk size, which a run at `CHUNKS_PER_THREAD` 2, 8 and
/// 32 confirms artifact-byte for artifact-byte — so it is chosen for balance
/// alone. Sizing from the channel's own budget rather than the iteration's is a
/// second knob of the same kind: on a 24-channel `p p > l+ l- j` iteration at 16
/// threads the two differ by more than a factor ten in chunk count and by
/// nothing that a contended host could resolve, so the simpler one stays.
fn chunk_size(channel_neval: usize, threads: usize) -> usize {
    const MIN_CHUNK: usize = 64;
    const CHUNKS_PER_THREAD: usize = 8;
    channel_neval
        .div_ceil(threads.max(1) * CHUNKS_PER_THREAD)
        .max(MIN_CHUNK)
}

/// Split `total` points across channels with per-point standard deviations
/// `sd`, `Nⱼ ∝ sdⱼ`, subject to `Nⱼ ≥ floor`.
///
/// Channels the proportional rule would put below the floor are pinned there and
/// the rest re-split over what remains, repeatedly — so the floor is exact rather
/// than a post-hoc clamp that would overspend the budget. When every channel is
/// pinned the total simply rises to `channels × floor`: coverage wins over the
/// budget, which is the point of having a floor at all.
pub fn neyman_allocation(sd: &[f64], total: usize, floor: usize) -> Vec<usize> {
    // Two points is the least an iteration's variance estimator can be formed
    // from, so a floor below that is not a floor on anything.
    let floor = floor.max(2);
    let n = sd.len();
    let mut out = vec![floor; n];
    let mut pinned = vec![false; n];
    loop {
        let free: Vec<usize> = (0..n).filter(|&j| !pinned[j]).collect();
        if free.is_empty() {
            break;
        }
        let remaining = total.saturating_sub(floor * (n - free.len()));
        let sd_total: f64 = free.iter().map(|&j| sd[j].max(0.0)).sum();
        let mut newly_pinned = false;
        for &j in &free {
            let share = if sd_total > 0.0 {
                remaining as f64 * sd[j].max(0.0) / sd_total
            } else {
                remaining as f64 / free.len() as f64
            };
            let share = if share.is_finite() && share > 0.0 {
                share.round() as usize
            } else {
                0
            };
            if share <= floor {
                out[j] = floor;
                pinned[j] = true;
                newly_pinned = true;
            } else {
                out[j] = share;
            }
        }
        if !newly_pinned {
            break;
        }
    }
    out
}

/// Run a multichannel integration under `budget`, one grid per channel.
///
/// Every iteration is a single rayon region scheduled by `(channel, chunk)`:
/// a narrow channel does not get a parallel region to itself, and a wide one does
/// not serialise the iteration behind it. The scheduling is inert — see
/// [`adapt_blocks_iteration`] — so which channels a region happens to interleave
/// cannot move a number.
///
/// `vegas_alpha` is the grid-damping exponent every channel's grid is built with.
///
/// `stop` is read at each iteration boundary; [`StopSignal::default`] is the
/// signal nobody can raise, and a run under it is decided by `budget` alone.
///
/// A run stopped before its warm-up is over has kept no iteration, and the terms
/// it returns are combined over nothing: they are not numbers, and
/// [`ConvergenceReport::kept_iterations`] is `0`. A caller that can be stopped
/// has to read that field before presenting a result — there is no cross section
/// to present, and the grids, while sampled, were never trained past the
/// iterations that were meant to be thrown away.
#[allow(clippy::too_many_arguments)]
pub fn integrate_channels<I>(
    integrand: &I,
    alphas: &[f64],
    vegas_alpha: f64,
    combination: IterationCombination,
    budget: Budget,
    allocation: BlockAllocation,
    seed: u64,
    stop: &StopSignal,
) -> (Vec<ChannelIntegration>, VegasResult, ConvergenceReport)
where
    I: ChannelIntegrand + Sync,
{
    let _span = info_span!("vegas").entered();
    let ndim = integrand.channel_grid_ndim();
    let scale_ndim = integrand.scale_draw_ndim();
    let point_ndim = ndim + scale_ndim;
    let neval = budget.neval();
    let single = alphas.len() == 1;

    let mut grids: Vec<VegasGrid> = alphas
        .iter()
        .map(|_| VegasGrid::new(ndim, VEGAS_NBINS, vegas_alpha).with_combination(combination))
        .collect();
    let mut channels: Vec<ChannelHistory> = alphas
        .iter()
        .map(|&alpha| ChannelHistory {
            alpha,
            drawn: 0,
            kept: Vec::new(),
            point_var_sum: 0.0,
            point_var_n: 0,
        })
        .collect();

    if let Budget::Target { .. } = budget {
        assert_eq!(
            combination,
            IterationCombination::Unweighted,
            "a convergence target reads the combined error bar, and the inverse-variance \
             combination's is biased small by its own weights"
        );
    }

    // The α split is what a fixed budget spends every iteration and what the
    // first iteration of a Neyman run spends, there being no variance measured
    // yet to reallocate on.
    let by_alpha: Vec<usize> = alphas
        .iter()
        .map(|&a| {
            if single {
                neval
            } else {
                crate::hadronic::channel_neval(a, neval)
            }
        })
        .collect();
    let alpha_total: usize = by_alpha.iter().sum();

    info!(
        "{} channels over {ndim} coordinates, {alpha_total} points per iteration, allocated {}",
        alphas.len(),
        match allocation {
            BlockAllocation::ByAlpha => "by α",
            BlockAllocation::Neyman => "by αⱼsⱼ (Neyman)",
        }
    );
    // Coverage outranks the budget, so a request under the floor is raised, not
    // honoured. Say so: the spend is then set by the channel count and is not a
    // knob `neval` can turn until it clears the floor.
    if alpha_total > neval {
        let floored = by_alpha.iter().filter(|&&n| n == MIN_CHANNEL_NEVAL).count();
        warn!(
            "neval {neval} is below what {} channels can cover: {floored} of them sit at the \
             {MIN_CHANNEL_NEVAL}-point floor, so an iteration spends {alpha_total}",
            alphas.len(),
        );
    }

    // The iteration count a display divides by. A convergence run may stop well
    // short of it, so it is an upper bound rather than a plan — but it is an
    // upper bound in points the run will really spend: `alpha_total`, not
    // `neval`, is what an iteration costs once the per-channel floor has raised
    // it, so the point cap buys fewer iterations than dividing by the requested
    // budget would suggest. One iteration is the floor, since a bound of zero
    // describes no run.
    let iteration_bound = match budget {
        Budget::Fixed { niter, .. } => niter as u64,
        Budget::Target {
            max_iters,
            max_points,
            ..
        } => (max_iters as u64)
            .min(max_points / alpha_total.max(1) as u64)
            .max(1),
    };

    let warmup = grids
        .first()
        .map(|g| g.warmup())
        .unwrap_or_default()
        .min(budget.max_iters().saturating_sub(1));

    let threads = rayon::current_num_threads();
    let mut current = by_alpha.clone();
    let mut points = 0_u64;
    let mut iteration_points: usize;
    let mut min_channel_neval = usize::MAX;
    let mut iteration = 0_usize;
    let mut reason = StopReason::Budget;

    loop {
        if iteration > 0 && !single && allocation == BlockAllocation::Neyman {
            let sd: Vec<f64> = channels
                .iter()
                .map(|c| c.point_sd().unwrap_or(0.0))
                .collect();
            let previous = current.clone();
            current = neyman_allocation(&sd, alpha_total, MIN_CHANNEL_NEVAL);
            report_reallocation(&previous, &current);
        }

        let plans: Vec<BlockPlan> = channels
            .iter()
            .zip(&current)
            .enumerate()
            .map(|(j, (c, &n_j))| BlockPlan {
                neval: n_j,
                first_point: c.drawn,
                stream: crate::hadronic::CHANNEL_STREAM_BASE + j as u64,
                chunk_size: chunk_size(n_j, threads),
            })
            .collect();

        min_channel_neval = min_channel_neval.min(plans.iter().map(|p| p.neval).min().unwrap_or(0));
        iteration_points = plans.iter().map(|p| p.neval).sum();
        let started = Instant::now();

        let outcomes = adapt_blocks_iteration(
            &grids,
            &plans,
            seed,
            // The grid draws its own coordinates from `CHANNEL_STREAM_BASE + j`;
            // the scale draw's trailing uniforms come off a stream of its own, so
            // the grid's sequence is what it would be with no draw installed.
            // Both are addressed by the point's index in the channel's own run, so
            // a chunk reproduces the points it would have drawn in sequence.
            |j, first| {
                (
                    SubStream::new(
                        seed,
                        SCALE_DRAW_STREAM_BASE + j as u64,
                        first * scale_ndim as u64,
                    ),
                    vec![0.0; point_ndim],
                )
            },
            |j, (scale_draw, point), u| {
                point[..ndim].copy_from_slice(u);
                scale_draw.fill_uniforms(&mut point[ndim..]);
                integrand.value_in_channel(j, point)
            },
        );

        for ((c, out), plan) in channels.iter_mut().zip(&outcomes).zip(&plans) {
            c.drawn += plan.neval as u64;
            points += plan.neval as u64;
            c.point_var_sum += out.point_variance(plan.neval);
            c.point_var_n += 1;
            if iteration >= warmup {
                c.kept.push((out.integral, out.variance, plan.neval));
            }
        }

        let ns_per_eval = started.elapsed().as_nanos() as f64 / iteration_points.max(1) as f64;
        iteration += 1;

        // Before the warm-up is over no iteration has been kept, and a combined
        // estimate over nothing is not a number.
        let estimate = (iteration > warmup).then(|| running_estimate(&channels, combination));
        match &estimate {
            Some(total) => info!(
                "iteration {iteration}: {:.6e} ± {:.6e} (χ²/dof {:.3}), {iteration_points} points \
                 at {ns_per_eval:.0} ns/eval",
                total.integral, total.std_dev, total.chi2_per_dof
            ),
            None => info!(
                "iteration {iteration} (warm-up): {iteration_points} points at \
                 {ns_per_eval:.0} ns/eval"
            ),
        }
        let total = estimate.unwrap_or(VegasResult {
            integral: 0.0,
            std_dev: 0.0,
            chi2_per_dof: 0.0,
        });
        progress::vegas_iteration(
            iteration as u64,
            Some(iteration_bound),
            total.integral,
            total.std_dev,
            total.chi2_per_dof,
        );
        progress::eval_rate(
            progress::stage::VEGAS,
            iteration as u64,
            Some(iteration_bound),
            ns_per_eval,
        );
        report_channels(&plans, &outcomes);

        let done = match budget {
            Budget::Fixed { niter, .. } => {
                reason = StopReason::Budget;
                iteration >= niter
            }
            Budget::Target {
                target_rel,
                min_iters,
                max_iters,
                max_points,
                ..
            } => {
                let met = iteration >= min_iters.max(warmup + 2)
                    && scaled_rel(&channels, combination) <= target_rel;
                if met {
                    reason = StopReason::TargetMet;
                    true
                } else if iteration >= max_iters {
                    reason = StopReason::MaxIters;
                    true
                } else if points + iteration_points as u64 > max_points {
                    // Prospective, and against what an iteration really costs:
                    // the per-channel floor can put that several times above
                    // `neval`, so a cap tested only after the fact is overshot
                    // by an iteration whose size the caller never asked for.
                    // The forecast is the iteration just drawn, which is the
                    // size the next one is allocated to as well.
                    reason = StopReason::MaxPoints;
                    true
                } else {
                    false
                }
            }
        };

        // A budget that has run out on this very iteration keeps its own reason:
        // the run reached the end it was given, and was not cut short of it.
        if done {
            break;
        }
        if stop.requested() {
            reason = StopReason::Aborted;
            if iteration > warmup {
                warn!(
                    "stopping at the operator's request after {iteration} iterations \
                     and {points} evaluations"
                );
            } else {
                warn!(
                    "stopping at the operator's request after {iteration} iterations \
                     and {points} evaluations, with the warm-up unfinished: no iteration \
                     was kept, so this run measured nothing"
                );
            }
            break;
        }
        // Only a run that will draw again refines: the banked grid is the one the
        // last iteration sampled against.
        for ((g, out), plan) in grids.iter_mut().zip(&outcomes).zip(&plans) {
            g.refine_grid(&out.hist, plan.neval);
        }
    }

    let per_channel: Vec<ChannelIntegration> = channels
        .iter()
        .zip(grids)
        .zip(&current)
        .map(|((c, grid), &n_j)| ChannelIntegration {
            alpha: c.alpha,
            neval: n_j,
            grid,
            result: c.combine(combination),
        })
        .collect();
    let total = combine_channels(&per_channel, iteration);
    let achieved_rel = rel_of(&total);
    let report = ConvergenceReport {
        stop: reason,
        iterations: iteration,
        kept_iterations: channels.first().map(|c| c.kept.len()).unwrap_or(0),
        points,
        points_per_iteration: iteration_points,
        target_rel: match budget {
            Budget::Target { target_rel, .. } => Some(target_rel),
            Budget::Fixed { .. } => None,
        },
        achieved_rel,
        scaled_rel: scaled_rel(&channels, combination),
        channel_points: channels.iter().map(|c| c.drawn).collect(),
        min_channel_neval,
    };
    (per_channel, total, report)
}

/// The combined estimate over every channel's kept iterations, as it stands.
///
/// The same sum the run's own result is formed from: terms add, their variances
/// add, and the reported χ²/dof is the mean of the channels' own.
fn running_estimate(channels: &[ChannelHistory], combination: IterationCombination) -> VegasResult {
    let terms: Vec<VegasResult> = channels.iter().map(|c| c.combine(combination)).collect();
    let integral: f64 = terms.iter().map(|t| t.integral).sum();
    let variance: f64 = terms.iter().map(|t| t.std_dev * t.std_dev).sum();
    let chi2_per_dof = if terms.is_empty() {
        0.0
    } else {
        terms.iter().map(|t| t.chi2_per_dof).sum::<f64>() / terms.len() as f64
    };
    VegasResult {
        integral,
        std_dev: variance.sqrt(),
        chi2_per_dof,
    }
}

/// What each channel's own iteration just measured, one line per channel.
fn report_channels(plans: &[BlockPlan], outcomes: &[BlockIteration]) {
    if !tracing::enabled!(tracing::Level::DEBUG) {
        return;
    }
    for (j, (plan, out)) in plans.iter().zip(outcomes).enumerate() {
        debug!(
            "channel {j}: {:.6e} ± {:.6e} over {} points",
            out.integral,
            out.variance.max(0.0).sqrt(),
            plan.neval
        );
    }
}

/// The channels whose per-iteration budget the variance reallocation moved, and by
/// how much. Channels it left alone are omitted — on a wide process they are most
/// of the list and none of the information.
fn report_reallocation(before: &[usize], after: &[usize]) {
    if !tracing::enabled!(tracing::Level::DEBUG) {
        return;
    }
    let moved: Vec<String> = before
        .iter()
        .zip(after)
        .enumerate()
        .filter(|(_, (b, a))| b != a)
        .map(|(j, (b, a))| format!("{j}: {b}→{a}"))
        .collect();
    if !moved.is_empty() {
        debug!("reallocated {}", moved.join(", "));
    }
}

/// `Δσ/σ` of a result, `∞` for a vanishing integral.
fn rel_of(r: &VegasResult) -> f64 {
    if r.integral > 0.0 {
        r.std_dev / r.integral
    } else {
        f64::INFINITY
    }
}

/// The relative uncertainty the stopping test reads: every channel's quoted error
/// inflated by `√max(1, χ²/dofⱼ)` before the channels are summed in quadrature.
///
/// The scale factor goes on per channel rather than on the total because that is
/// where the inconsistency is: one channel whose iterations disagree should widen
/// its own term, not the terms of channels that agree with themselves. It is
/// [`ChannelHistory::stop_scale`] rather than the channel's reported χ²/dof, so
/// that a channel with iterations that measured no variance at all still yields a
/// number a control decision can be made on.
fn scaled_rel(channels: &[ChannelHistory], combination: IterationCombination) -> f64 {
    let mut integral = 0.0_f64;
    let mut variance = 0.0_f64;
    for c in channels {
        let r = c.combine(combination);
        integral += r.integral;
        variance += r.std_dev * r.std_dev * c.stop_scale(combination);
    }
    if integral > 0.0 {
        variance.sqrt() / integral
    } else {
        f64::INFINITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vegas::DEFAULT_WARMUP_ITERS;

    /// A multichannel integrand with a known integral: channel `j` contributes
    /// `αⱼ · f(u)` with `Σ αⱼ = 1`, so the terms sum to `∫f`. `spread[j]` scales
    /// a peak that only channel `j` sees, which is how a test hands one channel
    /// all the variance without moving the answer.
    struct Toy {
        alphas: Vec<f64>,
        spread: Vec<f64>,
    }

    impl Toy {
        /// Every channel integrates to `αⱼ · 1.5`, so the terms always sum to
        /// `1.5` whatever the spreads are — what `spread[j]` changes is only how
        /// much variance channel `j` carries while getting there.
        ///
        /// `spread[j] == 0` is the degenerate end of that: a channel whose value
        /// is the constant `1.5`, with exactly zero variance, which is the
        /// profile a variance-driven allocation would spend nothing on.
        /// Otherwise `∫₀¹∫₀¹ (3u₀² + u₁) du = 1.5` plus a bump antisymmetric
        /// about `u₁ = ½`, which integrates to zero exactly and is therefore
        /// pure variance.
        fn value(&self, j: usize, u: &[f64]) -> f64 {
            let spread = self.spread[j];
            if spread == 0.0 {
                return self.alphas[j] * 1.5;
            }
            let base = 3.0 * u[0] * u[0] + u[1];
            let bump = spread * (u[1] - 0.5).signum() * (u[0] * 40.0).exp() / 4.0e16;
            self.alphas[j] * (base + bump)
        }
    }

    impl ChannelIntegrand for Toy {
        fn channel_count(&self) -> usize {
            self.alphas.len()
        }
        fn channel_grid_ndim(&self) -> usize {
            2
        }
        fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
            self.value(channel, u)
        }
    }

    fn toy(spread: Vec<f64>) -> Toy {
        let n = spread.len();
        Toy {
            alphas: vec![1.0 / n as f64; n],
            spread,
        }
    }

    /// A multichannel integrand whose minor channels have support narrower than
    /// their allocation can reliably find: channel `j > 0` is a spike on
    /// `u₀ < width` and zero elsewhere, so an iteration of it either lands a
    /// point in the spike or comes back a constant zero — with a measured
    /// variance of exactly zero.
    ///
    /// That is the profile a split wide enough to put most channels on the
    /// [`MIN_CHANNEL_NEVAL`] floor produces by the hundred, and the one the
    /// stopping test has to stay well-defined on. Every channel still integrates
    /// to `αⱼ · 1.5`, so the terms sum to `1.5` as the other toy's do.
    struct SparseTail {
        alphas: Vec<f64>,
        width: f64,
    }

    impl ChannelIntegrand for SparseTail {
        fn channel_count(&self) -> usize {
            self.alphas.len()
        }
        fn channel_grid_ndim(&self) -> usize {
            2
        }
        fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
            if channel == 0 {
                return self.alphas[0] * (3.0 * u[0] * u[0] + u[1]);
            }
            if u[0] < self.width {
                self.alphas[channel] * 1.5 / self.width
            } else {
                0.0
            }
        }
    }

    /// One channel's kept history, built by hand. Handing the stopping test an
    /// iteration of exactly zero measured variance on demand is the only way to
    /// pin what it does with one.
    fn history(kept: &[(f64, f64)]) -> ChannelHistory {
        ChannelHistory {
            alpha: 1.0,
            drawn: 0,
            kept: kept
                .iter()
                .map(|&(i, v)| (i, v, MIN_CHANNEL_NEVAL))
                .collect(),
            point_var_sum: 0.0,
            point_var_n: 0,
        }
    }

    /// Where every iteration measured a variance there is nothing to exclude, and
    /// the factor the stopping test widens by is the channel's own χ²/dof.
    #[test]
    fn the_stop_scale_is_the_reported_chi2_when_every_iteration_measured_one() {
        let rule = IterationCombination::Unweighted;
        let h = history(&[(1.0, 0.01), (1.2, 0.01), (0.9, 0.01)]);
        let chi2 = h.combine(rule).chi2_per_dof;
        assert!(
            chi2 > 1.0,
            "the case worth testing needs a widening χ²: {chi2}"
        );
        assert_eq!(h.stop_scale(rule), chi2);
    }

    /// An iteration whose points all came back equal measured no error bar, not a
    /// vanishing one. Dividing a residual by it is what puts the reported χ²/dof
    /// at ~1e250 on a wide split — a number the report is welcome to carry and a
    /// control decision cannot be made on. The stopping test reads the iterations
    /// that measured something.
    #[test]
    fn a_zero_variance_iteration_cannot_blow_up_the_stop_scale() {
        let rule = IterationCombination::Unweighted;
        let empty = [(0.0, 0.0); 6];
        let fired = [(1.0e-9, 1.0e-21), (1.3e-9, 1.0e-21)];
        let mixed: Vec<(f64, f64)> = empty.iter().chain(&fired).copied().collect();

        let reported = history(&mixed).combine(rule).chi2_per_dof;
        assert!(
            reported > 1.0e200 && reported.is_finite(),
            "the pathology this guards against did not reproduce: χ²/dof {reported}"
        );

        let scale = history(&mixed).stop_scale(rule);
        assert_eq!(scale, history(&fired).combine(rule).chi2_per_dof);
        assert!(scale.is_finite() && scale < 1.0e3, "stop scale {scale}");
    }

    /// A channel that fired in one iteration out of many has no scatter to
    /// measure, and gets no scale factor. What it contributes to the stopping sum
    /// is its quoted error, which for such a channel is already of the size of
    /// its own term.
    #[test]
    fn one_informative_iteration_is_no_scatter_measurement() {
        let rule = IterationCombination::Unweighted;
        let mut kept = vec![(0.0, 0.0); 7];
        kept.push((1.0e-9, 1.0e-21));
        assert_eq!(history(&kept).stop_scale(rule), 1.0);
    }

    /// End to end on a split wide enough to produce the degenerate iterations by
    /// itself: the reported χ²/dof still overflows — that pass-through is what the
    /// report is for — while the stopping test reads a finite number and the run
    /// reaches the accuracy it was asked for instead of burning its cap.
    #[test]
    fn a_wide_split_with_empty_iterations_still_converges() {
        let channels = 40;
        let epsilon = 1.0e-5;
        let mut alphas = vec![epsilon; channels];
        alphas[0] = 1.0 - epsilon * (channels - 1) as f64;
        let integ = SparseTail {
            alphas: alphas.clone(),
            width: 6.0e-4,
        };
        let target_rel = 5.0e-3;
        let (per_channel, total, report) = integrate_channels(
            &integ,
            &alphas,
            1.5,
            IterationCombination::Unweighted,
            Budget::Target {
                target_rel,
                neval: 4_000,
                min_iters: 6,
                max_iters: 40,
                max_points: 200_000_000,
            },
            BlockAllocation::ByAlpha,
            0xC0FFEE,
            &StopSignal::default(),
        );
        let worst = per_channel
            .iter()
            .map(|c| c.result.chi2_per_dof)
            .fold(0.0_f64, f64::max);
        assert!(
            worst > 1.0e100,
            "no channel produced the degenerate χ²/dof this test exists for (worst {worst})"
        );
        assert_eq!(report.stop, StopReason::TargetMet);
        assert!(
            report.scaled_rel.is_finite() && report.scaled_rel <= target_rel,
            "stopped at scaled rel {}",
            report.scaled_rel
        );
        assert!(report.achieved_rel <= report.scaled_rel);
        let pull = (total.integral - 1.5).abs() / total.std_dev;
        assert!(pull < 5.0, "σ = {} ± {}", total.integral, total.std_dev);
    }

    #[test]
    fn neyman_allocation_is_proportional_when_nothing_binds() {
        let n = neyman_allocation(&[1.0, 2.0, 5.0], 8_000, 100);
        assert_eq!(n, vec![1_000, 2_000, 5_000]);
    }

    /// The guardrail, on the profile that attacks it: one channel carrying every
    /// bit of the variance and the rest measuring exactly zero, which is what the
    /// proportional rule would hand nothing at all.
    #[test]
    fn neyman_allocation_floors_a_zero_variance_channel() {
        let floor = MIN_CHANNEL_NEVAL;
        let sd = vec![1.0e9, 0.0, 0.0, 0.0, 0.0, 0.0];
        let n = neyman_allocation(&sd, 100_000, floor);
        for (j, &n_j) in n.iter().enumerate() {
            assert!(n_j >= floor, "channel {j} got {n_j} < floor {floor}");
        }
        assert_eq!(n[0], 100_000 - 5 * floor);
        assert_eq!(&n[1..], &[floor; 5]);
        // Nothing was conjured: the split spends the budget it was given.
        assert_eq!(n.iter().sum::<usize>(), 100_000);
    }

    /// A budget too small to floor every channel raises the total rather than
    /// starving one: coverage outranks the budget.
    #[test]
    fn neyman_allocation_never_starves_below_the_floor() {
        let n = neyman_allocation(&[1.0, 0.0, 0.0], 100, MIN_CHANNEL_NEVAL);
        assert_eq!(n, vec![MIN_CHANNEL_NEVAL; 3]);
    }

    /// A NaN standard deviation (a channel that saw no non-zero point at all)
    /// is not allowed to swallow or to starve the split.
    #[test]
    fn neyman_allocation_survives_a_degenerate_estimate() {
        let n = neyman_allocation(&[f64::NAN, 3.0, 1.0], 10_000, MIN_CHANNEL_NEVAL);
        for (j, &n_j) in n.iter().enumerate() {
            assert!(n_j >= MIN_CHANNEL_NEVAL, "channel {j} got {n_j}");
        }
        assert!(
            n[1] > n[2],
            "the measurable channels still split by their sd"
        );
    }

    /// End to end: with the variance concentrated in one channel, the Neyman rule
    /// pours the budget into it — and every other channel still draws its floor,
    /// every iteration, for the whole run.
    #[test]
    fn convergence_run_keeps_every_channel_above_the_floor() {
        let integ = toy(vec![1.0, 0.0, 0.0, 0.0]);
        let budget = Budget::Target {
            target_rel: 1.0e-3,
            neval: 40_000,
            min_iters: 4,
            max_iters: 40,
            max_points: 20_000_000,
        };
        let (per_channel, _total, report) = integrate_channels(
            &integ,
            &integ.alphas.clone(),
            1.5,
            IterationCombination::Unweighted,
            budget,
            BlockAllocation::Neyman,
            0x1234,
            &StopSignal::default(),
        );
        assert!(
            report.min_channel_neval >= MIN_CHANNEL_NEVAL,
            "some channel was allocated {} points in some iteration, below the floor {}",
            report.min_channel_neval,
            MIN_CHANNEL_NEVAL
        );
        // And cumulatively too, which is the weaker statement but the one a
        // caller reading the report will look at first.
        for (j, points) in report.channel_points.iter().enumerate() {
            assert!(
                *points >= MIN_CHANNEL_NEVAL as u64 * report.iterations as u64,
                "channel {j} drew {points} over {} iterations",
                report.iterations
            );
        }
        assert!(
            per_channel[0].neval > per_channel[1].neval,
            "the variance-carrying channel should have been fed: {} vs {}",
            per_channel[0].neval,
            per_channel[1].neval
        );
    }

    /// A target budget stops on the accuracy it was asked for, no earlier than
    /// `min_iters`, and the estimate it stops on covers the known integral.
    #[test]
    fn convergence_run_stops_at_its_target() {
        let integ = toy(vec![1.0, 0.3, 0.0, 0.0]);
        let target_rel = 2.0e-3;
        let (_per_channel, total, report) = integrate_channels(
            &integ,
            &integ.alphas.clone(),
            1.5,
            IterationCombination::Unweighted,
            Budget::Target {
                target_rel,
                neval: 20_000,
                min_iters: 6,
                max_iters: 200,
                max_points: 200_000_000,
            },
            BlockAllocation::Neyman,
            0xBEEF,
            &StopSignal::default(),
        );
        assert_eq!(report.stop, StopReason::TargetMet);
        assert!(
            report.iterations >= 6,
            "stopped after {}",
            report.iterations
        );
        assert!(
            report.scaled_rel <= target_rel,
            "stopped at scaled rel {} above the target {target_rel}",
            report.scaled_rel
        );
        // The scale factor only ever widens the bar it is applied to.
        assert!(report.achieved_rel <= report.scaled_rel);
        let pull = (total.integral - 1.5).abs() / total.std_dev;
        assert!(pull < 5.0, "σ = {} ± {}", total.integral, total.std_dev);
    }

    /// The cap is a cap: an unreachable target stops on iterations and says so.
    #[test]
    fn convergence_run_respects_its_iteration_cap() {
        let integ = toy(vec![1.0, 1.0]);
        let (_c, _t, report) = integrate_channels(
            &integ,
            &integ.alphas.clone(),
            1.5,
            IterationCombination::Unweighted,
            Budget::Target {
                target_rel: 1.0e-12,
                neval: 4_000,
                min_iters: 2,
                max_iters: 5,
                max_points: 200_000_000,
            },
            BlockAllocation::ByAlpha,
            7,
            &StopSignal::default(),
        );
        assert_eq!(report.stop, StopReason::MaxIters);
        assert_eq!(report.iterations, 5);
        assert!(!report.stop.converged());
    }

    /// The point cap likewise, and before the iteration cap when it is the
    /// tighter of the two. It is a bound rather than a threshold: the run stops
    /// before an iteration it cannot afford, having spent the cap or less.
    #[test]
    fn convergence_run_respects_its_point_cap() {
        let integ = toy(vec![1.0, 1.0]);
        let cap = 22_000_u64;
        let (_c, _t, report) = integrate_channels(
            &integ,
            &integ.alphas.clone(),
            1.5,
            IterationCombination::Unweighted,
            Budget::Target {
                target_rel: 1.0e-12,
                neval: 4_000,
                min_iters: 2,
                max_iters: 1_000,
                max_points: cap,
            },
            BlockAllocation::ByAlpha,
            7,
            &StopSignal::default(),
        );
        assert_eq!(report.stop, StopReason::MaxPoints);
        assert!(
            report.points <= cap,
            "spent {} against a cap of {cap}",
            report.points
        );
        // And it stopped because it could not afford another, not early.
        assert!(
            report.points + report.points_per_iteration as u64 > cap,
            "stopped at {} with room for another {} points",
            report.points,
            report.points_per_iteration
        );
    }

    /// A split too wide for the requested budget spends what the per-channel
    /// floor costs, and the report says so in points rather than repeating the
    /// request back. That number is what an iteration count has to be priced at:
    /// the caps a caller sets bind at `points_per_iteration × iterations`.
    #[test]
    fn a_floored_split_reports_what_an_iteration_really_spends() {
        let channels = 40;
        let neval = 4_000;
        let integ = toy(vec![1.0; channels]);
        let (_c, _t, report) = integrate_channels(
            &integ,
            &integ.alphas.clone(),
            1.5,
            IterationCombination::Unweighted,
            Budget::Target {
                target_rel: 1.0e-12,
                neval,
                min_iters: 2,
                max_iters: 4,
                max_points: 200_000_000,
            },
            BlockAllocation::ByAlpha,
            7,
            &StopSignal::default(),
        );
        assert_eq!(report.points_per_iteration, channels * MIN_CHANNEL_NEVAL);
        assert!(
            report.points_per_iteration > neval,
            "{channels} channels at the {MIN_CHANNEL_NEVAL}-point floor should outspend {neval}"
        );
        assert_eq!(
            report.points,
            report.points_per_iteration as u64 * report.iterations as u64
        );
    }

    /// A toy that raises a stop signal once it has been evaluated `after` times.
    ///
    /// A key press arrives part-way through an iteration, and that is the only
    /// interesting moment to arrive at: a signal raised before the run starts
    /// cannot tell an iteration boundary from a loop that never began.
    struct StopAfter<'a> {
        inner: &'a Toy,
        stop: StopSignal,
        seen: std::sync::atomic::AtomicUsize,
        after: usize,
    }

    impl ChannelIntegrand for StopAfter<'_> {
        fn channel_count(&self) -> usize {
            self.inner.channel_count()
        }
        fn channel_grid_ndim(&self) -> usize {
            self.inner.channel_grid_ndim()
        }
        fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
            if self.seen.fetch_add(1, Ordering::Relaxed) + 1 >= self.after {
                self.stop.request();
            }
            self.inner.value_in_channel(channel, u)
        }
    }

    /// A stop raised mid-run ends it at the next iteration boundary, with whole
    /// iterations' worth of grids and terms to show for it — the property a
    /// caller banking the artifact depends on.
    #[test]
    fn a_raised_stop_signal_ends_the_run_at_an_iteration_boundary() {
        let inner = toy(vec![1.0, 1.0]);
        let neval = 4_000;
        let integ = StopAfter {
            inner: &inner,
            stop: StopSignal::new(),
            seen: std::sync::atomic::AtomicUsize::new(0),
            // Part-way into the iteration after the warm-up, so there is exactly
            // one kept iteration and the run is nowhere near its budget.
            after: neval * (DEFAULT_WARMUP_ITERS + 1) + neval / 2,
        };
        let (per_channel, total, report) = integrate_channels(
            &integ,
            &inner.alphas.clone(),
            1.5,
            IterationCombination::Unweighted,
            Budget::Fixed { neval, niter: 40 },
            BlockAllocation::ByAlpha,
            7,
            &integ.stop,
        );
        assert_eq!(report.stop, StopReason::Aborted);
        assert!(!report.stop.converged());
        assert!(
            report.iterations > DEFAULT_WARMUP_ITERS && report.iterations < 40,
            "stopped after {} iterations",
            report.iterations
        );
        assert_eq!(
            report.kept_iterations,
            report.iterations - DEFAULT_WARMUP_ITERS
        );
        assert_eq!(per_channel.len(), 2);
        assert!(
            total.integral.is_finite() && total.integral > 0.0,
            "{total:?}"
        );
    }

    /// A stop taken *inside* the warm-up keeps nothing at all, and the report
    /// says so rather than leaving a caller to discover it from a NaN. Only the
    /// `kept_iterations` field distinguishes this from a run that measured
    /// something, which is why a stoppable caller has to read it.
    #[test]
    fn a_stop_inside_the_warm_up_keeps_nothing_and_reports_it() {
        let integ = toy(vec![1.0, 1.0]);
        let stop = StopSignal::new();
        stop.request();
        let (_per_channel, total, report) = integrate_channels(
            &integ,
            &integ.alphas.clone(),
            1.5,
            IterationCombination::Unweighted,
            Budget::Fixed {
                neval: 4_000,
                niter: 20,
            },
            BlockAllocation::ByAlpha,
            7,
            &stop,
        );
        assert_eq!(report.stop, StopReason::Aborted);
        assert_eq!(report.iterations, 1);
        assert_eq!(report.kept_iterations, 0);
        assert!(!total.integral.is_finite(), "{total:?}");
    }

    /// A run that finishes its budget has kept every iteration past the warm-up,
    /// which is the baseline the stopped runs above are read against.
    #[test]
    fn a_completed_run_keeps_every_iteration_past_the_warm_up() {
        let integ = toy(vec![1.0, 1.0]);
        let (_c, _t, report) = integrate_channels(
            &integ,
            &integ.alphas.clone(),
            1.5,
            IterationCombination::Unweighted,
            Budget::Fixed {
                neval: 4_000,
                niter: 6,
            },
            BlockAllocation::ByAlpha,
            7,
            &StopSignal::default(),
        );
        assert_eq!(report.iterations, 6);
        assert_eq!(report.kept_iterations, 6 - DEFAULT_WARMUP_ITERS);
    }

    /// The signal is read at the boundary and nowhere else, so a run that is not
    /// stopped draws exactly what a run with no signal at all draws — the
    /// property that keeps the abort out of the arithmetic.
    #[test]
    fn an_unraised_stop_signal_changes_nothing() {
        let integ = toy(vec![1.0, 0.3]);
        let run = |stop: &StopSignal| {
            integrate_channels(
                &integ,
                &integ.alphas.clone(),
                1.5,
                IterationCombination::Unweighted,
                Budget::Fixed {
                    neval: 6_000,
                    niter: 5,
                },
                BlockAllocation::Neyman,
                0x9E,
                stop,
            )
        };
        let (_, quiet, quiet_report) = run(&StopSignal::default());
        let (_, armed, armed_report) = run(&StopSignal::new());
        assert_eq!(quiet.integral.to_bits(), armed.integral.to_bits());
        assert_eq!(quiet.std_dev.to_bits(), armed.std_dev.to_bits());
        assert_eq!(quiet_report.points, armed_report.points);
        assert_eq!(quiet_report.stop, StopReason::Budget);
        assert_eq!(armed_report.stop, StopReason::Budget);
    }

    /// A budget that runs out on the same iteration the signal is read keeps its
    /// own reason: the run reached its end rather than being cut short of it.
    #[test]
    fn a_completed_budget_is_not_reported_as_an_abort() {
        let integ = toy(vec![1.0, 1.0]);
        let stop = StopSignal::new();
        stop.request();
        let (_c, _t, report) = integrate_channels(
            &integ,
            &integ.alphas.clone(),
            1.5,
            IterationCombination::Unweighted,
            Budget::Fixed {
                neval: 4_000,
                niter: 1,
            },
            BlockAllocation::ByAlpha,
            7,
            &stop,
        );
        assert_eq!(report.stop, StopReason::Budget);
        assert_eq!(report.iterations, 1);
    }

    /// The prerequisite, pinned rather than asserted in prose: a convergence stop
    /// reads an error bar, and Lepage's is the one biased small by its own
    /// weights, so a target budget refuses it outright.
    #[test]
    #[should_panic(expected = "inverse-variance")]
    fn a_target_budget_refuses_the_inverse_variance_combination() {
        let integ = toy(vec![1.0, 1.0]);
        integrate_channels(
            &integ,
            &integ.alphas.clone(),
            1.5,
            IterationCombination::InverseVariance,
            Budget::Target {
                target_rel: 1.0e-2,
                neval: 2_000,
                min_iters: 2,
                max_iters: 10,
                max_points: 1_000_000,
            },
            BlockAllocation::ByAlpha,
            1,
            &StopSignal::default(),
        );
    }

    /// With every channel statistically identical the two allocation rules have
    /// nothing to choose between, so they must agree point for point — the
    /// control that says the `Neyman` arm differs from `ByAlpha` because of the
    /// variances it read and not because of a bookkeeping difference.
    #[test]
    fn the_allocation_rules_agree_when_there_is_nothing_to_reallocate() {
        let integ = toy(vec![0.5, 0.5, 0.5, 0.5]);
        let budget = Budget::Fixed {
            neval: 8_000,
            niter: 6,
        };
        let run = |a| {
            integrate_channels(
                &integ,
                &integ.alphas.clone(),
                1.5,
                IterationCombination::Unweighted,
                budget,
                a,
                0x51,
                &StopSignal::default(),
            )
        };
        let (_, by_alpha, ra) = run(BlockAllocation::ByAlpha);
        let (_, neyman, rn) = run(BlockAllocation::Neyman);
        // Rounding a proportional share can move a total by up to half a point
        // per channel; anything beyond that is a rule that spent differently.
        assert!(
            ra.points.abs_diff(rn.points) <= integ.alphas.len() as u64,
            "the two rules spent different budgets: {} vs {}",
            ra.points,
            rn.points
        );
        let d = (by_alpha.integral - neyman.integral).abs();
        assert!(
            d < 3.0 * by_alpha.std_dev,
            "{} vs {}",
            by_alpha.integral,
            neyman.integral
        );
    }
}
