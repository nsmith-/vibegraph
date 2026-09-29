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

### Decay runs

A [decay](07-phase-space.md#decays-at-rest) writes its events in MadEvent's
decay-run convention. `<init>` names the decaying particle as beam 1, with
its mass as the beam energy, and leaves beam 2 empty (`IDBMUP = 6 0`,
`EBMUP = 173 0` for a top). `XSECUP`, and `XWGTUP` under `IDWTUP = -4`, carry
the partial width in GeV where a scattering run's carry picobarns. Each event
starts with the mother at rest, status $-1$, and every product points back at
it alone, mothers `1 0` rather than the range `1 1` (`unwgt.f` zeroes the
second pointer when there is one incoming particle). `SCALUP` is the larger
of the card's two factorisation scales and `AQCDUP` is $\alpha_s$ at the
renormalisation scale, the particle's mass by default — both fixed for every
event, as MadEvent fixes them. `PDFSUP` is MadEvent's `get_pdf_id(pdlabel)`:
`247000` for the card's default `nn23lo1` although nothing reads a parton
density, as on every fixed-energy run. One field differs from MadEvent's file:
MadEvent writes an intermediate resonance of a plain process inside its
Breit–Wigner window (the $W$ in `t > b e+ ve`) as a status-2 record, which
this writer does only for [decay chains](#decay-chains).
`vibegraph check-events` reads an empty second beam as a decay run and
expects one incoming leg per event there.

### Decay chains

A decay-chain card (`p p > t t~, t > b e+ ve, t~ > b~ mu- vm~`) lists its
resonances as status-2 records, following MadEvent's `addmothers`. The
records are the timelike lines of the event's configuration — the one its
colour flow is drawn in, among the configurations whose forced lines are
inside their windows — that MadEvent's `cut_bw` flags at the event's momenta:
every forced line, and a free line (the $W$ inside `t > b e+ ve`) whenever it
lands within `bwcutoff` widths of its pole and $\Gamma/M < 0.1$, so a free
line's record comes and goes from event to event. A line with a same-flavour
daughter loses to it as `cut_bw` decides. Each record carries its
daughters' summed momentum, its virtuality as the mass, `SPINUP = 9`, and the
colour its daughters leave open once the lines one carries as colour and
another as anticolour are contracted; a top-level record descends from the
initial state (`1 2`, or `1 0` on a decay), a nested one and each leg below a
record name their innermost record twice (`3 3`). Records sit between the
incoming and the outgoing legs, parents before children; MadEvent orders
siblings by its configuration tag, which changes between the configurations
of one subprocess, and pointers name positions, so the order carries no
physics and is not reproduced. On `e+ e- > z z, z > e+ e-`, whose pairings
are all kept, the two `Z` records carry the pairing of the drawn
configuration, one whose windows the event is inside.

An event whose flow its configuration reaches only below leading colour lists
no record, as MadEvent's do.

### Several process numbers

A card with several process numbers (`generate p p > e+ e- / z @1`, `add process
p p > mu+ mu- / a @2`) writes one `<init>` entry per number and each event's
`IDPRUP` is its own line's. A line without `@N` takes MadGraph's number, its
position among the process lines. The integration sums every line in one set of
channels, so the per-process `XSECUP` is the file's own split: the process's
share of the written weight times the file's cross section, with `XERRUP` the
integration's relative error on it plus the share's sampling error, and
`XMAXUP` the process's own largest weight. A streaming
(`stochastic-rounding`) run knows the split before its first event only by
drawing the sample once and replaying it, which it does when there is more
than one process. A card summing several final-state multiplicities (`@0`, `@1`,
`@2` of a matched sample) is different: there each multiplicity is integrated as
its own part and the default strategy normalises every part to its own cross
section, which unit weights cannot do, so `stochastic-rounding` refuses such a
card.

## Reweighting

`generate --reweight-card reweight_card.dat` gives every event its weight under
alternative model parameters too, MadGraph's reweighting workflow. The card is
MadGraph's: each `launch` block is one hypothesis, and its `set` lines move
external parameters by LHA address or by name.

```text
launch --rwgt_name=ymt_150
  set yukawa 6 150.0
launch --rwgt_name=ymt_200
  set ymt 200.0
```

An event drawn under the model's own parameters is carried to a hypothesis by
the ratio `|M|²_new / |M|²_old` of its own concrete subprocess at its own momenta
and strong coupling; the phase-space density, the parton densities and the
scales cancel. The file declares the hypotheses in an `<initrwgt>` block in its
`<header>` and writes each event's weights in a `<rwgt>` block, in `XWGTUP`'s own
units, so the mean of a hypothesis's weights over a buffered file is its cross
section exactly as the mean of `XWGTUP` is the nominal one. A reweighted sample
is only as good as the nominal sample's coverage of the phase space the
hypothesis populates; the ratios say nothing about regions the nominal sample
never visits.

A hadronic flavour group shares one matrix element at the card's parameters,
which is what grouped it, but not necessarily at a hypothesis's — a parameter can
move one quark flavour's coupling and not another's — so each event is reweighted
with its own member's amplitude.

### The polynomial path

Reweighting costs amplitude evaluations, and an effective-field-theory study asks
for many hypotheses: with `n` new couplings entering at most once per diagram,
`|M|²` is a quadratic form in them, and the usual card is a basis grid of at
least `n(n+1)/2` points that pins it down. Evaluating each point directly costs
one amplitude per point per event.

The polynomial path instead collects the amplitude by coupling monomial. When
parameters `P` enter a subprocess only through its couplings, each a polynomial
in `P`, and no mass or width moves with them, the amplitude of every helicity
combination and colour flow is `A(P) = Σ_μ μ(P)·a_μ`, a sum over monomials `μ`
with one coupling class `a_μ` each. The monomials are read off the UFO coupling
expressions and the diagrams' vertices symbolically (`reweight::poly`), not
fitted: a diagram carries the product of its vertices' monomials. Then

```text
|M(P)|² = Σ_μν μ(P) ν(P) · Re Σ_hel Σ_fg conj(a_μ,f) CF_fg a_ν,g
```

is a quadratic form in the monomial vector with a per-event matrix, and every
hypothesis is one `K × K` quadratic form in it. With one insertion per diagram
`K = 1 + n`, so an event costs `n` amplitude evaluations beyond its own however
dense the grid is.

The class amplitudes come from `K` evaluations at parameter nodes rather than
from splitting diagrams, because a coupling can shift an existing vertex
(`a + b·c`): that one diagram belongs to two classes. The nodes are chosen for
conditioning — the card's own point first, the rest taken by a column-pivoted QR
from a Chebyshev grid over the box the hypotheses span — and each hypothesis's
weights over them are solved once, before the first event. The matrix is formed
in the node basis directly, so the class amplitudes are never materialised.

`--reweight-couplings cW,cHW,cHWB` asks for this explicitly: every hypothesis is
served by one polynomial per subprocess jointly in the named parameters, a card
that moves anything else is refused, and so is a named parameter that some
subprocess does not carry polynomially. Without the flag, hypotheses moving one
and the same parameter are grouped by it where that is cheaper than evaluating
them, and everything else — a hypothesis moving several parameters, a parameter
that is not a polynomial coupling (`aEWM1`, which reaches the couplings through
square roots; `MZ`, which is also a propagator pole) — is evaluated directly:
its amplitude is bound to its parameters before the first event and evaluated
once per event. `--reweight-exact` forces that path for every hypothesis; the
paths agree to within rounding, and the unit tests hold them to it.

The generation's own amplitudes drop what vanishes at the card's parameters,
and a hypothesis that switches on a coupling the card leaves at zero revives
exactly that. Reweighting therefore compiles its own amplitudes, pruned at a
generic point where every parameter any hypothesis moves is set to an
unremarkable value.

### What is refused

Each of these is an error naming the card line, never a skipped line: `change`
directives (another model, process or mode), a param-card path in place of `set`
lines, `scan:` values, an internal or unknown parameter, a parameter the model's
restriction fixed to zero (its vertices were removed with it), anything that
moves the strong coupling (each event takes it from the scale choice, so that is
a scale variation), the mass of an external particle (the momenta sit on the old
mass shell), and a process with a forbidden s-channel (`$`), whose amplitude
depends on each event's position relative to the veto windows.

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
[`emit`](../api/vibegraph/lhef/emit/index.html) layers;
[`vibegraph::reweight`](../api/vibegraph/reweight/index.html) for reweighting.
