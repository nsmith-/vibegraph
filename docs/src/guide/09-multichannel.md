# Multichannel integration

A single [channel](07-phase-space.md#channels-from-diagrams) flattens the
peaks of one diagram. A process has many diagrams with peaks in different
invariants, and no single map is good for all of them. The **multichannel**
method ([Kleiss & Pittau 1994](../bibliography.md#phase-space-and-integration),
[Maltoni & Stelzer 2003](../bibliography.md#the-pipeline-as-a-whole))
samples from a mixture of the channels' densities,

$$
g(p) = \sum_j \alpha_j\, g_j(p), \qquad \sum_j \alpha_j = 1,\; \alpha_j \ge 0,
$$

and weights each point by $f(p)/g(p)$. Wherever any one channel's
density is large, the mixture is large, so every peak is covered as long
as some channel has it. The price is that computing the weight at a point
requires every channel's density at that point, not only the one that
generated it, which is why a channel exposes its density at an arbitrary
momentum configuration and not only at the points it produced. That
density sum is a measurable fraction of the per-point cost on a process with
hundreds of channels, so it is formed only once the rest of the integrand is
known to be nonzero: the cuts, the matrix element and the parton luminosity are
evaluated first, and a point whose product of those is zero contributes zero
whatever density it would have carried, so its density row is never built. The
ordering is an arithmetic shortcut and nothing else. Integration rejects no
points at all, and no acceptance probability is formed during it; every point
that survives the cuts enters the estimator with weight $f/g$. The one place a
point is accepted or rejected is
[event generation](11-unweighting-events.md#acceptreject), and the ratio
$f/g$ that decides it needs the matrix element and the density both, so it is
formed after both are in hand.

## One grid per channel

The mixture is one integral over the unit hypercube with an extra
coordinate for the channel draw, and a VEGAS grid could adapt to it as a
whole. The same estimator also splits into one term per channel,

$$
\int d\Phi\, f = \sum_j \int d\Phi\, f\,\frac{\alpha_j g_j}{g}
              = \sum_j \mathbb{E}_{p \sim g_j}\Big[\frac{\alpha_j f(p)}{g(p)}\Big],
$$

each term sampled from its own channel alone and weighted by the same
combined $g$. The two arrangements compute the same integral and differ
in what an importance grid in front of them can learn: a grid per term
adapts to a density conditional on the channel, which one separable grid
over the mixture cannot express. vibegraph banks one VEGAS grid per
channel, and this **hard split** with deterministic per-channel point counts
is what makes a channel's frozen grid the unit of event generation later.

## The same split, arranged differently

MadEvent splits the integral the same way and runs it as a different kind of
program. Its single-diagram-enhanced multichannel gives every configuration its
own integration in its own directory, with its own grid and its own
$\sigma_j \pm \Delta\sigma_j$, run as an independent job: a survey pass
over the configurations, then a refine pass in which each of them is driven
toward its own accuracy, with configurations related by a symmetry folded onto
one representative and the per-configuration results summed at the end.
vibegraph computes the same decomposition inside one process: the channel
weights come from the Kleiss-Pittau adaptation, the points are re-allocated
across channels every iteration, and one stopping rule reads the combined
estimate.

Each arrangement buys something the other does not.

- **A farm of independent jobs** is the strongest form of parallelism available:
  no shared state, no coordination, and a channel that failed or came back too
  noisy can be re-run by itself without touching the rest. vibegraph has to get
  its parallelism from inside one run, over `(channel, chunk)` work units, and
  has no way to resume one channel alone.
- **A global allocation** can spend where the *total*'s variance is, which
  per-channel targets cannot see. Each independent job converges toward its own
  accuracy, and the channels of a hadronic process converge at wildly different
  rates. Measured here, under a live target on the combined error, deriving the
  split from the channels' measured variances rather than from their weights
  took 2.18 times fewer evaluations, and the mechanism was feeding the starved
  channels whose per-channel inconsistency was inflating the stopping test.
- **Coverage** is a property of the allocation rule, and a rule with a floor in
  accepted points is what keeps a thin channel sampling its own structure. Split
  the integral into independent jobs and the equivalent guarantee has to be
  restated as a per-job budget policy.
- **One stopping rule** can apply the $\chi^2$ scale factor below to the
  combination that is actually being reported, rather than to each job's own
  error bar. The cost is that the channels are coupled: an iteration that
  measured no variance at all has to be kept out of the consistency test by
  hand, which an independent job would never have had to think about.
- **Determinism** is easier here: the hard split with deterministic per-channel
  counts makes a run a pure function of its seed, byte-identical at any thread
  count. A farm's result is a sum over jobs, and reproducing it means
  reproducing the job decomposition as well.

The comparison that exists end to end is CPU spent for accuracy reached: at
MadGraph's own banked accuracy, `p p > l+ l- j` costs about the same as its
summed job CPU and `p p > l+ l-` costs a factor 4.2 to 4.5 less. That is a
measurement of two whole programs, not of the two allocation rules.

## Adapting the channel weights

The $\alpha_j$ are free, and the variance-minimising choice gives more
weight to channels whose region is where the integrand's variance is.
Kleiss and Pittau's adaptation reads each channel's share of the second
moment off a survey and moves the weights toward it; vibegraph runs it
before the grids are trained, and the weights then stay fixed while the
grids adapt, since re-adapting both at once would let each chase the other.

Training them alternately is an open direction rather than a rejected one: run
the Kleiss-Pittau update against the trained grids, freeze the weights again,
refine the grids under the new weights, and repeat, the way an
expectation-maximisation loop alternates between two sets of parameters that are
each easy to fit with the other held still. Each half-step reduces variance on
its own, so the question is not whether such a loop converges but whether its
fixed point is enough better to pay for the extra passes, and an answer would
have to be read against the estimator's measured seed spread rather than off a
single run: the $\alpha$ update is itself a survey estimate over the same
heavy-tailed weights that make the stopping rule below need a scale factor.

## Allocating points

With the integral split into per-channel terms, each iteration must decide
how many points every channel gets. For a fixed total, the
variance-minimising split is Neyman's, $N_j \propto s_j$, with
$s_j$ the standard deviation of channel $j$'s term. Spending
$N_j \propto \alpha_j$ instead is nearly the same thing, because the
adapted $\alpha_j$ already are most of a variance estimate; measured on
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
- A minimum number of iterations must have run, so the $\chi^2$ below
  has degrees of freedom and the grids have been refined more than once or
  twice.
- The error tested is the quoted error times
  $\sqrt{\max(1, \chi^2/\mathrm{dof})}$ per channel, the Particle Data
  Group's scale-factor treatment of measurements that disagree by more than
  their quoted errors, applied where the disagreement lives. This is the
  condition that does the work. The per-point weights of the channels that
  carry the cross section of `p p > l+ l- j` have a Pareto tail with index
  near 2, the boundary at which the variance exists at all, so empirical
  variances converge slowly and biased low, the iterations scatter by more
  than their quoted errors, and $\chi^2/\mathrm{dof}$ of 2 to 3 at
  production budgets is what the excess measures. The factor costs a
  two-to-threefold increase in points before a stop is granted, and buys an
  error bar the seed-to-seed spread supports.

The status pane's progress bar during integration is a projection of when
this test will pass, re-estimated every iteration from the current error
under the $1/\sqrt N$ law, which is why it moves in both directions.

## Evidence, not a fixed-seed pull

A sampler is validated statistically. A single seed agreeing with the
reference is not evidence, because the inverse-variance combination can
turn a missed region into a confidently wrong result with a small error
bar. The cross-section gates therefore sweep several seeds and read the
spread and the $\chi^2$ per degree of freedom, with the budget scanned
as a second axis; five mutually consistent seeds have been collectively 1%
low before, and a failure that migrates between seeds as the budget grows,
rather than shrinking, is a bug. The [validation](12-validation.md) chapter
has the full principle.

## Where it lives

[`vibegraph::phasespace::channel`](../api/vibegraph/phasespace/channel/index.html)
for the [`MultiChannel`](../api/vibegraph/phasespace/channel/struct.MultiChannel.html)
combiner and its $\alpha$ adaptation,
[`vibegraph::budget`](../api/vibegraph/budget/index.html) for allocation and
stopping, and the integrands in [`hadronic`](../api/vibegraph/hadronic/index.html)
and [`proton`](../api/vibegraph/proton/index.html) that present a process
as a set of channel terms.
