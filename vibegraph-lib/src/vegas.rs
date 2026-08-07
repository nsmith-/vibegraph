//! Classic VEGAS adaptive Monte Carlo integrator (Lepage 1978).
//!
//! Integrates a function `f: [0,1]^N → ℝ` by iteratively reshaping a
//! piecewise-uniform importance-sampling grid to concentrate evaluations
//! where `|f|` is large.
//!
//! # Algorithm summary
//!
//! The integration domain `[0,1]^N` is partitioned into `nbins` equal-probability
//! bins per dimension.  Each iteration:
//!
//! 1. Draw `neval` random samples; accumulate the integral estimate and
//!    per-bin f² weights.
//! 2. Refine the grid: rescale bin boundaries so that each bin captures an
//!    equal share of `∫ |f| dx` (Lepage's importance-function update with
//!    α-damping to suppress noise).
//!
//! Multiple iterations are then combined into one estimate (next section); the
//! χ²/dof across iterations diagnoses convergence.
//!
//! # Combining the iterations
//!
//! Every iteration's estimate is unbiased for the integral whatever grid it ran
//! on — importance sampling divides by the density it drew from. What is *not*
//! unbiased is the `1/σ²` combination of them, because each iteration's weight
//! is estimated from the very samples that produced its integral: an iteration
//! that undersamples the peak returns a low integral **and** a low variance, so
//! the combination weights that low answer up. The resulting bias is `O(1/N)` in
//! the per-iteration sample count, and at the per-channel budgets a
//! multichannel integration can afford it is the dominant error.
//!
//! Two independent knobs control the combination, both recoverable to the plain
//! Lepage behaviour:
//!
//! * [`VegasGrid::warmup`] — leading iterations excluded from the combination.
//!   They still draw their points and still refine the grid; only their
//!   estimates are dropped, so this costs `warmup/niter` of the run's
//!   statistics and no integrand evaluations. What it buys is variance, not
//!   bias: the first iterations run on a grid that has not found the peak, and
//!   their estimates are the noisy ones.
//! * [`VegasGrid::combination`] — how the surviving iterations are averaged.
//!   [`IterationCombination::Unweighted`] takes the arithmetic mean, whose
//!   weights are fixed in advance and therefore cannot correlate with the
//!   estimates; [`IterationCombination::InverseVariance`] is Lepage's `1/σ²`
//!   mean.
//!
//! Measured on a 5-dimensional product of Gaussians (`σ = 0.15`) whose exact
//! integral is known, over 4 000 seeds at `niter = 10` — mean relative error,
//! the seeds' RMS spread about the truth, and the error the rule quotes:
//!
//! | points/iter | rule, warm-up | mean rel | RMS rel | mean quoted err |
//! |---|---|--:|--:|--:|
//! | 2 000 | `1/σ²`, 0 (Lepage) | **−1.21%** | 8.2% | 0.72% |
//! | 2 000 | `1/σ²`, 2 | **−1.40%** | 9.3% | 0.76% |
//! | 2 000 | unweighted, 2 | +0.14% | 10.7% | 1.70% |
//! | 10 000 | `1/σ²`, 0 (Lepage) | **−0.024%** | 0.187% | 0.182% |
//! | 10 000 | `1/σ²`, 2 | **−0.022%** | 0.191% | 0.186% |
//! | 10 000 | unweighted, 2 | −0.0004% | 0.193% | 0.191% |
//! | 50 000 | `1/σ²`, 0 (Lepage) | −0.0028% | 0.076% | 0.076% |
//! | 50 000 | unweighted, 2 | +0.0011% | 0.081% | 0.081% |
//!
//! The mean's own error over 4 000 seeds is the RMS over `√4000`, so the bold
//! entries are 8–9 standard errors from zero and the unweighted ones are inside
//! one. Two readings set the defaults. **The warm-up discard alone does not
//! remove the bias** — the correlation between an iteration's estimate and its
//! weight is in every iteration, not only the early ones, and dropping the
//! early ones makes it marginally worse by leaving the sharper grids to
//! dominate. The unweighted mean does remove it, at no cost in spread once the
//! warm-up iterations are out of it, and it quotes an error much closer to the
//! spread it actually has (at 2 000 points Lepage's rule is wrong by 1.2% while
//! claiming 0.7%).
//!
//! # Two-phase usage: adapt, then freeze
//!
//! [`VegasGrid::adapt`] both estimates the integral and reshapes the grid.
//! Once a grid has converged it can be serialized ([`VegasGrid`] implements
//! `Serialize`/`Deserialize`), shipped elsewhere, and reused for importance
//! sampling with no further refinement via [`VegasGrid::sample_frozen`] — the
//! primitive a distributed event-generation phase builds on.
//!
//! # Quick start
//!
//! ```rust
//! use vibegraph::vegas::Vegas;
//! use rand::SeedableRng;
//!
//! let mut vegas = Vegas::new(1, 50, 1.5);
//! let mut rng = rand::rngs::StdRng::seed_from_u64(0);
//! // Integrate f(u) = 3u² over [0,1] — exact answer is 1.
//! let result = vegas.integrate(|u| 3.0 * u[0] * u[0], 10_000, 5, &mut rng);
//! assert!((result.integral - 1.0).abs() < 0.01);
//! ```

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};

use crate::phasespace::rng::WORDS_PER_DRAW;

/// Leading adaptation iterations a freshly built grid excludes from its
/// combined estimate — see the module's "Combining the iterations" section.
///
/// The unweighted mean gives every surviving iteration the same weight, so the
/// iterations that ran before the grid found the peak enter it at full strength
/// and their variance dominates the result. Dropping them is what makes the
/// unweighted rule affordable: in the same 5-dimensional Gaussian study, the
/// seeds' RMS spread at 10 000 points an iteration runs `0.53%` at `warmup = 0`,
/// `0.20%` at 1, `0.19%` at 2, `0.20%` at 3, `0.22%` at 4 — a minimum one or two
/// iterations in, then the slow rise of spending a tenth of the run per further
/// discard (`0.22%`, `0.25%` at 4 and 5). Two rather than one because the
/// minimum is flat there and the harder configurations (7 dimensions) put it at
/// two.
pub const DEFAULT_WARMUP_ITERS: usize = 2;

/// How an adaptation's surviving per-iteration estimates are averaged.
///
/// See the module's "Combining the iterations" section for why the default is
/// not Lepage's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IterationCombination {
    /// Arithmetic mean of the iteration estimates, quoting `√(Σσᵢ²)/n` for the
    /// mean's own error.
    ///
    /// The weights are fixed before any sampling, so they cannot correlate with
    /// what they weight, and the mean is exactly as unbiased as the individual
    /// estimates are. It is not the minimum-variance combination when the
    /// iterations genuinely differ in precision — but the `1/σ²` weights that
    /// would be are estimated from the same samples, and paying that
    /// correlation is what the bias is.
    #[default]
    Unweighted,
    /// Lepage's `1/σ²` weighted mean, quoting `1/√(Σ1/σᵢ²)`.
    InverseVariance,
}

/// One evaluation point handed to a batched integrand callback.
///
/// `u` is the `ndim`-length point in `[0,1]^ndim` after grid remapping (the
/// same coordinate the unbatched integrand receives), borrowed from the
/// batch's internal buffer.
#[derive(Debug, Clone, Copy)]
pub struct SamplePoint<'a> {
    pub u: &'a [f64],
}

/// Errors rejected by [`VegasGrid::from_raw`] (and hence by `Deserialize`).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum VegasGridError {
    #[error("VegasGrid: ndim must be nonzero")]
    ZeroDims,
    #[error("VegasGrid: nbins must be nonzero")]
    ZeroBins,
    #[error("VegasGrid: xi has {got} dimension(s), expected ndim = {expected}")]
    ShapeMismatch { expected: usize, got: usize },
    #[error("VegasGrid: xi[{dim}] has {got} edge(s), expected nbins+1 = {expected}")]
    BinCountMismatch {
        dim: usize,
        expected: usize,
        got: usize,
    },
    #[error("VegasGrid: xi[{dim}][{which}] = {value}, expected {expected}")]
    BadEndpoint {
        dim: usize,
        which: &'static str,
        value: f64,
        expected: f64,
    },
    #[error(
        "VegasGrid: xi[{dim}] is not strictly increasing at bin edge {bin} ({prev} >= {next})"
    )]
    NonMonotone {
        dim: usize,
        bin: usize,
        prev: f64,
        next: f64,
    },
}

/// VEGAS adaptive importance-sampling grid.
///
/// The grid `xi[d][k]` stores the `k`-th bin boundary in dimension `d`,
/// with `xi[d][0] = 0` and `xi[d][nbins] = 1`.
///
/// `Deserialize` runs the same validation as [`VegasGrid::from_raw`]
/// (monotone edges, `0`/`1` endpoints, shape consistency) so a corrupt grid
/// is rejected at deserialize time rather than surfacing as silent
/// mis-sampling later.
#[derive(Debug, Clone, Serialize)]
pub struct VegasGrid {
    ndim: usize,
    nbins: usize,
    alpha: f64,
    xi: Vec<Vec<f64>>,
    /// Leading adaptation iterations excluded from the combined estimate.
    ///
    /// Not serialized: what a stored grid carries is its trained bin edges,
    /// and a grid rebuilt from those edges has no warm-up left to do. Keeping
    /// it out of the wire format also leaves every banked artifact's bytes
    /// unchanged.
    #[serde(skip)]
    warmup: usize,
    /// How the surviving iterations are averaged. Not serialized, and for the
    /// same reason: it describes an adaptation, not a grid.
    #[serde(skip)]
    combination: IterationCombination,
}

/// Plain deserialization target; validated into a [`VegasGrid`] afterward.
#[derive(Debug, Deserialize)]
struct VegasGridRaw {
    ndim: usize,
    nbins: usize,
    alpha: f64,
    xi: Vec<Vec<f64>>,
}

