# Unweighting and event files

The integration leaves behind a frozen grid per channel and a cross section.
Event generation turns them into a sample of unit-weight events, and writes
the sample in the format a parton shower reads.

## Accept/reject

A point drawn from channel \\(j\\)'s frozen grid carries a weight
\\(w_j(x)\\), the integrand over the sampling density. Accepting it with
probability \\(\min(1, w_j(x)/w^{\max}_j)\\) produces points distributed as
the integrand itself. The acceptance rate is the mean weight over the
maximum, so a sampling density that follows the integrand closely, which
is what the adapted grids provide, is what makes unweighting affordable.

Two decisions remain.

**Which channel to draw.** The accepted events must carry cross section in
proportion to each channel's \\(\sigma_j\\). A trial in channel \\(j\\)
accumulates weight at a rate proportional to \\(q_j \sigma_j / w^{\max}_j\\)
when the channel is drawn with probability \\(q_j\\), which is proportional
to \\(\sigma_j\\) exactly when \\(q_j \propto w^{\max}_j\\). Any other rule
needs a compensating per-event weight and stops being an unweighted sample.
The overall acceptance is then \\(\sigma / \sum_j w^{\max}_j\\), the best
any selection rule can reach, which makes the largest channel's share of
\\(\sum_j w^{\max}_j\\), not the channel count, what decides how much the
split buys.

**What the maximum is.** \\(w^{\max}_j\\) is estimated from a frozen scan of
each channel's grid, and the weight distributions here have Pareto tails
with index near 2, so the scan's largest weight never converges. Taking the
extremum makes the acceptance collapse as the scan grows. MadGraph's
`unwgt.f` instead takes the lowest scanned weight that leaves less than a
small share of the scan's summed weight above it, and vibegraph adopts the
same truncated rule; on the validated processes it raised acceptance from
4–23% to 10–54% at matched budgets. Points above the maximum are then
expected, not an error: they are kept at weight \\(w/w^{\max} > 1\\), which
keeps the estimator unbiased, and counted two ways, as a fraction of events
and as a share of the cross section, the share being the number that
detects the silent failure of a handful of events carrying a large part of
\\(\sigma\\).

## Filling in the record

The cross section is computed with helicities summed and colours
contracted, but an event record names one helicity combination and one
colour flow. Once a point is accepted, each is drawn categorically from an
accumulator the evaluation already produced: the per-combination
\\(|\mathcal{M}_h|^2\\) for helicity and the per-flow \\(\sum_h |J_f|^2\\)
for [colour](05-color.md#colour-flows-for-events), MadGraph's `SELECT_HEL`
and `SELECT_COLOR`. A third draw picks the integration configuration, the
diagram whose channel is taken to have produced the event, which the
[kT-clustering scale](10-hadronic.md#scales) needs. These selections enter
no integrand and move no cross section; they exist only to fill in
columns.

## The Les Houches Event File

The output is a Les Houches Event File
([Alwall et al. 2007](../bibliography.md#event-files)), an XML-like text
format carrying the run-level `<init>` block (beams, PDF identifiers, the
cross section, its error, the maximum weight, and `IDWTUP`, which says how
event weights are to be read) and one `<event>` block per event: the
particle count, the subprocess id, the weight, the scale `SCALUP`, the
couplings `AQEDUP` and `AQCDUP`, then a line per particle with its PDG
code, status, mother pointers, colour and anticolour tags, four-momentum,
mass, lifetime and helicity. The record layout follows the earlier Les
Houches Accord ([Boos et al. 2001](../bibliography.md#event-files)).

The overweight tail has to be represented somehow, and the file offers two
honest ways. The default keeps the weights, declaring `IDWTUP = -4`, so the
weight column is a cross section in picobarns and the total is the mean of
the event weights; the tail stays visible event by event.
`--strategy stochastic-rounding` declares `IDWTUP = +3` and writes an
overweight event \\(\lfloor w \rfloor + \mathrm{Bernoulli}(w - \lfloor w \rfloor)\\)
times, representing the tail as multiplicity. Since `<init>` precedes the
events and carries totals, a strategy whose header depends on the realised
sample cannot stream, and the interface is shaped around that.

The accord fixes the fields but not their formatting, and showers have
historically parsed columns. The layout written here is byte for byte the
one MadGraph's delivered `unweighted_events.lhe` carries, which is the
Python post-processor's layout rather than the Fortran writer's; the two
differ in exponent width and significant digits, and a file in either
dialect is re-emitted in its own, since the parser keeps each line it
decoded and hands it back verbatim when the values are unchanged. The
writer is validated by round-tripping MadGraph's own event files unchanged,
and every emitted file is read by Pythia 8 in the banked validation layer.

## Seeds and reproducibility

A `generate` run is a pure function of the artifact, the cards and a seed.
The same seed reproduces the same file; distinct seeds are independent
streams of the counter-based generator described under
[random numbers](07-phase-space.md#random-numbers), so files from
different seeds concatenate into one sample. Reproducibility holds under a
pinned seed and an unchanged sampling order, and nothing else is promised.

## Where it lives

[`vibegraph::unweight`](../api/vibegraph/unweight/index.html) with
[`MaxRule`](../api/vibegraph/unweight/enum.MaxRule.html),
[`vibegraph::select`](../api/vibegraph/select/index.html) for the categorical
draws, and [`vibegraph::lhef`](../api/vibegraph/lhef/index.html) with its
[`record`](../api/vibegraph/lhef/record/index.html),
[`write`](../api/vibegraph/lhef/write/index.html),
[`parse`](../api/vibegraph/lhef/parse/index.html),
[`build`](../api/vibegraph/lhef/build/index.html) and
[`emit`](../api/vibegraph/lhef/emit/index.html) layers.
