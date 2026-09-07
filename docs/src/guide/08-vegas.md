# VEGAS

A phase-space [map](07-phase-space.md#maps-from-the-unit-hypercube) turns
the integral into one over the unit hypercube,
$\int_{[0,1]^d} f(u)\, du$, and the crude Monte Carlo estimate is the
average of $f$ over $N$ uniform points, with an error of
$\sqrt{\mathrm{Var}(f)/N}$. The error falls as $1/\sqrt N$ whatever
the dimension, which is why Monte Carlo is the only option in $3n-4$
dimensions, and it is proportional to the standard deviation of the
integrand, which is why the sampling density matters. **Importance
sampling** draws points from a density $g(u)$ instead of uniformly and
averages $f/g$; the variance is then that of $f/g$, which vanishes
when $g \propto |f|$. VEGAS ([Lepage 1978](../bibliography.md#phase-space-and-integration))
is an algorithm for learning such a $g$ from the integrand itself.

## The algorithm

VEGAS restricts $g$ to a product of one-dimensional step functions.
Each axis is divided into bins of equal probability but variable width;
sampling draws a bin uniformly and a point uniformly within it, so the
density in a bin is inversely proportional to its width. An iteration:

1. Draw $N$ points from the current grid, accumulate the estimate of
   the integral and its variance, and accumulate per bin along each axis
   the sum of $f^2$ seen there.
2. Refine the grid: move the bin boundaries so that each bin captures an
   equal share of the accumulated importance, shrinking bins where the
   integrand was large. Lepage's damping, a smoothing of the importance
   with a power $\alpha$, keeps one iteration's noise from overreacting.

Repeated for some iterations, the grid converges toward the separable
approximation of $|f|$. Separability is the limitation: a peak along a
diagonal cannot be resolved by a product density, and that is what the
[multichannel](09-multichannel.md) maps are for, each channel making its
own peak axis-aligned in its own coordinates. Classic VEGAS is importance
sampling only. Its successor, VEGAS+
([Lepage 2021](../bibliography.md#phase-space-and-integration)), adds
adaptive stratification. The crate implements the classic algorithm. The
channel decomposition takes care of the diagonal structure that
stratification helps with most, and the point budget is already
stratified across channels, but whether VEGAS+'s within-channel
stratification would still buy convergence speed on these integrands has
not been measured; it is a tracked research item.

## Combining iterations

Each iteration's estimate is unbiased whatever grid it ran on, because
importance sampling divides by the density it drew from. Lepage's rule for
combining the iterations, a mean weighted by $1/\sigma_i^2$, is *not*
unbiased, and at the point counts a multichannel run can afford per channel
the bias is the dominant error. The weight of an iteration is estimated from
the same samples that produced its integral: an iteration that undersamples
the peak returns a low integral and a low variance together, and the
combination weights that low answer up.

The effect is measured in the module documentation on a five-dimensional
Gaussian with a known integral, over four thousand seeds. At two thousand
points per iteration Lepage's rule is 1.2% low while claiming a 0.7%
error; the plain arithmetic mean of the iterations is unbiased at the same
spread. Two knobs therefore depart from Lepage's defaults:

- **Warm-up.** The first two iterations still draw points and still refine
  the grid, but their estimates are excluded from the combination. They ran
  on a grid that had not found the peak yet and are the noisy ones; the
  measured spread has a broad minimum at one or two discarded iterations.
- **Unweighted combination.** The surviving iterations are averaged with
  equal weights, quoting $\sqrt{\sum \sigma_i^2}/n$. Fixed weights cannot
  correlate with what they weight. The inverse-variance rule remains
  available, and the [stopping rule](09-multichannel.md#stopping) refuses
  to read an error bar produced by it.

The warm-up discard alone does not remove the bias, since the correlation
is in every iteration; the unweighted mean does, and the discard is what
makes it affordable.

## Adapt, then freeze

An adapted grid serialises. Integration is the phase that adapts; event
generation reloads the frozen grid and samples from it with no further
refinement, which is what makes a [`generate`](../cli/overview.md) job a
pure function of the artifact and a seed. The sampling itself is chunked
deterministically over the counter-based [random streams](07-phase-space.md#random-numbers),
so a parallel adaptation produces the same grid as a serial one.

## What the diagnostics mean

The $\chi^2$ per degree of freedom across iterations measures whether
the iterations agree with each other to within their own quoted errors. A
value well above one says the quoted errors are optimistic, which on
heavy-tailed integrands is the norm rather than the exception; a value
well below one is as loud a warning, since it says the errors are
overestimated or the iterations correlated. The
[multichannel](09-multichannel.md#stopping) stopping rule folds the
measured $\chi^2$ into the error it tests against.

## Where it lives

[`vibegraph::vegas`](../api/vibegraph/vegas/index.html):
[`Vegas`](../api/vibegraph/vegas/struct.Vegas.html) is the integrator,
[`VegasGrid`](../api/vibegraph/vegas/struct.VegasGrid.html) the grid with
its `adapt` and `sample_frozen` phases, and
[`IterationCombination`](../api/vibegraph/vegas/enum.IterationCombination.html)
the combination rule. The module documentation carries the bias
measurement in full.
