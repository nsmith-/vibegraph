# Unweighting and event files

The integration leaves behind a frozen grid per channel and a cross section.
Event generation turns them into a sample of unit-weight events, and writes
the sample in the format a parton shower reads.

## Accept/reject

A point drawn from channel $j$'s frozen grid carries a weight
$w_j(x)$, the integrand over the sampling density. Accepting it with
probability $\min(1, w_j(x)/w^{\max}_j)$ produces points distributed as
the integrand itself. The acceptance rate is the mean weight over the
maximum, so a sampling density that follows the integrand closely, which
is what the adapted grids provide, is what makes unweighting affordable.

Two decisions remain.

**Which channel to draw.** The accepted events must carry cross section in
proportion to each channel's $\sigma_j$. A trial in channel $j$
accumulates weight at a rate proportional to $q_j \sigma_j / w^{\max}_j$
when the channel is drawn with probability $q_j$, which is proportional
to $\sigma_j$ exactly when $q_j \propto w^{\max}_j$. Any other rule
needs a compensating per-event weight and stops being an unweighted sample.
The overall acceptance is then $\sigma / \sum_j w^{\max}_j$, the best
any selection rule can reach, which makes the largest channel's share of
$\sum_j w^{\max}_j$, not the channel count, what decides how much the
split buys.

MadEvent meets the same requirement with a different arrangement, and the two
are the same statement seen from opposite sides. Its channels are separate
integrations with separate grids and separate $\sigma_j$, so it unweights each
of them on its own and writes an intermediate per-channel event file, and the
delivered `unweighted_events.lhe` is assembled from those files afterwards. The
combined sample carries cross section in proportion to $\sigma_j$ because each
channel contributes its own events, which requires the per-channel event counts
to be in that proportion; what rule MadEvent uses to set them is not established
in this project's notes. vibegraph has no per-channel files to apportion, only
one pass whose single lever is which channel a trial is spent in, so the
proportionality has to be produced by the trial rule itself, which is
$q_j \propto w^{\max}_j$. The practical difference is in the failure mode: a
per-channel file can come up short without the other channels noticing, while
one pass either reaches the requested event count or does not.

**What the maximum is.** $w^{\max}_j$ is estimated from a frozen scan of
each channel's grid, and the weight distributions here have Pareto tails
with index near 2, so the scan's largest weight never converges. Taking the
extremum makes the acceptance collapse as the scan grows. MadGraph's
`unwgt.f` instead takes the lowest scanned weight that leaves less than a
small share of the scan's summed weight above it, and vibegraph adopts the
same truncated rule, with the same 1% default share; on the validated
processes it raised acceptance from 4–23% to 10–54% at matched budgets. This
decision agrees with MadGraph's by adoption rather than by imitation: the rule
is the one a cleaner design would reach for anyway, since it turns a quantity
that provably never converges into one the caller sets.

Points above the maximum are then expected, not an error: they are kept at
weight $w/w^{\max} > 1$, which keeps the estimator unbiased, and counted two
ways, as a fraction of events and as a share of the cross section, the share
being the number that detects the silent failure of a handful of events carrying
a large part of $\sigma$. Whether MadEvent keeps its own overweighted events
the same way or clips them is not established in this project's notes; what the
notes do establish is the maximum rule, not what happens above it.

## Filling in the record

The cross section is computed with helicities summed and colours
contracted, but an event record names one helicity combination and one
colour flow. Once a point is accepted, each is drawn categorically from an
accumulator the evaluation already produced: the per-combination
$|\mathcal{M}_h|^2$ for helicity and the per-flow $\sum_h |J_f|^2$
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
overweight event $\lfloor w \rfloor + \mathrm{Bernoulli}(w - \lfloor w \rfloor)$
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

> **MadGraph compatibility.** The layout above is a Python post-processor's
> format strings, and the two-dialect rule exists because the file MadGraph
> delivers is sometimes its Fortran writer's spelling and sometimes the
> post-processor's, with nothing in the run card saying which. The fields are
> also wider than the digits that reach them, so a file re-emitted in the
> Fortran dialect keeps seven significant digits on the scale and coupling
> columns where this writer's own layout carries nine. A cleaner design would
> choose one layout, at the precision the values deserve, and write every file
> in it. What reproducing both dialects buys is a byte-exact round trip of
> MadGraph's own event files, which turns the writer's validation into a
> comparison against the reference instead of an assertion about ourselves.
> `vibegraph-lib/src/lhef/mod.rs`, `vibegraph-lib/src/lhef/write.rs`.

The line is drawn at defects rather than at conventions. MadGraph's `unwgt.f`
divides by a $\pi$ truncated to eight digits when it fills `AQCDUP`, so its
coupling column carries a relative bias of $1.7\times10^{-8}$; this writer
emits $\alpha_s(\mu_R)$ untruncated and pins the size of the difference in a
test, so the departure is on the record and cannot drift
(`vibegraph-lib/src/lhef/build.rs`). The jet-count memo on the
[scale](10-hadronic.md#scales) path is the other place the same line is drawn.

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