impl<'de> Deserialize<'de> for VegasGrid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = VegasGridRaw::deserialize(deserializer)?;
        VegasGrid::from_raw(raw.ndim, raw.nbins, raw.alpha, raw.xi)
            .map_err(serde::de::Error::custom)
    }
}

/// Result returned by a VEGAS integration pass (adapt or frozen).
#[derive(Debug, Clone, Copy)]
pub struct VegasResult {
    /// Best estimate of the integral (weighted combination of all iterations).
    pub integral: f64,
    /// Standard deviation of the estimate.
    pub std_dev: f64,
    /// χ²/dof across iterations.  Values near 1 indicate consistency between
    /// iterations; large values warn of poor convergence or a pathological
    /// integrand.  `0` for a single-iteration (e.g. frozen-grid) pass.
    pub chi2_per_dof: f64,
}

impl VegasGrid {
    /// Create a new VEGAS grid with a uniform initial bin layout.
    ///
    /// * `ndim`  – integration dimensions
    /// * `nbins` – bins per dimension (50–100 is typical)
    /// * `alpha` – grid-damping exponent (Lepage: 1.5)
    ///
    /// Starts at [`DEFAULT_WARMUP_ITERS`] warm-up iterations; use
    /// [`with_warmup`](Self::with_warmup) to change or disable the discard.
    pub fn new(ndim: usize, nbins: usize, alpha: f64) -> Self {
        let xi = (0..ndim)
            .map(|_| (0..=nbins).map(|i| i as f64 / nbins as f64).collect())
            .collect();
        VegasGrid {
            ndim,
            nbins,
            alpha,
            xi,
            warmup: DEFAULT_WARMUP_ITERS,
            combination: IterationCombination::default(),
        }
    }

    /// Build a grid from raw parts, validating shape and bin-edge invariants.
    ///
    /// Used directly and by `Deserialize`. Rejects: zero `ndim`/`nbins`,
    /// `xi.len() != ndim`, any `xi[d].len() != nbins + 1`, endpoints other
    /// than `0.0`/`1.0`, and non-strictly-increasing edges.
    pub fn from_raw(
        ndim: usize,
        nbins: usize,
        alpha: f64,
        xi: Vec<Vec<f64>>,
    ) -> Result<Self, VegasGridError> {
        if ndim == 0 {
            return Err(VegasGridError::ZeroDims);
        }
        if nbins == 0 {
            return Err(VegasGridError::ZeroBins);
        }
        if xi.len() != ndim {
            return Err(VegasGridError::ShapeMismatch {
                expected: ndim,
                got: xi.len(),
            });
        }
        for (dim, edges) in xi.iter().enumerate() {
            if edges.len() != nbins + 1 {
                return Err(VegasGridError::BinCountMismatch {
                    dim,
                    expected: nbins + 1,
                    got: edges.len(),
                });
            }
            if edges[0] != 0.0 {
                return Err(VegasGridError::BadEndpoint {
                    dim,
                    which: "first",
                    value: edges[0],
                    expected: 0.0,
                });
            }
            if edges[nbins] != 1.0 {
                return Err(VegasGridError::BadEndpoint {
                    dim,
                    which: "last",
                    value: edges[nbins],
                    expected: 1.0,
                });
            }
            for (bin, w) in edges.windows(2).enumerate() {
                if !(w[0] < w[1]) {
                    return Err(VegasGridError::NonMonotone {
                        dim,
                        bin,
                        prev: w[0],
                        next: w[1],
                    });
                }
            }
        }
        Ok(VegasGrid {
            ndim,
            nbins,
            alpha,
            xi,
            // Edges supplied from outside are a trained grid, so there is no
            // warm-up phase to discard; a caller that means to keep adapting
            // from uniform edges asks for one explicitly.
            warmup: 0,
            combination: IterationCombination::default(),
        })
    }

    pub fn ndim(&self) -> usize {
        self.ndim
    }

    pub fn nbins(&self) -> usize {
        self.nbins
    }

    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    /// Bin edges per dimension: `xi()[d]` has length `nbins() + 1`.
    pub fn xi(&self) -> &[Vec<f64>] {
        &self.xi
    }

    /// Leading adaptation iterations excluded from the combined estimate.
    pub fn warmup(&self) -> usize {
        self.warmup
    }

    /// Exclude the first `warmup` adaptation iterations from the combined
    /// estimate.
    ///
    /// Those iterations still draw their points and still refine the grid; only
    /// their `(integral, variance)` pairs are kept out of the combination. What
    /// the discard removes is the variance of estimates taken before the grid
    /// found the peak, which under an unweighted mean would enter at full
    /// strength (module docs). `0` combines every iteration.
    ///
    /// An adaptation always contributes at least its last iteration: a
    /// `warmup` at or above `niter` is clamped to `niter - 1`, since an
    /// estimate from no iterations is not an estimate.
    pub fn set_warmup(&mut self, warmup: usize) {
        self.warmup = warmup;
    }

    /// Builder form of [`set_warmup`](Self::set_warmup).
    pub fn with_warmup(mut self, warmup: usize) -> Self {
        self.warmup = warmup;
        self
    }

    /// How the surviving iterations are averaged.
    pub fn combination(&self) -> IterationCombination {
        self.combination
    }

    /// Set how the surviving iterations are averaged. See
    /// [`IterationCombination`].
    pub fn set_combination(&mut self, combination: IterationCombination) {
        self.combination = combination;
    }

    /// Builder form of [`set_combination`](Self::set_combination).
    pub fn with_combination(mut self, combination: IterationCombination) -> Self {
        self.combination = combination;
        self
    }

    /// Iterations of an `niter`-iteration adaptation that are discarded.
    fn effective_warmup(&self, niter: usize) -> usize {
        self.warmup.min(niter.saturating_sub(1))
    }

    // ── Adapt phase ──────────────────────────────────────────────────────

    /// Integrate `f` over `[0, 1]^ndim`, refining the grid between iterations.
    ///
    /// * `f`     – integrand; receives a `&[f64]` of length `ndim`
    /// * `neval` – integrand evaluations per iteration
    /// * `niter` – number of adaptation iterations
    /// * `rng`   – random number source
    ///
    /// The returned estimate combines the iterations after the first
    /// [`warmup`](Self::warmup) of them; all `niter` still sample and refine.
    pub fn adapt(
        &mut self,
        mut f: impl FnMut(&[f64]) -> f64,
        neval: usize,
        niter: usize,
        rng: &mut impl Rng,
    ) -> VegasResult {
        self.adapt_batched(
            |points, out| {
                for (p, o) in points.iter().zip(out.iter_mut()) {
                    *o = f(p.u);
                }
            },
            neval,
            niter,
            1,
            rng,
        )
    }

    /// Batched-integrand form of [`VegasGrid::adapt`].
    ///
    /// `f` is invoked once per batch of up to `batch_size` points, writing
    /// each point's raw integrand value (before the VEGAS Jacobian weight)
    /// into the matching slot of the output slice — the seam a lane-batched
    /// evaluator plugs into. Draw order and accumulation order are
    /// independent of `batch_size`: the result is bit-identical to
    /// [`VegasGrid::adapt`] for any `batch_size`.
    pub fn adapt_batched<Fb>(
        &mut self,
        mut f: Fb,
        neval: usize,
        niter: usize,
        batch_size: usize,
        rng: &mut impl Rng,
    ) -> VegasResult
    where
        Fb: FnMut(&[SamplePoint], &mut [f64]),
    {
        let warmup = self.effective_warmup(niter);
        let mut iter_results: Vec<(f64, f64)> = Vec::with_capacity(niter - warmup);
        for iter_idx in 0..niter {
            let (integral, var, d) = self.run_iter_batched(&mut f, neval, batch_size, rng);
            if iter_idx >= warmup {
                iter_results.push((integral, var.max(f64::MIN_POSITIVE)));
            }
            if iter_idx + 1 < niter {
                self.refine_grid(&d, neval);
            }
        }
        combine_iterations(&iter_results, self.combination)
    }

    /// Deterministic-parallel form of [`VegasGrid::adapt`].
    ///
    /// `neval` samples per iteration are split into fixed-size chunks of
    /// `chunk_size`, each evaluated on its own [`ChaCha8Rng`] substream
    /// (stream id keyed by `(iteration, chunk_index)`, word position `0`;
    /// see [`substream_id`]) and reduced sequentially in chunk order — so
    /// the result is bit-identical regardless of the rayon thread-pool
    /// size. `f` must be `Sync` since chunks run concurrently.
    pub fn adapt_parallel<Fp>(
        &mut self,
        f: Fp,
        neval: usize,
        niter: usize,
        seed: u64,
        chunk_size: usize,
    ) -> VegasResult
    where
        Fp: Fn(&[f64]) -> f64 + Sync,
    {
        let warmup = self.effective_warmup(niter);
        let mut iter_results: Vec<(f64, f64)> = Vec::with_capacity(niter - warmup);
        for iter_idx in 0..niter {
            let (integral, var, d) =
                self.run_iter_parallel(&f, neval, chunk_size, iter_idx as u32, seed);
            if iter_idx >= warmup {
                iter_results.push((integral, var.max(f64::MIN_POSITIVE)));
            }
            if iter_idx + 1 < niter {
                self.refine_grid(&d, neval);
            }
        }
        combine_iterations(&iter_results, self.combination)
    }

