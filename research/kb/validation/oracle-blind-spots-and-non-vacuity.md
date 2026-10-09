---
type: Validation Methodology
title: Oracle blind spots, convention claims and non-vacuity
description: "Worked cases behind AGENTS.md's validation rules: each oracle's blind spot and what covers it, convention claims pinned by falsifiers, and checks shown able to fire."
status: draft
tags: [validation, methodology, negative-control, blind-spot, non-vacuity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n16-debrief, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/16-color-flow-design.md#L497-L554", title: "Note 16 §6, sprint debrief"}
  - {id: n27-b4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/27-v3-backlog-plan.md#L482-L715", title: "Note 27 B4, the IDWTUP blind spot"}
  - {id: n28-s1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L1299-L1363", title: "Note 28 S1, permutation-closure control"}
  - {id: n28-s23, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L1539-L1622", title: "Note 28 S2.3, the ordering test and its negative controls"}
  - {id: n28-cov, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L1851-L1873", title: "Note 28, coverage per process"}
  - {id: n28-spine, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L2746-L2756", title: "Note 28, what the spine-sign test cannot see"}
  - {id: n28-k5b5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L3207-L3236", title: "Note 28 K5b.5, samples cells and what they cannot see"}
  - {id: n28-z3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L4159-L4226", title: "Note 28 Z.3, the vacuity guard's instance"}
  - {id: n36-b0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/36-banked-open-ends-plan.md#L86-L143", title: "Note 36 B0, seed-sweep headroom census"}
  - {id: kt-gate, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/tests/validate_kt_cluster.rs#L500-L510", title: "validate_kt_cluster.rs, the dumps-present assertion"}
---
# Oracle blind spots, convention claims and non-vacuity

The binding rules are in `AGENTS.md`, "Physics Validation": every oracle has a
blind spot, convention claims are hypotheses, keep a known-wrong informational
comparison running, and a report is evidence only if every green cell is a
recorded measurement. This concept carries the cases those rules came from, the
catalogue of blind spots with what covers each, and the shapes of check that
cannot pass vacuously. For amplitude disagreements the method is
[bit-exact-amplitude-debugging](bit-exact-amplitude-debugging.md); for what a
report cell means, [validation-report](validation-report.md).

## Blind spots, and what covers each

| oracle | provably cannot see | covered by |
|---|---|---|
| total |M|² | a global phase | per-diagram and per-flow amplitude gates ([amplitude-oracle](amplitude-oracle.md)) |
| the colour CF matrix | a uniform index transpose (it is a Gram matrix) | per-flow complex JAMPs |
| `color_cf_oracle` (graphs compared as sets of colour structures) | which colour structure multiplies which Lorentz structure | per-flow amplitude gates on `gg_to_gg`, `uux_to_ggg`, `gg_to_ggg`; `g g > t t~ g` has none ([gg-ttxg-no-amplitude-gate](../backlog/validation/gg-ttxg-no-amplitude-gate.md)) |
| per-flow JAMPs on `gg_to_gg` | a swap within trace-reversal pairs (`J₁=J₆`, `J₂=J₄`, `J₃=J₅`) | `color_flow_tags_oracle` against `leshouche.inc` ([colour-oracles](colour-oracles.md)) |
| `spine_sign_from_flow`'s class test | a sign common to every diagram | the `ud_to_epemud_qcd0` row of `amplitude_oracle`; if it were demoted, the class test would pass with the spine sign globally wrong[^n28-spine] |
| the coupling oracle | a rounding both sides share | nothing; stated in its doc comment |
| KS and χ² `samples` columns | normalisation: σ, and how a reader takes σ from weights | the `integrals` cells, and an absolute-spectrum comparison |
| `samples` columns on outgoing legs | the beams | the incoming-leg column, added after a massive-beam σ defect cleared the KS floor |
| σ | a per-event scale right on average and wrong per event | per-event replays ([scale-replay-gate](scale-replay-gate.md), [kt-cluster-dump-oracle](kt-cluster-dump-oracle.md)) |
| per-event dumps of MadEvent | points MadEvent rejected; regions it never populates | σ in sliced regions on both sides ([mlm-dump-oracle](mlm-dump-oracle.md)) |
| a map's volume against RAMBO | a wrong rung ordering (both orderings integrate dΦ correctly) | a coverage test on a peaked integrand[^n28-s23] |
| the LHE byte round trip with a source-preserving writer | whether this crate's own layout is MadGraph's | a second pass with the source dropped, which must still reproduce at least one file |
| one seed's pull | a region the sampler missed (small integral *and* small error) | [seed sweeps and budget ladders](seed-sweeps-and-budget-ladders.md) |

Two cases show the cost of an unlisted blind spot:
- **`IDWTUP`.** The sample reader took σ as the mean of `XWGTUP`, true at
  `IDWTUP = −4`. Every banked file was `−4`, and every `samples` cell was KS or χ²,
  invariant under rescaling one sample's weights. The first absolute comparison
  (`dσ/dm_ll` in pb) read MadGraph a factor 2.0e5 low, uniformly: those files were
  `−3`, where the sum is σ. The reader now dispatches on the field and panics on a
  value it does not know[^n27-b4].
- **Two σ rows 5.5% wrong** gated their `samples` cells, correctly: the defect
  was a nearly uniform factor, which a shape statistic cannot see. The cells say
  so rather than leaving it to be noticed[^n28-k5b5].

Every validation layer was blind to the colour-conjugation bug the CF oracle
(23/23 green) could not see; it was found one layer down, in per-flow complex
JAMPs[^n16-debrief].

## Convention claims are hypotheses

- A design note's "the conjugation is automatic" survived five sessions' gates
  because none could falsify it; the first process mixing `f`- and `T`-structures
  did. Schedule the falsifying probe early and read "all gates green" as "not yet
  contradicted"[^n16-debrief].
- The fermion-line reversal sign was charged per internal fermion propagator. On
  every SM row that equals the right rule (the `CΓᵀC⁻¹` parity of the line's
  bilinears), because every SM fermion line reaches a gauge vertex. A toy row
  built entirely of Yukawa-type bilinears was the first to tell them apart: the
  case for toy models as validation instruments ([toy-ufo-models](toy-ufo-models.md)).
- The conventions pinned this way, each by a test that fails if it is false, are
  catalogued in
  [amplitudes/convention-sign-inventory](../amplitudes/convention-sign-inventory.md)
  and [convention-channel-coverage](convention-channel-coverage.md).
- Verify a candidate fix arithmetically on dumped values before writing it:
  reconstructing |M|² by hand from MadGraph's amplitudes under each colorize
  hypothesis turned a plausible sign fix into a derived one.

## Every check is shown able to fire

A gate's negative control is part of the gate: assert that a known-wrong input
fails it, and print the margin.

- **The swapped chain.** The rung-ordering coverage test builds the same channel
  with its rungs reversed and asserts that at least one criterion fails for it
  (`assert!(swapped_fails, "the ordering test cannot fire")`). A precondition
  check requires the two maps to differ in density at all, at the floor a real
  run uses: at floor zero, all four `g g > g g` channels collapse onto one map and
  the test would have no content[^n28-s23].
- **Permutation closure.** The check that `g g > g g`'s channel set is invariant
  under exchanging the outgoing momenta refuses to pass unless dropping some
  single channel breaks the invariance. Its first run caught the unregulated
  collapse above[^n28-s1].
- **Coverage.** The bounded-channel coverage check counts which accepted points
  only a bounded channel reaches, and pushing every bound out 100× must lose
  points where coverage is a real constraint; where every row keeps an unbounded
  channel, the table says the check is not a constraint[^n28-cov].
- **A collapse-to-constant assertion.** The per-event cluster-scale rows require
  that the prescription resolved, did not collapse to a constant, and was handed
  channel forests: a collapse to `m_Z` would leave σ and samples looking fine
  while measuring nothing about the clustering.
- **Physics-sized controls.** Dropping the `αs` reweighting must move the matched
  σ by more than 1%; deleting `<scales>` moves the Pythia acceptances by up to
  25σ; the unrotated mirror convention is rejected at pull −9.31; the PDF grid and
  the parameter card's `αs` are separated by more than half a printed `AQCDUP`
  digit, or 20000 events would agree with either source and pin neither; the
  bundle fetch refuses the previous cut by digest as well as accepting the current
  one.
- **Inventories asserted both ways.** `GRID_ALPHA_S_RUNS`, `SCALE_FALLBACK_ROWS`
  and declined-run lists fail when a run joins or leaves the class, and a
  known-defect allowlist entry is required present, so it cannot outlive its
  cause.

## A rule with no instance is untested

Refuse to pass when the instance set is empty, and draw the instance from state
the repository keeps. A guard asserted that some manifest row was
`bundled = false` so its absent-row rule had something to check; `bundled = false`
is transient, so the coverage vanished exactly when the manifest was tidiest. The
test now builds all three classifications from sets it makes itself and keeps the
manifest's set only where it is the right oracle (a row that silently acquires
`bundled = false` fails)[^n28-z3].

A missing input fails rather than skips. The banked layer takes no runtime skips:
its inputs are acquired by `pixi run validate`'s dependency tasks, and a gate that
still finds one missing fails naming it. The kT clustering gate once printed "no kT
clustering dumps" and passed, having compared nothing, on every fetching checkout;
it is now registered at the oracle layer (`#[ignore]`, `pixi run -e madgraph
validate-kt-cluster`) and asserts the dumps are present[^kt-gate]. What a green CI
run covers is [tooling/ci-coverage](../tooling/ci-coverage.md).

## Thresholds are measurements too

- A tolerance's calibration comment is a measurement of the streams it was taken
  on. When sampling streams moved, every σ calibration comment written before
  that change stopped reproducing while every one written after it did; such
  comments are re-recorded, never trusted across a stream change[^n36-b0].
- Standardised thresholds (pulls, p-value floors, χ²/dof bands) are false-positive
  rates. Forming an extremum-against-floor statistic over more seeds raises its
  flag rate, so those are reported against their own scale, not judged by
  headroom ([gate-thresholds](gate-thresholds.md)).

[^n16-debrief]: Note 16 §6, observations 1–3 and 6.
[^n27-b4]: Note 27 B4, "The defect: a sample's cross section was read without looking at `IDWTUP`".
[^n28-spine]: Note 28, "What the new test cannot see".
[^n28-k5b5]: Note 28 K5b.5.
[^n28-s23]: Note 28 S2.3, NEG-A to NEG-C and the list of what the test cannot detect.
[^n28-s1]: Note 28 S1, the channel-enumeration decision.
[^n28-cov]: Note 28, "Coverage, per process switched on".
[^n28-z3]: Note 28 Z.3.
[^kt-gate]: `vibegraph-lib/tests/validate_kt_cluster.rs`, `the_clustering_engine_reproduces_madgraphs_own`.
[^n36-b0]: Note 36 B0.
