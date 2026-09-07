# Validation

Everything in the preceding chapters is checked against MadGraph5\_aMC@NLO,
and the way it is checked is as much a part of the design as the
algorithms. Three ideas organise it.

## Every oracle has a blind spot

Each comparison is made at the finest level the quantity permits, and for
each the class of error it *cannot* see is recorded, so that some other
comparison covers it. The [colour](05-color.md#what-each-oracle-can-and-cannot-see)
chapter has the canonical example: a Gram matrix cannot see a consistent
index transpose, a squared amplitude cannot see a global phase, and a
per-flow complex value can see both. The same logic runs through the
pipeline:

| Quantity | Strictness | Blind to |
|---|---|---|
| Model and card inputs, the $\alpha_s$ iteration, helicity survivor sets, colour tags, the kT merge sequence, the LHEF writer | bit-exact equality | nothing they compute, everything they do not |
| Per-point $\lvert\mathcal{M}\rvert^2$ against MadGraph's Fortran | $\le 10^{-12}$ relative, sized for summation order | global phases, flow relabellings |
| Per-flow JAMPs, per-diagram amplitudes per helicity | one fitted unit phase per process | a swapped pair of flows with identical JAMPs |
| Cross sections | statistical, at the reference's own uncertainty | a missed region a fixed seed happens not to expose |
| Event samples | weighted Kolmogorov–Smirnov and $\chi^2$ tests per observable | tails, correlations between columns |

A convention claim, "this sign comes for free", is treated as a hypothesis
until a test exists that would fail if it were false. A passing gate that
cannot see the convention is not confirmation.

## Samplers gate statistically

A cross section carries Monte Carlo error, and the
[inverse-variance combination](08-vegas.md#combining-iterations) can turn a
missed region into a confidently wrong result. Sampler gates therefore
sweep several seeds, read the spread and $\chi^2$ per degree of freedom
rather than a headline pull, and scan the budget as a second axis. Event
samples are compared distribution by distribution against MadGraph's
banked samples with tests that take the overweight events' weights into
account, since treating them as unit weight misstates exactly the tail
that is hardest to sample.

## Layers, and what each may assume

Every test registers into exactly one of three layers, declared in
`validation/manifest.toml`, and a test that quietly skips when its data is
absent is not permitted:

| Layer | May assume | Runs as |
|---|---|---|
| hermetic | a bare clone: no submodule, no fetched data, no network | `cargo test` |
| banked | the MadGraph submodule, fetched PDF sets, and a pinned, checksummed bundle of frozen MadGraph runs; may not run MadGraph | `pixi run validate` |
| oracle | a full MadGraph and LHAPDF toolchain, regenerating every reference | `pixi run generate-references` |

The banked layer ends by rendering a report: one row per validated
process, one column per category, each cell carrying its metric and
whether it is gated, informational, blocked on a named feature, or an
admitted gap. The driver asserts that the cells measured are exactly the
cells the manifest declares, so coverage cannot shrink unnoticed. CI runs
the hermetic and banked layers on every change and publishes the report as
the job summary.

## Where it lives

The gates are the integration tests under `vibegraph-lib/tests/` and
`vibegraph-cli/tests/`, the manifest and reference data under
`validation/`, and the collator in the `validation-report` crate. The
two-sample tests are in [`vibegraph::stats`](../api/vibegraph/stats/index.html),
and the observables an event is compared on in
[`vibegraph::lhef::observables`](../api/vibegraph/lhef/observables/index.html).
The `extended-validation` skill under `.agents/skills/` maps a kind of
change to the gate it must clear.