    /// Parallel form of [`VegasGrid::adapt`] over a **seekable** substream, whose
    /// result is bit-for-bit that of the sequential `adapt` driven by
    /// `SubStream::new(seed, stream, 0)`.
    ///
    /// Two properties make that identity hold rather than merely hold usually:
    ///
    /// * **Draw addressing.** One point consumes exactly `ndim` 64-bit draws, so
    ///   the point at global index `p` (counting across iterations, as the
    ///   sequential form's single generator does) starts at draw `p · ndim`.
    ///   A chunk seeks straight to its own first point instead of inheriting a
    ///   predecessor's generator state, which is what lets chunks run out of
    ///   order without moving a single point.
    /// * **Accumulation order.** Chunks return their per-point weighted values and
    ///   bin indices; the sums and the refinement histogram are then formed in
    ///   global point order on one thread. Floating-point addition is not
    ///   associative, so a per-chunk partial sum would give a different — equally
    ///   valid, but different — grid at the next refinement, and from there a
    ///   different point sequence entirely.
    ///
    /// Consequently `chunk_size` and the rayon pool size are pure scheduling
    /// knobs: neither changes the answer. `init` builds whatever per-chunk state
    /// the integrand needs alongside the grid coordinates — a substream of its
    /// own, positioned from the chunk's first global point index — and `f`
    /// evaluates one point against it.
    #[allow(clippy::too_many_arguments)]
    pub fn adapt_parallel_seeded<S, Init, Fp>(
        &mut self,
        init: Init,
        f: Fp,
        neval: usize,
        niter: usize,
        seed: u64,
        stream: u64,
        chunk_size: usize,
    ) -> VegasResult
    where
        Init: Fn(u64) -> S + Sync,
        Fp: Fn(&mut S, &[f64]) -> f64 + Sync,
        S: Send,
    {
        let warmup = self.effective_warmup(niter);
        let mut iter_results: Vec<(f64, f64)> = Vec::with_capacity(niter - warmup);
        for iter_idx in 0..niter {
            let first_point = (iter_idx * neval) as u64;
            let (integral, var, d) =
                self.run_iter_seeded(&init, &f, neval, chunk_size, first_point, seed, stream);
            if iter_idx >= warmup {
                iter_results.push((integral, var.max(f64::MIN_POSITIVE)));
            }
            if iter_idx + 1 < niter {
                self.refine_grid(&d, neval);
            }
        }
        combine_iterations(&iter_results, self.combination)
    }

    // ── Frozen phase (no grid refinement) ───────────────────────────────

    /// One importance-sampling pass over `f` with **no** grid refinement —
    /// the distributed-generation phase primitive: sample against a
    /// previously adapted (and possibly deserialized) grid.
    pub fn sample_frozen(
        &self,
        mut f: impl FnMut(&[f64]) -> f64,
        neval: usize,
        rng: &mut impl Rng,
    ) -> VegasResult {
        self.sample_frozen_batched(
            |points, out| {
                for (p, o) in points.iter().zip(out.iter_mut()) {
                    *o = f(p.u);
                }
            },
            neval,
            1,
            rng,
        )
    }

    /// Batched-integrand form of [`VegasGrid::sample_frozen`]. See
    /// [`VegasGrid::adapt_batched`] for the batching/ordering contract.
    pub fn sample_frozen_batched<Fb>(
        &self,
        mut f: Fb,
        neval: usize,
        batch_size: usize,
        rng: &mut impl Rng,
    ) -> VegasResult
    where
        Fb: FnMut(&[SamplePoint], &mut [f64]),
    {
        let (integral, var, _d) = self.run_iter_batched(&mut f, neval, batch_size, rng);
        combine_iterations(&[(integral, var.max(f64::MIN_POSITIVE))], self.combination)
    }

    /// Deterministic-parallel form of [`VegasGrid::sample_frozen`]. See
    /// [`VegasGrid::adapt_parallel`] for the substream addressing contract.
    pub fn sample_frozen_parallel<Fp>(
        &self,
        f: Fp,
        neval: usize,
        seed: u64,
        chunk_size: usize,
    ) -> VegasResult
    where
        Fp: Fn(&[f64]) -> f64 + Sync,
    {
        let (integral, var, _d) = self.run_iter_parallel(&f, neval, chunk_size, 0, seed);
        combine_iterations(&[(integral, var.max(f64::MIN_POSITIVE))], self.combination)
    }

    // ── Private helpers ──────────────────────────────────────────────────

    /// Draw one point against the grid: fill `x` (length `ndim`) and return the
    /// VEGAS Jacobian weight `1/p(x)`.
    ///
    /// The weight an accept/reject pass over a frozen grid divides by: a point's
    /// full sampling weight is this Jacobian times the integrand there, so the
    /// per-point maximum of that product is what unweighting is normalised
    /// against. Draws the same sequence [`sample_frozen`](Self::sample_frozen)
    /// draws from the same `rng` state.
    pub fn draw(&self, rng: &mut impl Rng, x: &mut [f64]) -> f64 {
        self.draw_into(rng, x, |_, _| {})
    }

    /// Draw one point: fill `x` (length `ndim`) and `ks` (length `ndim`,
    /// bin index per dimension) and return the VEGAS Jacobian weight.
    fn draw_point(&self, rng: &mut impl Rng, x: &mut [f64], ks: &mut [usize]) -> f64 {
        self.draw_into(rng, x, |dim, k| ks[dim] = k)
    }

    /// The draw itself, reporting each dimension's bin index to `record` — so a
    /// caller that needs the bins for grid refinement and one that needs only the
    /// point share a single definition of the sampling.
    fn draw_into(
        &self,
        rng: &mut impl Rng,
        x: &mut [f64],
        mut record: impl FnMut(usize, usize),
    ) -> f64 {
        let nbins = self.nbins;
        let mut wgt = 1.0_f64;
        for (dim, (xd, edges)) in x.iter_mut().zip(&self.xi).enumerate() {
            let u: f64 = rng.random();
            let pos = u * nbins as f64;
            let k = (pos as usize).min(nbins - 1);
            let r = pos - k as f64;
            let lo = edges[k];
            let hi = edges[k + 1];
            *xd = lo + r * (hi - lo);
            wgt *= nbins as f64 * (hi - lo); // Jacobian dx/du = bin_width, wgt = 1/p(x)
            record(dim, k);
        }
        wgt
    }

    /// One iteration of [`VegasGrid::adapt`] against the grid as it stands,
    /// which it leaves unrefined: the estimate, its variance, and the histogram
    /// [`VegasGrid::refine_grid`] reshapes from.
    ///
    /// `adapt` is this in a loop at a constant `neval`. Driving the loop from
    /// outside is what an adaptation whose per-iteration point count *varies*
    /// needs — a channel of a block split is allocated afresh every iteration —
    /// and it is how a sequential reference for such a run is written.
    #[cfg(test)]
    pub(crate) fn adapt_iteration(
        &self,
        mut f: impl FnMut(&[f64]) -> f64,
        neval: usize,
        rng: &mut impl Rng,
    ) -> (f64, f64, Vec<Vec<f64>>) {
        self.run_iter_batched(
            &mut |points: &[SamplePoint], out: &mut [f64]| {
                for (p, o) in points.iter().zip(out.iter_mut()) {
                    *o = f(p.u);
                }
            },
            neval,
            1,
            rng,
        )
    }

    /// Draw `neval` samples in batches, evaluating `f` once per batch.
    /// Returns `(integral_estimate, variance, d)`, `d[dim][bin]` accumulating
    /// `fval²` for grid refinement.
    fn run_iter_batched<Fb>(
        &self,
        f: &mut Fb,
        neval: usize,
        batch_size: usize,
        rng: &mut impl Rng,
    ) -> (f64, f64, Vec<Vec<f64>>)
    where
        Fb: FnMut(&[SamplePoint], &mut [f64]),
    {
        let batch_size = batch_size.max(1);
        let mut d = vec![vec![0.0_f64; self.nbins]; self.ndim];
        let mut sum = 0.0_f64;
        let mut sum2 = 0.0_f64;

        let mut flat_x = vec![0.0_f64; batch_size * self.ndim];
        let mut flat_ks = vec![0_usize; batch_size * self.ndim];
        let mut wgts = vec![0.0_f64; batch_size];
        let mut fvals = vec![0.0_f64; batch_size];

        let mut remaining = neval;
        while remaining > 0 {
            let this_batch = remaining.min(batch_size);
            for i in 0..this_batch {
                let x = &mut flat_x[i * self.ndim..(i + 1) * self.ndim];
                let ks = &mut flat_ks[i * self.ndim..(i + 1) * self.ndim];
                wgts[i] = self.draw_point(rng, x, ks);
            }
            let points: Vec<SamplePoint> = flat_x[..this_batch * self.ndim]
                .chunks_exact(self.ndim)
                .map(|u| SamplePoint { u })
                .collect();
            f(&points, &mut fvals[..this_batch]);

            for i in 0..this_batch {
                let fval = fvals[i] * wgts[i];
                let fval2 = fval * fval;
                sum += fval;
                sum2 += fval2;
                let ks = &flat_ks[i * self.ndim..(i + 1) * self.ndim];
                for dim in 0..self.ndim {
                    d[dim][ks[dim]] += fval2;
                }
            }
            remaining -= this_batch;
        }

        let n = neval as f64;
        let mean = sum / n;
        let variance = ((sum2 / n - mean * mean) / (n - 1.0)).max(0.0);
        (mean, variance, d)
    }

    /// Sequential accumulation of `neval` points for one chunk of the
    /// deterministic-parallel path.
    fn accumulate_points(
        &self,
        f: &(impl Fn(&[f64]) -> f64 + Sync),
        neval: usize,
        rng: &mut impl Rng,
    ) -> (f64, f64, Vec<Vec<f64>>) {
        let mut d = vec![vec![0.0_f64; self.nbins]; self.ndim];
        let mut x = vec![0.0_f64; self.ndim];
        let mut ks = vec![0_usize; self.ndim];
        let mut sum = 0.0_f64;
        let mut sum2 = 0.0_f64;

        for _ in 0..neval {
            let wgt = self.draw_point(rng, &mut x, &mut ks);
            let fval = f(&x) * wgt;
            let fval2 = fval * fval;
            sum += fval;
            sum2 += fval2;
            for dim in 0..self.ndim {
                d[dim][ks[dim]] += fval2;
            }
        }

        (sum, sum2, d)
    }

