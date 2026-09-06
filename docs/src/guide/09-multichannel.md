# Multichannel integration

A single [channel](07-phase-space.md#channels-from-diagrams) flattens the
peaks of one diagram. A process has many diagrams with peaks in different
invariants, and no single map is good for all of them. The **multichannel**
method ([Kleiss & Pittau 1994](../bibliography.md#phase-space-and-integration),
[Maltoni & Stelzer 2003](../bibliography.md#the-pipeline-as-a-whole))
samples from a mixture of the channels' densities,

\\[
g(p) = \sum_j \alpha_j\, g_j(p), \qquad \sum_j \alpha_j = 1,\; \alpha_j \ge 0,
\\]

and weights each point by \\(f(p)/g(p)\\). Wherever any one channel's
density is large, the mixture is large, so every peak is covered as long
as some channel has it. The price is that computing the weight at a point
requires every channel's density at that point, not only the one that
generated it, which is why a channel exposes its density at an arbitrary
momentum configuration and not only at the points it produced. That
density sum is a measurable fraction of the per-point cost on a process
with hundreds of channels, and it is evaluated only after the cuts and the
matrix element, since a rejected point needs no weight.

## One grid per channel

The mixture is one integral over the unit hypercube with an extra
coordinate for the channel draw, and a VEGAS grid could adapt to it as a
whole. The same estimator also splits into one term per channel,

\\[
\int d\Phi\, f = \sum_j \int d\Phi\, f\,\frac{\alpha_j g_j}{g}
              = \sum_j \mathbb{E}_{p \sim g_j}\Big[\frac{\alpha_j f(p)}{g(p)}\Big],
\\]

each term sampled from its own channel alone and weighted by the same
combined \\(g\\). The two arrangements compute the same integral and differ
in what an importance grid in front of them can learn: a grid per term
adapts to a density conditional on the channel, which one separable grid
over the mixture cannot express. vibegraph banks one VEGAS grid per
channel, and this **hard split** with deterministic per-channel point counts
is what makes a channel's frozen grid the unit of event generation later.

## Adapting the channel weights

The \\(\alpha_j\\) are free, and the variance-minimising choice gives more
weight to channels whose region is where the integrand's variance is.
Kleiss and Pittau's adaptation reads each channel's share of the second
moment off a survey and moves the weights toward it; vibegraph runs it
before the grids are trained, and the weights then stay fixed while the
grids adapt, since re-adapting both at once would let each chase the other.

## Allocating points

With the integral split into per-channel terms, each iteration must decide
how many points every channel gets. For a fixed total, the
variance-minimising split is Neyman's, \\(N_j \propto s_j\\), with
\\(s_j\\) the standard deviation of channel \\(j\\)'s term. Spending
\\(N_j \propto \alpha_j\\) instead is nearly the same thing, because the
adapted \\(\alpha_j\\) already are most of a variance estimate; measured on
`p p > l+ l- j`, re-deriving the split from the trained grids improves the
variance at equal points by a factor between 1.00 and 1.22.

Both rules are floored. A channel whose map is the only one with density on
some structure has to keep sampling that structure even when its variance
estimate says it is cheap, because a channel that stops covering its own
region is exactly the [confidently wrong](07-phase-space.md#channels-from-diagrams)
failure. The floor counts *accepted* points: a point the cuts reject
contributes nothing to a channel's term or its variance, so a floor in
drawn points would promise coverage it does not deliver, and each
channel's allocation is divided by its own measured acceptance, read from
iterations already finished so that the allocation cannot correlate with
the estimate it weights.

## Stopping

`integrate` runs until the combined estimate's relative uncertainty reaches
a target, 0.1% by default, unless a fixed budget is requested. A stop is a
claim about an error bar, and error bars here are known to be optimistic,
so three conditions guard it:

- The combination must be the [unweighted mean](08-vegas.md#combining-iterations).
  The inverse-variance rule's bias comes with a small error bar, which is
  precisely the shape that makes a convergence test stop early on a wrong
  number, and the target-budget mode refuses to read such an error.
- A minimum number of iterations must have run, so the \\(\chi^2\\) below
  has degrees of freedom and the grids have been refined more than once or
  twice.
- The error tested is the quoted error times
  \\(\sqrt{\max(1, \chi^2/\mathrm{dof})}\\) per channel, the Particle Data
  Group's scale-factor treatment of measurements that disagree by more than
  their quoted errors, applied where the disagreement lives. This is the
  condition that does the work. The per-point weights of the channels that
  carry the cross section of `p p > l+ l- j` have a Pareto tail with index
  near 2, the boundary at which the variance exists at all, so empirical
  variances converge slowly and biased low, the iterations scatter by more
  than their quoted errors, and \\(\chi^2/\mathrm{dof}\\) of 2 to 3 at
  production budgets is what the excess measures. The factor costs a
  two-to-threefold increase in points before a stop is granted, and buys an
  error bar the seed-to-seed spread supports.

The status pane's progress bar during integration is a projection of when
this test will pass, re-estimated every iteration from the current error
under the \\(1/\sqrt N\\) law, which is why it moves in both directions.

## Evidence, not a fixed-seed pull

A sampler is validated statistically. A single seed agreeing with the
reference is not evidence, because the inverse-variance combination can
turn a missed region into a confidently wrong result with a small error
bar. The cross-section gates therefore sweep several seeds and read the
spread and the \\(\chi^2\\) per degree of freedom, with the budget scanned
as a second axis; five mutually consistent seeds have been collectively 1%
low before, and a failure that migrates between seeds as the budget grows,
rather than shrinking, is a bug. The [validation](12-validation.md) chapter
has the full principle.

## Where it lives

[`vibegraph::phasespace::channel`](../api/vibegraph/phasespace/channel/index.html)
for the [`MultiChannel`](../api/vibegraph/phasespace/channel/struct.MultiChannel.html)
combiner and its \\(\alpha\\) adaptation,
[`vibegraph::budget`](../api/vibegraph/budget/index.html) for allocation and
stopping, and the integrands in [`hadronic`](../api/vibegraph/hadronic/index.html)
and [`proton`](../api/vibegraph/proton/index.html) that present a process
as a set of channel terms.