    /// Split `neval` into fixed-size chunks, each on its own `(iter,
    /// chunk_idx)`-keyed `ChaCha8Rng` substream, evaluated in parallel and
    /// reduced sequentially in chunk order.
    fn run_iter_parallel<Fp>(
        &self,
        f: &Fp,
        neval: usize,
        chunk_size: usize,
        iter_idx: u32,
        seed: u64,
    ) -> (f64, f64, Vec<Vec<f64>>)
    where
        Fp: Fn(&[f64]) -> f64 + Sync,
    {
        let chunk_size = chunk_size.max(1);
        let nchunks = neval.div_ceil(chunk_size);

        let chunk_results: Vec<(f64, f64, Vec<Vec<f64>>)> = (0..nchunks)
            .into_par_iter()
            .map(|chunk_idx| {
                let this_chunk = if chunk_idx + 1 == nchunks {
                    neval - chunk_idx * chunk_size
                } else {
                    chunk_size
                };
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                rng.set_stream(substream_id(iter_idx, chunk_idx as u32));
                rng.set_word_pos(0);
                self.accumulate_points(f, this_chunk, &mut rng)
            })
            .collect();

        let mut sum = 0.0_f64;
        let mut sum2 = 0.0_f64;
        let mut d = vec![vec![0.0_f64; self.nbins]; self.ndim];
        for (chunk_sum, chunk_sum2, chunk_d) in &chunk_results {
            sum += chunk_sum;
            sum2 += chunk_sum2;
            for (d_dim, chunk_d_dim) in d.iter_mut().zip(chunk_d) {
                for (v, cv) in d_dim.iter_mut().zip(chunk_d_dim) {
                    *v += cv;
                }
            }
        }

        let n = neval as f64;
        let mean = sum / n;
        let variance = ((sum2 / n - mean * mean) / (n - 1.0)).max(0.0);
        (mean, variance, d)
    }

    /// One iteration of [`adapt_parallel_seeded`](Self::adapt_parallel_seeded):
    /// chunks evaluate concurrently from seeked generator positions, then a single
    /// pass accumulates their per-point values in global point order.
    ///
    /// The two halves are what make the result independent of `chunk_size` and of
    /// the pool size: the first reproduces the sequential draw sequence, the second
    /// reproduces its summation order.
    #[allow(clippy::too_many_arguments)]
    fn run_iter_seeded<S, Init, Fp>(
        &self,
        init: &Init,
        f: &Fp,
        neval: usize,
        chunk_size: usize,
        first_point: u64,
        seed: u64,
        stream: u64,
    ) -> (f64, f64, Vec<Vec<f64>>)
    where
        Init: Fn(u64) -> S + Sync,
        Fp: Fn(&mut S, &[f64]) -> f64 + Sync,
        S: Send,
    {
        assert!(
            self.nbins <= usize::from(u16::MAX) + 1,
            "a chunk carries its bin indices out as u16, so {} bins do not fit",
            self.nbins
        );
        let chunk_size = chunk_size.max(1);
        let nchunks = neval.div_ceil(chunk_size);
        let ndim = self.ndim;

        // Per chunk: the weighted integrand value at each of its points, and the
        // bin each point landed in per dimension (flattened, `ndim` per point).
        let chunks: Vec<(Vec<f64>, Vec<u16>)> = (0..nchunks)
            .into_par_iter()
            .map(|chunk_idx| {
                let this_chunk = if chunk_idx + 1 == nchunks {
                    neval - chunk_idx * chunk_size
                } else {
                    chunk_size
                };
                let chunk_first = first_point + (chunk_idx * chunk_size) as u64;
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                rng.set_stream(stream);
                rng.set_word_pos(u128::from(chunk_first) * ndim as u128 * WORDS_PER_DRAW);

                let mut state = init(chunk_first);
                let mut x = vec![0.0_f64; ndim];
                let mut ks = vec![0_usize; ndim];
                let mut fvals = Vec::with_capacity(this_chunk);
                let mut bins = Vec::with_capacity(this_chunk * ndim);
                for _ in 0..this_chunk {
                    let wgt = self.draw_point(&mut rng, &mut x, &mut ks);
                    fvals.push(f(&mut state, &x) * wgt);
                    bins.extend(ks.iter().map(|&k| k as u16));
                }
                (fvals, bins)
            })
            .collect();

        let mut d = vec![vec![0.0_f64; self.nbins]; self.ndim];
        let mut sum = 0.0_f64;
        let mut sum2 = 0.0_f64;
        for (fvals, bins) in &chunks {
            for (i, &fval) in fvals.iter().enumerate() {
                let fval2 = fval * fval;
                sum += fval;
                sum2 += fval2;
                for (dim, &k) in bins[i * ndim..(i + 1) * ndim].iter().enumerate() {
                    d[dim][usize::from(k)] += fval2;
                }
            }
        }

        let n = neval as f64;
        let mean = sum / n;
        let variance = ((sum2 / n - mean * mean) / (n - 1.0)).max(0.0);
        (mean, variance, d)
    }

    /// Reshape the grid using accumulated `f²` weights from one iteration.
    ///
    /// Each dimension is processed independently:
    /// 1. Normalize by expected hits per bin.
    /// 2. Apply Lepage's α-damping to smooth the importance function.
    /// 3. Smooth with nearest-neighbour averaging.
    /// 4. Redistribute bin edges so each new bin captures equal total weight.
    pub(crate) fn refine_grid(&mut self, d: &[Vec<f64>], neval: usize) {
        let nbins = self.nbins;
        let hits_per_bin = (neval / nbins).max(1) as f64;

        for (dim, d_dim) in d.iter().enumerate() {
            // Step 1: normalise by expected hits per bin.
            let mut m: Vec<f64> = d_dim
                .iter()
                .map(|&v| (v / hits_per_bin).max(1e-100))
                .collect();

            // Step 2: Lepage α-damping  m[k] → avg × ((m[k]/avg − 1) / ln(m[k]/avg))^α
            let avg = (m.iter().sum::<f64>() / nbins as f64).max(1e-100);
            for mk in &mut m {
                let ratio = (*mk / avg).max(1e-100);
                *mk = if (ratio - 1.0).abs() < 1e-7 {
                    avg
                } else {
                    avg * ((ratio - 1.0) / ratio.ln()).powf(self.alpha)
                };
                *mk = mk.max(1e-100);
            }

            // Step 3: nearest-neighbour smoothing.
            let m_copy = m.clone();
            for k in 0..nbins {
                let left = if k > 0 { m_copy[k - 1] } else { 0.0 };
                let right = if k + 1 < nbins { m_copy[k + 1] } else { 0.0 };
                m[k] = ((left + m_copy[k] + right) / 3.0).max(1e-100);
            }

            // Step 4: redistribute bin edges.
            let total: f64 = m.iter().sum();
            let target = total / nbins as f64; // weight per new bin

            let mut new_xi = vec![0.0_f64; nbins + 1];
            new_xi[0] = 0.0;
            new_xi[nbins] = 1.0;

            let mut acc = 0.0_f64;
            let mut k_old = 0_usize;
            let mut k_new = 1_usize;

            while k_new < nbins {
                while acc < target && k_old < nbins {
                    acc += m[k_old];
                    k_old += 1;
                }
                // k_old has overshot by `overshoot`; interpolate within old bin k_old-1.
                let overshoot = acc - target;
                let old_width = self.xi[dim][k_old] - self.xi[dim][k_old - 1];
                let frac = overshoot / m[k_old - 1];
                new_xi[k_new] =
                    (self.xi[dim][k_old] - frac * old_width).clamp(new_xi[k_new - 1], 1.0);
                acc = overshoot;
                k_new += 1;
            }

            self.xi[dim] = new_xi;
        }
    }
}

/// What one block draws in one iteration of a block-split adaptation.
///
/// A "block" is one grid sampled over its own coordinates — in the multichannel
/// integrand's case, one channel with the channel frozen. Blocks never share a
/// point, so a block's numbers depend on its own plan alone and on nothing about
/// how the blocks were scheduled.
#[derive(Debug, Clone, Copy)]
pub struct BlockPlan {
    /// Points the block draws this iteration.
    pub neval: usize,
    /// How many points the block has already drawn in this run, which is where
    /// its generator seeks to. Iterations of one block must therefore be planned
    /// with a running total, not with `iteration × neval`, once `neval` varies.
    pub first_point: u64,
    /// The block's `ChaCha8Rng` stream id.
    pub stream: u64,
    /// Points one rayon task evaluates. Scheduling only — the result is
    /// identical at any chunk size, for the reasons
    /// [`VegasGrid::adapt_parallel_seeded`] documents.
    pub chunk_size: usize,
}

/// One block's estimate from one iteration, with the refinement histogram that
/// iteration accumulated.
#[derive(Debug, Clone)]
pub struct BlockIteration {
    /// The block's integral estimate over its own coordinates.
    pub integral: f64,
    /// Variance of that estimate (not of a single point).
    pub variance: f64,
    /// Points whose integrand value was not exactly zero.
    ///
    /// For an integrand that returns a hard zero on every point its cuts reject —
    /// what a cut-first evaluation does — this counts the points that contributed
    /// to the estimate at all, and `accepted / neval` is the block's acceptance.
    /// An integrand whose support had interior zeros would be undercounted by
    /// them, a set a continuous draw hits with probability zero.
    pub accepted: usize,
    /// `hist[dim][bin]` accumulating `(f·w)²`, the input
    /// [`VegasGrid::refine_grid`] reshapes the block's grid from.
    pub(crate) hist: Vec<Vec<f64>>,
}

impl BlockIteration {
    /// The per-*point* variance this iteration measured, the quantity a Neyman
    /// allocation compares across blocks. The estimate's variance is this
    /// divided by the points behind it.
    pub fn point_variance(&self, neval: usize) -> f64 {
        self.variance * neval as f64
    }
}

/// Run one iteration of every block in a **single** rayon region, scheduled by
/// `(block, chunk)`.
///
/// This is [`VegasGrid::adapt_parallel_seeded`]'s iteration body lifted over a set
/// of grids. Both contracts that make that function bit-for-bit the sequential
/// `adapt` are kept per block and are what make the scheduling inert:
///
/// * each chunk seeks its own generator to `first_point + offset` draws rather
///   than inheriting a predecessor's state, and
/// * a block's points are reduced in global point order on one thread, so the
///   sums and the histogram do not depend on where the chunk boundaries fell.
///
/// What changes is only *when* the work runs: one block per parallel region
/// leaves a narrow block unable to fill the pool, while all blocks' chunks in one
/// region schedule against each other. Nothing about a block's arithmetic can see
/// the difference.
///
/// `init` builds a block's per-chunk state from `(block, first global point of
/// the chunk)`; `f` evaluates one point of a block against it. Grids are **not**
/// refined here — the caller decides whether another iteration follows and
/// refines from [`BlockIteration::hist`].
pub fn adapt_blocks_iteration<S, Init, Fp>(
    grids: &[VegasGrid],
    plans: &[BlockPlan],
    seed: u64,
    init: Init,
    f: Fp,
) -> Vec<BlockIteration>
where
    Init: Fn(usize, u64) -> S + Sync,
    Fp: Fn(usize, &mut S, &[f64]) -> f64 + Sync,
    S: Send,
{
    assert_eq!(
        grids.len(),
        plans.len(),
        "one plan per grid: {} grid(s), {} plan(s)",
        grids.len(),
        plans.len()
    );
    // (block, chunk index within the block), block-major and ascending in chunk
    // index, so a block's chunk results come back contiguous and in point order.
    let tasks: Vec<(usize, usize)> = plans
        .iter()
        .enumerate()
        .flat_map(|(b, p)| {
            let nchunks = p.neval.div_ceil(p.chunk_size.max(1));
            (0..nchunks).map(move |c| (b, c))
        })
        .collect();

    let chunks: Vec<(Vec<f64>, Vec<u16>)> = tasks
        .par_iter()
        .map(|&(b, c)| {
            let grid = &grids[b];
            let plan = &plans[b];
            let ndim = grid.ndim;
            assert!(
                grid.nbins <= usize::from(u16::MAX) + 1,
                "a chunk carries its bin indices out as u16, so {} bins do not fit",
                grid.nbins
            );
            let chunk_size = plan.chunk_size.max(1);
            let offset = c * chunk_size;
            let this_chunk = chunk_size.min(plan.neval - offset);
            let chunk_first = plan.first_point + offset as u64;

            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            rng.set_stream(plan.stream);
            rng.set_word_pos(u128::from(chunk_first) * ndim as u128 * WORDS_PER_DRAW);

            let mut state = init(b, chunk_first);
            let mut x = vec![0.0_f64; ndim];
            let mut ks = vec![0_usize; ndim];
            let mut fvals = Vec::with_capacity(this_chunk);
            let mut bins = Vec::with_capacity(this_chunk * ndim);
            for _ in 0..this_chunk {
                let wgt = grid.draw_point(&mut rng, &mut x, &mut ks);
                fvals.push(f(b, &mut state, &x) * wgt);
                bins.extend(ks.iter().map(|&k| k as u16));
            }
            (fvals, bins)
        })
        .collect();

    let mut out = Vec::with_capacity(plans.len());
    let mut next = 0_usize;
    for (b, plan) in plans.iter().enumerate() {
        let grid = &grids[b];
        let ndim = grid.ndim;
        let nchunks = plan.neval.div_ceil(plan.chunk_size.max(1));
        let mut hist = vec![vec![0.0_f64; grid.nbins]; ndim];
        let mut sum = 0.0_f64;
        let mut sum2 = 0.0_f64;
        let mut accepted = 0_usize;
        for (fvals, bins) in &chunks[next..next + nchunks] {
            for (i, &fval) in fvals.iter().enumerate() {
                let fval2 = fval * fval;
                sum += fval;
                sum2 += fval2;
                accepted += usize::from(fval != 0.0);
                for (dim, &k) in bins[i * ndim..(i + 1) * ndim].iter().enumerate() {
                    hist[dim][usize::from(k)] += fval2;
                }
            }
        }
        next += nchunks;

        let n = plan.neval as f64;
        let mean = sum / n;
        let variance = ((sum2 / n - mean * mean) / (n - 1.0)).max(0.0);
        out.push(BlockIteration {
            integral: mean,
            variance,
            accepted,
            hist,
        });
    }
    out
}

/// Combine per-iteration `(integral, variance)` pairs under `rule`.
///
/// The χ²/dof is the same statistic either way — the surviving iterations'
/// scatter about the combined estimate in units of their own quoted errors —
/// since what it diagnoses is whether the iterations agree, not how they were
/// averaged.
pub(crate) fn combine_iterations(
    iter_results: &[(f64, f64)],
    rule: IterationCombination,
) -> VegasResult {
    let niter = iter_results.len();
    let (integral, std_dev) = match rule {
        IterationCombination::Unweighted => {
            let integral = iter_results.iter().map(|(i, _)| i).sum::<f64>() / niter as f64;
            let var: f64 = iter_results.iter().map(|(_, v)| v).sum::<f64>();
            (integral, var.sqrt() / niter as f64)
        }
        IterationCombination::InverseVariance => {
            let weight: f64 = iter_results.iter().map(|(_, v)| 1.0 / v).sum();
            let integral: f64 = iter_results.iter().map(|(i, v)| i / v).sum::<f64>() / weight;
            (integral, (1.0 / weight).sqrt())
        }
    };

    let chi2_per_dof = if niter > 1 {
        iter_results
            .iter()
            .map(|(i, v)| (i - integral).powi(2) / v)
            .sum::<f64>()
            / (niter - 1) as f64
    } else {
        0.0
    };

    VegasResult {
        integral,
        std_dev,
        chi2_per_dof,
    }
}

/// Substream id for `ChaCha8Rng::set_stream`, keyed by `(iteration,
/// chunk_index)`. `ChaCha8Rng` exposes 2⁶⁴ independent streams selectable
/// by a `u64` id, so packing `iter_idx` into the high 32 bits and
/// `chunk_idx` into the low 32 bits gives every `(iteration, chunk)` pair
/// its own structurally independent stream with no collisions for
/// realistic iteration/chunk counts. The same addressing scheme extends to
/// multi-machine sharding: a shard is just a chunk-index range.
fn substream_id(iter_idx: u32, chunk_idx: u32) -> u64 {
    ((iter_idx as u64) << 32) | chunk_idx as u64
}

/// Compatibility shim preserving the pre-split `Vegas::new` / `integrate`
/// API as a thin wrapper over [`VegasGrid::adapt`].
pub struct Vegas {
    grid: VegasGrid,
}

impl Vegas {
    /// Create a new VEGAS integrator with a uniform initial grid.
    ///
    /// * `ndim`  – integration dimensions
    /// * `nbins` – bins per dimension (50–100 is typical)
    /// * `alpha` – grid-damping exponent (Lepage: 1.5)
    pub fn new(ndim: usize, nbins: usize, alpha: f64) -> Self {
        Vegas {
            grid: VegasGrid::new(ndim, nbins, alpha),
        }
    }

    /// See [`VegasGrid::set_warmup`].
    pub fn with_warmup(mut self, warmup: usize) -> Self {
        self.grid.set_warmup(warmup);
        self
    }

    /// See [`VegasGrid::set_combination`].
    pub fn with_combination(mut self, combination: IterationCombination) -> Self {
        self.grid.set_combination(combination);
        self
    }

    /// Integrate `f` over `[0, 1]^ndim`. See [`VegasGrid::adapt`].
    pub fn integrate(
        &mut self,
        f: impl FnMut(&[f64]) -> f64,
        neval: usize,
        niter: usize,
        rng: &mut impl Rng,
    ) -> VegasResult {
        self.grid.adapt(f, neval, niter, rng)
    }
}

// ── Unit tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phasespace::rng::SubStream;
    use rand::SeedableRng;

    fn seeded_rng() -> impl Rng {
        rand::rngs::StdRng::seed_from_u64(12345)
    }

    /// Constant integrand: ∫₀¹ 1 du = 1.
    #[test]
    fn test_constant() {
        let mut v = Vegas::new(1, 50, 1.5);
        let mut rng = seeded_rng();
        let r = v.integrate(|_| 1.0, 10_000, 5, &mut rng);
        assert!(
            (r.integral - 1.0).abs() < 0.01,
            "constant: {:.6}",
            r.integral
        );
    }

    /// Linear: ∫₀¹ 2u du = 1.
    #[test]
    fn test_linear() {
        let mut v = Vegas::new(1, 50, 1.5);
        let mut rng = seeded_rng();
        let r = v.integrate(|u| 2.0 * u[0], 10_000, 5, &mut rng);
        assert!((r.integral - 1.0).abs() < 0.01, "linear: {:.6}", r.integral);
    }

    /// Quadratic: ∫₀¹ 3u² du = 1.
    #[test]
    fn test_quadratic() {
        let mut v = Vegas::new(1, 50, 1.5);
        let mut rng = seeded_rng();
        let r = v.integrate(|u| 3.0 * u[0] * u[0], 10_000, 5, &mut rng);
        assert!(
            (r.integral - 1.0).abs() < 0.02,
            "quadratic: {:.6}",
            r.integral
        );
    }

    /// Gaussian peak: ∫₀¹ exp(−(u−0.3)²/0.01) du = 0.1·(√π/2)·(erf(7)+erf(3)).
    ///
    /// Derivation: substitute t = (u−0.3)/0.1, du = 0.1·dt, limits t ∈ [−3, 7]:
    ///   = 0.1 · ∫₋₃⁷ exp(−t²) dt = 0.1 · (√π/2) · (erf(7) + erf(3))
    ///
    /// VEGAS should adapt to the peak and converge quickly.
    #[test]
    fn test_peaked() {
        let exact = {
            // Exact analytic result via erf substitution t = (u−0.3)/0.1.
            // ∫₀¹ exp(−(u−0.3)²/0.01) du = 0.1·(√π/2)·(erf(7)+erf(3))
            let half_sqrt_pi = std::f64::consts::PI.sqrt() / 2.0;
            0.1 * half_sqrt_pi * (libm::erf(7.0) + libm::erf(3.0))
        };
        let mut v = Vegas::new(1, 50, 1.5);
        let mut rng = seeded_rng();
        let r = v.integrate(
            |u| (-(u[0] - 0.3).powi(2) / 0.01).exp(),
            50_000,
            10,
            &mut rng,
        );
        let rel = (r.integral - exact).abs() / exact;
        assert!(
            rel < 0.01,
            "peaked: got {:.6}, exact {exact:.6}, rel {rel:.4}",
            r.integral
        );
    }

    /// 2D: ∫₀¹∫₀¹ (u + v) du dv = 1.
    #[test]
    fn test_2d() {
        let mut v = Vegas::new(2, 50, 1.5);
        let mut rng = seeded_rng();
        let r = v.integrate(|u| u[0] + u[1], 20_000, 5, &mut rng);
        assert!((r.integral - 1.0).abs() < 0.02, "2d: {:.6}", r.integral);
    }

    /// Asserts a value sits within `1e-12` relative of a pinned golden.
    ///
    /// The goldens are exact bit patterns captured on one platform, but the
    /// grid refinement runs `ln`/`powf` through the system libm, and system
    /// libms legitimately disagree at the last ulp; through the `sum2/n −
    /// mean²` cancellation that shows up as a relative drift measured at
    /// ≤ 2.4e-15 in `std_dev` between the two platforms this suite runs on.
    /// The `1e-12` bound keeps ~400× headroom over that noise while sitting
    /// orders of magnitude below what any change to draw order, accumulation
    /// order, or the refinement algorithm produces (≥ 1e-6 in practice), so
    /// the golden still guards the refactor it was captured for.
    fn assert_matches_golden(got: f64, golden_bits: u64, what: &str) {
        let golden = f64::from_bits(golden_bits);
        let rel = ((got - golden) / golden).abs();
        assert!(
            rel < 1e-12,
            "{what}: got {got:.17e} ({:#018x}), golden {golden:.17e} ({golden_bits:#018x}), rel {rel:.2e}",
            got.to_bits()
        );
    }

    /// Pinned-seed regression golden, captured from the pre-split
    /// monolithic `Vegas::integrate` implementation. Guards the refactor:
    /// any change to draw order, accumulation order, or the refinement
    /// algorithm would move these values far beyond the golden tolerance.
    ///
    /// Driven at `warmup = 0`, the unfiltered `1/σ²` combination the goldens
    /// were captured under, so they keep guarding exactly what they were
    /// captured for. The discard's own effect on the same seed is pinned by
    /// [`test_warmup_discards_leading_iterations`].
    #[test]
    fn test_pinned_seed_regression_shim() {
        let mut v = Vegas::new(2, 50, 1.5)
            .with_warmup(0)
            .with_combination(IterationCombination::InverseVariance);
        let mut rng = rand::rngs::StdRng::seed_from_u64(999);
        let r = v.integrate(|u| u[0] * u[0] + u[1], 5000, 4, &mut rng);
        assert_matches_golden(r.integral, 4605706486304428084, "integral");
        assert_matches_golden(r.std_dev, 4564401184564159150, "std_dev");
        assert_matches_golden(r.chi2_per_dof, 4605496727683902589, "chi2_per_dof");
    }

    /// Same golden, driven directly through `VegasGrid::adapt` (bypassing
    /// the `Vegas` shim) to pin the new entry point independently.
    #[test]
    fn test_pinned_seed_regression_grid_adapt() {
        let mut grid = VegasGrid::new(2, 50, 1.5)
            .with_warmup(0)
            .with_combination(IterationCombination::InverseVariance);
        let mut rng = rand::rngs::StdRng::seed_from_u64(999);
        let r = grid.adapt(|u| u[0] * u[0] + u[1], 5000, 4, &mut rng);
        assert_matches_golden(r.integral, 4605706486304428084, "integral");
        assert_matches_golden(r.std_dev, 4564401184564159150, "std_dev");
        assert_matches_golden(r.chi2_per_dof, 4605496727683902589, "chi2_per_dof");
    }

    // ── Warm-up discard ─────────────────────────────────────────────────

    /// The discard changes nothing about sampling: the grid a run ends on, and
    /// the last iterations' estimates, are the ones a `warmup = 0` run of the
    /// same seed produced. What changes is which of them the combination sees.
    #[test]
    fn test_warmup_discards_leading_iterations() {
        let f = |u: &[f64]| (-(u[0] - 0.3).powi(2) / 0.01).exp() + u[1];

        // Per-iteration estimates, recovered by running 1..=niter iterations
        // from the same seed: iteration k's own (integral, σ) is what a
        // `warmup = k-1` run over k iterations reports as a single-iteration
        // combination.
        let per_iter: Vec<(f64, f64)> = (1..=6)
            .map(|k| {
                let mut grid = VegasGrid::new(2, 50, 1.5).with_warmup(k - 1);
                let mut rng = rand::rngs::StdRng::seed_from_u64(4242);
                let r = grid.adapt(f, 4000, k, &mut rng);
                (r.integral, r.std_dev)
            })
            .collect();

        for rule in [
            IterationCombination::Unweighted,
            IterationCombination::InverseVariance,
        ] {
            for warmup in 0..6 {
                let mut grid = VegasGrid::new(2, 50, 1.5)
                    .with_warmup(warmup)
                    .with_combination(rule);
                let mut rng = rand::rngs::StdRng::seed_from_u64(4242);
                let got = grid.adapt(f, 4000, 6, &mut rng);
                let kept: Vec<(f64, f64)> = per_iter[warmup..]
                    .iter()
                    .map(|&(i, s)| (i, s * s))
                    .collect();
                let want = combine_iterations(&kept, rule);
                assert!(
                    (got.integral - want.integral).abs() <= 1e-12 * want.integral.abs(),
                    "{rule:?} warmup {warmup}: integral {} vs {}",
                    got.integral,
                    want.integral
                );
                assert!(
                    (got.std_dev - want.std_dev).abs() <= 1e-12 * want.std_dev,
                    "{rule:?} warmup {warmup}: std_dev {} vs {}",
                    got.std_dev,
                    want.std_dev
                );
            }
        }
    }

    /// The unweighted rule is the arithmetic mean of the surviving iterations,
    /// and quotes the error that mean actually has.
    #[test]
    fn test_unweighted_combination_is_the_arithmetic_mean() {
        let r = combine_iterations(
            &[(10.0, 1.0), (20.0, 4.0), (30.0, 4.0)],
            IterationCombination::Unweighted,
        );
        assert_eq!(r.integral, 20.0);
        assert!((r.std_dev - 9.0f64.sqrt() / 3.0).abs() < 1e-15, "{r:?}");

        // The same three iterations under Lepage's rule are pulled toward the
        // precise one, which is the correlation the default avoids.
        let w = combine_iterations(
            &[(10.0, 1.0), (20.0, 4.0), (30.0, 4.0)],
            IterationCombination::InverseVariance,
        );
        assert!((w.integral - 15.0).abs() < 1e-13, "{w:?}");
    }

    /// The grid a run trains does not depend on how many iterations the
    /// combination keeps — the discard is an estimator change, not a sampling
    /// change.
    #[test]
    fn test_warmup_leaves_the_trained_grid_untouched() {
        let f = |u: &[f64]| (-(u[0] - 0.7).powi(2) / 0.004).exp();
        let mut baseline = VegasGrid::new(1, 50, 1.5).with_warmup(0);
        let mut rng = rand::rngs::StdRng::seed_from_u64(31337);
        baseline.adapt(f, 3000, 5, &mut rng);

        for warmup in [1usize, 3, 4] {
            let mut grid = VegasGrid::new(1, 50, 1.5).with_warmup(warmup);
            let mut rng = rand::rngs::StdRng::seed_from_u64(31337);
            grid.adapt(f, 3000, 5, &mut rng);
            assert_eq!(grid.xi(), baseline.xi(), "warmup {warmup} moved the grid");
        }
    }

    /// A `warmup` at or above `niter` keeps the last iteration rather than
    /// combining nothing.
    #[test]
    fn test_warmup_clamped_to_leave_one_iteration() {
        let f = |u: &[f64]| 3.0 * u[0] * u[0];
        let mut clamped = VegasGrid::new(1, 50, 1.5).with_warmup(99);
        let mut rng = rand::rngs::StdRng::seed_from_u64(5);
        let got = clamped.adapt(f, 5000, 4, &mut rng);

        let mut last_only = VegasGrid::new(1, 50, 1.5).with_warmup(3);
        let mut rng = rand::rngs::StdRng::seed_from_u64(5);
        let want = last_only.adapt(f, 5000, 4, &mut rng);

        assert!(got.integral.is_finite() && got.std_dev.is_finite());
        assert_eq!(got.integral.to_bits(), want.integral.to_bits());
        assert_eq!(got.std_dev.to_bits(), want.std_dev.to_bits());
        assert_eq!(got.chi2_per_dof, 0.0);
    }

    /// The warm-up count is a run-time knob, not part of a stored grid: a
    /// round-trip carries the trained edges and comes back with no warm-up.
    #[test]
    fn test_warmup_is_not_serialized() {
        let grid = VegasGrid::new(2, 20, 1.5).with_warmup(3);
        let bytes = bincode::serialize(&grid).expect("bincode serialize");
        let back: VegasGrid = bincode::deserialize(&bytes).expect("bincode deserialize");
        assert_eq!(back.warmup(), 0);
        assert_eq!(back.xi(), grid.xi());

        let plain = VegasGrid::new(2, 20, 1.5).with_warmup(0);
        assert_eq!(
            bytes,
            bincode::serialize(&plain).expect("bincode serialize"),
            "the warm-up count reached the wire format"
        );
    }

    // ── Serde round-trip / validation ───────────────────────────────────

    #[test]
    fn test_serde_roundtrip_bincode() {
        let mut grid = VegasGrid::new(2, 20, 1.5);
        let mut rng = seeded_rng();
        grid.adapt(|u| u[0] + u[1], 2_000, 3, &mut rng);

        let bytes = bincode::serialize(&grid).expect("bincode serialize");
        let restored: VegasGrid = bincode::deserialize(&bytes).expect("bincode deserialize");
        assert_eq!(restored.ndim(), grid.ndim());
        assert_eq!(restored.nbins(), grid.nbins());
        assert_eq!(restored.alpha(), grid.alpha());
        assert_eq!(restored.xi(), grid.xi());
    }

    #[test]
    fn test_serde_roundtrip_json() {
        let mut grid = VegasGrid::new(2, 20, 1.5);
        let mut rng = seeded_rng();
        grid.adapt(|u| u[0] + u[1], 2_000, 3, &mut rng);

        let json = serde_json::to_string(&grid).expect("json serialize");
        let restored: VegasGrid = serde_json::from_str(&json).expect("json deserialize");
        assert_eq!(restored.ndim(), grid.ndim());
        assert_eq!(restored.nbins(), grid.nbins());
        assert_eq!(restored.alpha(), grid.alpha());
        assert_eq!(restored.xi(), grid.xi());
    }

    #[test]
    fn test_from_raw_rejects_non_monotone() {
        let err = VegasGrid::from_raw(1, 3, 1.5, vec![vec![0.0, 0.9, 0.5, 1.0]]).unwrap_err();
        assert!(matches!(err, VegasGridError::NonMonotone { .. }));
    }

    #[test]
    fn test_from_raw_rejects_bad_endpoint() {
        let err = VegasGrid::from_raw(1, 2, 1.5, vec![vec![0.1, 0.5, 1.0]]).unwrap_err();
        assert!(matches!(err, VegasGridError::BadEndpoint { .. }));

        let err = VegasGrid::from_raw(1, 2, 1.5, vec![vec![0.0, 0.5, 0.9]]).unwrap_err();
        assert!(matches!(err, VegasGridError::BadEndpoint { .. }));
    }

    #[test]
    fn test_from_raw_rejects_shape_mismatch() {
        let err = VegasGrid::from_raw(2, 2, 1.5, vec![vec![0.0, 0.5, 1.0]]).unwrap_err();
        assert!(matches!(err, VegasGridError::ShapeMismatch { .. }));
    }

    #[test]
    fn test_from_raw_rejects_bin_count_mismatch() {
        let err = VegasGrid::from_raw(1, 3, 1.5, vec![vec![0.0, 0.5, 1.0]]).unwrap_err();
        assert!(matches!(err, VegasGridError::BinCountMismatch { .. }));
    }

    #[test]
    fn test_from_raw_rejects_zero_dims_or_bins() {
        assert!(matches!(
            VegasGrid::from_raw(0, 2, 1.5, vec![]).unwrap_err(),
            VegasGridError::ZeroDims
        ));
        assert!(matches!(
            VegasGrid::from_raw(1, 0, 1.5, vec![vec![]]).unwrap_err(),
            VegasGridError::ZeroBins
        ));
    }

    /// Mirrors `VegasGridRaw`'s field order/types so a corrupt payload can
    /// be constructed without going through the validating constructor.
    #[derive(Serialize)]
    struct RawGridForTest {
        ndim: usize,
        nbins: usize,
        alpha: f64,
        xi: Vec<Vec<f64>>,
    }

    #[test]
    fn test_deserialize_rejects_corrupt_bincode() {
        let bad = RawGridForTest {
            ndim: 1,
            nbins: 3,
            alpha: 1.5,
            xi: vec![vec![0.0, 0.9, 0.5, 1.0]], // non-monotone
        };
        let bytes = bincode::serialize(&bad).unwrap();
        let result: Result<VegasGrid, _> = bincode::deserialize(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_rejects_corrupt_json() {
        let json = r#"{"ndim":1,"nbins":3,"alpha":1.5,"xi":[[0.0,0.9,0.5,1.0]]}"#;
        let result: Result<VegasGrid, _> = serde_json::from_str(json);
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("increasing") || err.contains("monoton"),
            "unexpected error message: {err}"
        );
    }

    // ── Frozen-grid sampling vs adapt-phase estimate ────────────────────

    #[test]
    fn test_frozen_agrees_with_adapt() {
        let mut grid = VegasGrid::new(1, 50, 1.5);
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        let adapt_result = grid.adapt(|u| 3.0 * u[0] * u[0], 20_000, 8, &mut rng);

        let mut frozen_rng = rand::rngs::StdRng::seed_from_u64(1234);
        let frozen_result = grid.sample_frozen(|u| 3.0 * u[0] * u[0], 50_000, &mut frozen_rng);

        assert!(
            (adapt_result.integral - 1.0).abs() < 0.02,
            "adapt: {:.6}",
            adapt_result.integral
        );
        assert!(
            (frozen_result.integral - 1.0).abs() < 0.02,
            "frozen: {:.6}",
            frozen_result.integral
        );
        let combined_err = (adapt_result.std_dev.powi(2) + frozen_result.std_dev.powi(2)).sqrt();
        assert!(
            (adapt_result.integral - frozen_result.integral).abs() < 5.0 * combined_err,
            "adapt {:.6} vs frozen {:.6}, combined_err {:.6}",
            adapt_result.integral,
            frozen_result.integral,
            combined_err
        );
    }

    // ── Batched vs unbatched bit-identity ───────────────────────────────

    #[test]
    fn test_adapt_batched_matches_unbatched() {
        let seed = 42;
        let f = |u: &[f64]| u[0] * u[0] + 2.0 * u[1];

        let mut grid_scalar = VegasGrid::new(2, 30, 1.5);
        let mut rng_scalar = rand::rngs::StdRng::seed_from_u64(seed);
        let r_scalar = grid_scalar.adapt(f, 4000, 3, &mut rng_scalar);

        for batch_size in [2usize, 7, 64, 4000] {
            let mut grid_batched = VegasGrid::new(2, 30, 1.5);
            let mut rng_batched = rand::rngs::StdRng::seed_from_u64(seed);
            let r_batched = grid_batched.adapt_batched(
                |points, out| {
                    for (p, o) in points.iter().zip(out.iter_mut()) {
                        *o = f(p.u);
                    }
                },
                4000,
                3,
                batch_size,
                &mut rng_batched,
            );
            assert_eq!(
                r_scalar.integral.to_bits(),
                r_batched.integral.to_bits(),
                "batch_size={batch_size}"
            );
            assert_eq!(
                r_scalar.std_dev.to_bits(),
                r_batched.std_dev.to_bits(),
                "batch_size={batch_size}"
            );
            assert_eq!(
                grid_scalar.xi(),
                grid_batched.xi(),
                "batch_size={batch_size}"
            );
        }
    }

    #[test]
    fn test_sample_frozen_batched_matches_unbatched() {
        let grid = VegasGrid::new(1, 40, 1.5);
        let f = |u: &[f64]| (u[0] - 0.5).abs();
        let seed = 314;

        let mut rng_scalar = rand::rngs::StdRng::seed_from_u64(seed);
        let r_scalar = grid.sample_frozen(f, 3000, &mut rng_scalar);

        for batch_size in [1usize, 8, 3000] {
            let mut rng_batched = rand::rngs::StdRng::seed_from_u64(seed);
            let r_batched = grid.sample_frozen_batched(
                |points, out| {
                    for (p, o) in points.iter().zip(out.iter_mut()) {
                        *o = f(p.u);
                    }
                },
                3000,
                batch_size,
                &mut rng_batched,
            );
            assert_eq!(
                r_scalar.integral.to_bits(),
                r_batched.integral.to_bits(),
                "batch_size={batch_size}"
            );
        }
    }

    // ── Thread-count bit-identity ────────────────────────────────────────

    fn run_with_threads<T>(nthreads: usize, work: impl FnOnce() -> T + Send) -> T
    where
        T: Send,
    {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(nthreads)
            .build()
            .unwrap();
        pool.install(work)
    }

    /// The public per-point draw reproduces the sampling `sample_frozen` performs
    /// internally: same points, and a mean of `weight · f` equal to the frozen
    /// pass's integral. A `draw` that diverged from the internal one would make an
    /// accept/reject pass sample a different density than the integral it is
    /// normalised against.
    #[test]
    fn test_draw_matches_sample_frozen() {
        let mut grid = VegasGrid::new(2, 40, 1.5);
        let mut warm = rand::rngs::StdRng::seed_from_u64(5);
        grid.adapt(|u| 3.0 * u[0] * u[0] + u[1], 5_000, 5, &mut warm);

        let f = |u: &[f64]| 3.0 * u[0] * u[0] + u[1];
        let n = 20_000;

        let mut rng_frozen = rand::rngs::StdRng::seed_from_u64(31);
        let frozen = grid.sample_frozen(f, n, &mut rng_frozen);

        let mut rng_draw = rand::rngs::StdRng::seed_from_u64(31);
        let mut x = vec![0.0; grid.ndim()];
        let mut sum = 0.0;
        for _ in 0..n {
            let w = grid.draw(&mut rng_draw, &mut x);
            sum += w * f(&x);
        }
        assert_eq!((sum / n as f64).to_bits(), frozen.integral.to_bits());
    }

    #[test]
    fn test_adapt_parallel_thread_count_invariant() {
        let f = |u: &[f64]| u[0] * u[0] + 2.0 * u[1] * u[1];
        let seed = 2026;

        let r1 = run_with_threads(1, || {
            let mut grid = VegasGrid::new(2, 25, 1.5);
            grid.adapt_parallel(f, 5000, 3, seed, 128)
        });
        let r4 = run_with_threads(4, || {
            let mut grid = VegasGrid::new(2, 25, 1.5);
            grid.adapt_parallel(f, 5000, 3, seed, 128)
        });

        assert_eq!(r1.integral.to_bits(), r4.integral.to_bits());
        assert_eq!(r1.std_dev.to_bits(), r4.std_dev.to_bits());
        assert_eq!(r1.chi2_per_dof.to_bits(), r4.chi2_per_dof.to_bits());
    }

    #[test]
    fn test_sample_frozen_parallel_thread_count_invariant() {
        let f = |u: &[f64]| (u[0] - 0.3).abs() + u[1];
        let seed = 777;
        let grid = VegasGrid::new(2, 25, 1.5);

        let r1 = run_with_threads(1, || grid.sample_frozen_parallel(f, 6000, seed, 97));
        let r8 = run_with_threads(8, || grid.sample_frozen_parallel(f, 6000, seed, 97));

        assert_eq!(r1.integral.to_bits(), r8.integral.to_bits());
        assert_eq!(r1.std_dev.to_bits(), r8.std_dev.to_bits());
    }

    #[test]
    fn test_parallel_agrees_with_sequential_statistically() {
        let f = |u: &[f64]| 3.0 * u[0] * u[0];
        let seed = 55;

        let mut grid_seq = VegasGrid::new(1, 50, 1.5);
        let mut rng_seq = rand::rngs::StdRng::seed_from_u64(seed);
        let r_seq = grid_seq.adapt(f, 20_000, 5, &mut rng_seq);

        let mut grid_par = VegasGrid::new(1, 50, 1.5);
        let r_par = grid_par.adapt_parallel(f, 20_000, 5, seed, 500);

        assert!((r_seq.integral - 1.0).abs() < 0.02);
        assert!((r_par.integral - 1.0).abs() < 0.02);
    }

    // ── Seeked-substream parallelism: identical to the sequential form ───

    /// A point consumes exactly `ndim` 64-bit draws — the arithmetic the seeked
    /// parallel path addresses chunks with. Pinned directly on the generator
    /// rather than inferred, because a change in how `rand` renders an `f64`
    /// would silently shift every chunk's starting point.
    #[test]
    fn test_a_point_costs_ndim_draws() {
        for ndim in 1..5 {
            let grid = VegasGrid::new(ndim, 32, 1.5);
            let mut rng = ChaCha8Rng::seed_from_u64(9);
            rng.set_stream(3);
            rng.set_word_pos(0);
            let mut x = vec![0.0; ndim];
            let mut ks = vec![0_usize; ndim];
            for point in 1..4_u128 {
                grid.draw_point(&mut rng, &mut x, &mut ks);
                assert_eq!(
                    rng.get_word_pos(),
                    point * ndim as u128 * WORDS_PER_DRAW,
                    "ndim={ndim}"
                );
            }
        }
    }

    /// The seeked parallel adaptation is the sequential one, bit for bit, at any
    /// chunk size and any pool size — the property that lets the validation layer
    /// run single-threaded and still measure what the parallel CLI produces.
    #[test]
    fn test_adapt_parallel_seeded_is_the_sequential_adapt() {
        let f = |u: &[f64]| (u[0] * 3.0).exp() * (1.0 + u[1]) / (0.05 + u[2]);
        let seed = 0x5EED_1;
        let stream = 0xC7A0_0000;
        let (neval, niter) = (4000, 5);

        let mut grid_seq = VegasGrid::new(3, 64, 1.5);
        let mut seq_extra = SubStream::from_stream(seed, 0x5CA1_0000);
        let mut seq_tail = [0.0_f64; 2];
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        rng.set_stream(stream);
        rng.set_word_pos(0);
        let r_seq = grid_seq.adapt(
            |u| {
                seq_extra.fill_uniforms(&mut seq_tail);
                f(u) * (1.0 + seq_tail[0] + seq_tail[1])
            },
            neval,
            niter,
            &mut rng,
        );

        for nthreads in [1, 3, 8] {
            for chunk in [1, 7, 512, neval, neval * 2] {
                let r_par = run_with_threads(nthreads, || {
                    let mut grid = VegasGrid::new(3, 64, 1.5);
                    let r = grid.adapt_parallel_seeded(
                        |first| (SubStream::new(seed, 0x5CA1_0000, first * 2), [0.0_f64; 2]),
                        |(extra, tail), u| {
                            extra.fill_uniforms(tail);
                            f(u) * (1.0 + tail[0] + tail[1])
                        },
                        neval,
                        niter,
                        seed,
                        stream,
                        chunk,
                    );
                    (r, grid)
                });
                assert_eq!(
                    r_par.0.integral.to_bits(),
                    r_seq.integral.to_bits(),
                    "integral at {nthreads} threads, chunk {chunk}"
                );
                assert_eq!(
                    r_par.0.std_dev.to_bits(),
                    r_seq.std_dev.to_bits(),
                    "std_dev at {nthreads} threads, chunk {chunk}"
                );
                assert_eq!(
                    r_par.0.chi2_per_dof.to_bits(),
                    r_seq.chi2_per_dof.to_bits(),
                    "chi2 at {nthreads} threads, chunk {chunk}"
                );
                // The trained grid, not only the estimate: a refinement fed
                // differently-rounded histograms would still agree on the first
                // iteration's integral and diverge from there.
                for (dim, (a, b)) in r_par.1.xi().iter().zip(grid_seq.xi()).enumerate() {
                    for (k, (ea, eb)) in a.iter().zip(b).enumerate() {
                        assert_eq!(
                            ea.to_bits(),
                            eb.to_bits(),
                            "edge {k} of dim {dim} at {nthreads} threads, chunk {chunk}"
                        );
                    }
                }
            }
        }
    }

    /// The `(block, chunk)` scheduler is the per-block `adapt_parallel_seeded`,
    /// bit for bit, at any chunk size and pool size.
    ///
    /// Three blocks of deliberately unequal width — one of them narrow enough
    /// that on its own it could not fill a pool — driven iteration-major through
    /// one parallel region per iteration, against the same three blocks driven
    /// block-major with a parallel region each. Both the estimates and the
    /// trained grids must agree exactly: a histogram summed in a different order
    /// would still match on the first iteration and diverge from the second.
    #[test]
    fn test_adapt_blocks_iteration_is_the_per_block_adaptation() {
        let f = |b: usize, u: &[f64]| {
            (u[0] * (2.0 + b as f64)).exp() * (1.0 + u[1]) / (0.05 + u[2] + 0.1 * b as f64)
        };
        let seed = 0x5EED_B10C;
        let streams = [0xC7A0_0000_u64, 0xC7A0_0001, 0xC7A0_0002];
        let nevals = [4000_usize, 517, 1300];
        let niter = 5;
        let ndim = 3;

        // Block-major: each block gets its own adaptation, as the per-channel
        // loop this replaces did.
        let mut want_results = Vec::new();
        let mut want_grids = Vec::new();
        for b in 0..3 {
            let mut grid = VegasGrid::new(ndim, 64, 1.5);
            let r = grid.adapt_parallel_seeded(
                |first| {
                    (
                        SubStream::new(seed, 0x5CA1_0000 + b as u64, first * 2),
                        [0.0_f64; 2],
                    )
                },
                |(extra, tail), u| {
                    extra.fill_uniforms(tail);
                    f(b, u) * (1.0 + tail[0] + tail[1])
                },
                nevals[b],
                niter,
                seed,
                streams[b],
                256,
            );
            want_results.push(r);
            want_grids.push(grid);
        }

        for nthreads in [1, 3, 8] {
            for chunk in [1, 7, 512, 4000, 9000] {
                let (got_results, got_grids) = run_with_threads(nthreads, || {
                    let mut grids: Vec<VegasGrid> =
                        (0..3).map(|_| VegasGrid::new(ndim, 64, 1.5)).collect();
                    let mut drawn = [0_u64; 3];
                    let mut kept: Vec<Vec<(f64, f64)>> = vec![Vec::new(); 3];
                    for iter_idx in 0..niter {
                        let plans: Vec<BlockPlan> = (0..3)
                            .map(|b| BlockPlan {
                                neval: nevals[b],
                                first_point: drawn[b],
                                stream: streams[b],
                                chunk_size: chunk,
                            })
                            .collect();
                        let out = adapt_blocks_iteration(
                            &grids,
                            &plans,
                            seed,
                            |b, first| {
                                (
                                    SubStream::new(seed, 0x5CA1_0000 + b as u64, first * 2),
                                    [0.0_f64; 2],
                                )
                            },
                            |b, (extra, tail): &mut (SubStream, [f64; 2]), u| {
                                extra.fill_uniforms(tail);
                                f(b, u) * (1.0 + tail[0] + tail[1])
                            },
                        );
                        for b in 0..3 {
                            drawn[b] += nevals[b] as u64;
                            if iter_idx >= grids[b].warmup() {
                                kept[b].push((
                                    out[b].integral,
                                    out[b].variance.max(f64::MIN_POSITIVE),
                                ));
                            }
                            if iter_idx + 1 < niter {
                                grids[b].refine_grid(&out[b].hist, nevals[b]);
                            }
                        }
                    }
                    let results: Vec<VegasResult> = kept
                        .iter()
                        .map(|k| combine_iterations(k, IterationCombination::default()))
                        .collect();
                    (results, grids)
                });

                for b in 0..3 {
                    let (g, w) = (&got_results[b], &want_results[b]);
                    assert_eq!(
                        g.integral.to_bits(),
                        w.integral.to_bits(),
                        "block {b} integral at {nthreads} threads, chunk {chunk}"
                    );
                    assert_eq!(
                        g.std_dev.to_bits(),
                        w.std_dev.to_bits(),
                        "block {b} std_dev at {nthreads} threads, chunk {chunk}"
                    );
                    assert_eq!(
                        g.chi2_per_dof.to_bits(),
                        w.chi2_per_dof.to_bits(),
                        "block {b} chi2 at {nthreads} threads, chunk {chunk}"
                    );
                    for (dim, (a, e)) in
                        got_grids[b].xi().iter().zip(want_grids[b].xi()).enumerate()
                    {
                        for (k, (ea, eb)) in a.iter().zip(e).enumerate() {
                            assert_eq!(
                                ea.to_bits(),
                                eb.to_bits(),
                                "block {b} edge {k} of dim {dim} at {nthreads} threads, chunk {chunk}"
                            );
                        }
                    }
                }
            }
        }
    }
}
