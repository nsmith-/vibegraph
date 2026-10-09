---
type: Validation Gate
title: "The samples gate: event samples against MadGraph's"
description: "Weighted KS and chi2 per observable against MadGraph's banked events at P_FLOOR 1e-4 over three seeds, plus beam, scale, polarization and spectrum columns; and what it cannot see."
status: draft
tags: [validation, samples, ks-test, lhef, madgraph]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n25-samples, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/25-validation-layering-plan.md#L151-L162", title: "Note 25 §3.4 (samples category)"}
  - {id: n25-machinery, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/25-validation-layering-plan.md#L377-L395", title: "Note 25 §5.5 (samples machinery)"}
  - {id: n25-closeout, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/25-validation-layering-plan.md#L585-L621", title: "Note 25 §10 (what each session landed)"}
  - {id: n27-b4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/27-v3-backlog-plan.md#L482-L715", title: "Note 27 B4 (Drell-Yan event banks, IDWTUP, the m_ll spectrum)"}
  - {id: n28-k5b5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L3207-L3236", title: "Note 28 K5b.5 (samples cells and what they cannot see)"}
  - {id: n28-c5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L3701-L3775", title: "Note 28 C.5–C.6 (pp_to_jj samples cell and instruments)"}
  - {id: n28-c26, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L4050-L4075", title: "Note 28 C2.6 (instruments after the enumeration repair)"}
  - {id: n29-e4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L1722-L1806", title: "Note 29 E.4 (mode decided by a rule stated before measuring)"}
  - {id: n29-addenda, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L3771-L3844", title: "Note 29 chain D addenda (A6: ee_to_mumua headroom)"}
  - {id: n29-rulings, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L6126-L6136", title: "Note 29 close-out rulings (p-floor not raised)"}
  - {id: n36-b2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/36-banked-open-ends-plan.md#L144-L191", title: "Note 36 B2 (incoming legs in samples)"}
  - {id: n36-b6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/36-banked-open-ends-plan.md#L458-L524", title: "Note 36 B6 (SCALUP/AQCDUP columns)"}
  - {id: n38-z2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/38-process-grammar-sprint-plan.md#L1528-L1686", title: "Note 38 §8.5 (polarized-leg count column)"}
  - {id: vs-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/tests/validate_samples.rs#L1-L200", title: "validate_samples.rs module docs, P_FLOOR, Row"}
  - {id: samples-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/validation/samples.rs", title: "vibegraph::validation::samples"}
  - {id: manifest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/validation/manifest.toml", title: "validation/manifest.toml samples cells"}
---

The `samples` category[^n25-samples] compares the events a generator actually emits against
MadGraph's banked event samples, distribution by distribution. Every other
category compares a number (a diagram count, an amplitude, a σ); this one is
the only place several things become visible at all: a mis-sampled region of
small measure, which σ averages over, and the record fields `SPINUP`,
`ICOLUP`, the incoming legs and the scales, which move no weight and so leave σ
and every shape untouched when wrong.[^vs-rs]

## Where it runs

| file | rows | path |
|---|---|---|
| `vibegraph-lib/tests/validate_samples.rs` | the fixed-beam rows (`ebeam1 = ebeam2`, so lab frame = partonic CM, asserted per row) | library: `FixedBeamIntegrand`, `SubprocessRecord`, the production record assembly |
| `vibegraph-cli/tests/validate_samples_proton.rs` | the proton rows (`pp_to_jj`, `pp_to_ll` on both dy13 cards, the `llj`, `bb` and `scalefact2` rows) | the shipped binary: one `integrate`, then `generate` per seed |

Both are banked-layer gates (see [layers](validation-layers.md)); the cells
render in the [validation report](validation-report.md).

## Protocol

- **Samples.** `GEN_SEEDS` = three generation seeds, `EVENTS_PER_SEED =
  20_000` each, against MadGraph's banked sample (10 000 events for most rows;
  200 000 for the dy13 Drell-Yan banks). A seed that hits
  `MAX_TRIALS_PER_EVENT = 400` reports "produced N of 20000"; the remedy is
  more integration budget, never fewer events, seeds or
  trials.[^n29-e4][^vs-rs]
- **Weights.** Our sample is nearly unweighted (Buffer strategy keeps
  `w/w_max > 1` points as overweights), so KS uses the weighted empirical
  CDF; MadGraph's `XWGTUP` is carried the same way.[^n25-machinery]
- **Continuous columns**: two-sample KS on pair invariant masses,
  Collins–Soper `cos θ*` for Drell-Yan-like rows, per-particle `pT`, `y`, `φ`,
  per process class.[^n25-machinery]
- **Discrete columns**: χ² homogeneity on `SPINUP`, `ICOLUP` flow labels (via
  `leshouche.inc`) and the flavour assignment. Categories below a share
  threshold are pooled.[^n25-closeout]
- **Incoming legs** (`samples::beam_columns`): per beam `E`, `pz`, `m`. Where
  the field is constant (fixed beams) it is an equality at half the last
  printed digit, read off the file's own spelling; where it varies (proton
  beams, `x₁`, `x₂`) it is a weighted KS on `E` plus the mass equality. This is
  the only comparison of the realised `x` spectra.[^n36-b2]
- **Reported scales** (`FieldColumn`): `SCALUP` and `AQCDUP`, filled through
  `FixedBeamIntegrand::record_scales`, the shipped generator's own call, so the
  column measures what the binary writes. Exact where constant, KS where they
  vary. Both sides are rounded onto the reference's grid of **seven
  significant digits** (what `rw_events.f` writes on the `<event>` line;
  MadGraph's Python re-serialisation prints two more digits of padding) before
  the KS. Without that rounding an exact-value KS reads the large share of
  MadGraph events piled on one printed value (all 10 000 of `uux_to_uux`'s) as
  the gap.[^n36-b6][^manifest]
- **Polarized legs** (`POLARIZED_LEGS`): a count of events whose polarized
  leg's `SPINUP` lies outside its restriction, on both samples. The `SPINUP`
  χ² cannot see a rare wrong helicity because it pools small categories; the
  count can, and `the_polarized_legs_carry_only_their_restricted_helicities`
  shows it fires on unpolarized runs of the same legs.[^n38-z2]
- **Absolute spectra** (`samples::Spectrum`): `dσ/dm_ll` in picobarns, binned
  down to threshold on `pp_to_ll` (both dy13 cards,
  `the_drell_yan_mass_spectrum_is_binned_against_madgraph`) and on
  `ee_to_mumu_tata_qcd0` (`the_low_m_ll_region_is_binned_against_madgraph`).
  A bin is judged if it carries at least `1e-4` of MadGraph's sample, against a
  4σ combined-error threshold (`SPECTRUM_MAX_PULL`) set from the ~50-bin trial
  count. On `pp_to_ll` the lower edge `m_ll ≥ 2 ptl = 20 GeV` is exact at this
  order (the pair recoils against nothing) and the gate requires zero weight
  below it on both sides.[^n27-b4]

**A sample's σ follows its `IDWTUP`.** `EventSample::from_lhe` takes the
mean of `XWGTUP` under `-4`, `XSECUP` under `±3`, and panics on any other
value rather than guessing. Which value MadGraph writes is a property of the
run card, not the version: `event_norm` defaults to `average` in MadGraph's own
full cards but to `sum` (giving `-3`) when a hand-written card omits it
(`madgraph/various/banner.py:4298`, `sys_default='sum'`). The shape statistics
are invariant under rescaling one sample's weights, so only the absolute
spectrum could catch a wrong reading, and it did, at a constant factor of
`2.0e5` (the event count).[^n27-b4]

## The p-floor

`P_FLOOR = 1e-4` in both files. It is chosen from the trial count, not from
taste: a run takes the smallest p over every observable of every gating row on
every seed, a few hundred effectively independent draws from the null (a 2→2
row's observables are heavily correlated). At `1e-3` that would be about half a
spurious failure per run; at `1e-4` about 0.1.[^vs-rs]

- **Never loosened after a failure.** A column that falls below the floor is
  recorded, the row is marked `info` with the measurement in its manifest note,
  and the disagreement is filed. The floor does not move, and raising it to
  protect a row near it would be the same loosening.[^vs-rs][^n29-rulings]
- **More seeds is not a remedy.** The statistic is the smallest of some hundred
  draws from a uniform distribution, so more seeds *lower* the expected minimum
  and raise the false-flag rate. Unlike a σ row, whose statistic is a mean, a
  `samples` row cannot buy headroom with seeds.[^vs-rs]
- **The headroom reading is not pinned.** A p-value near the floor moves with
  evaluator re-association that is checked but not bit-identical
  (`ee_to_mumua` read `1.29e-4`, then `3.605e-4`), so the `P_FLOOR` doc comment
  is re-recorded whenever checked rather than trusted. Its current account
  names `ee_to_wpwm` `pt(w+)` at `1.573e-4` (1.6× the floor) as the row to
  watch, unchanged over five seeds (`probe_samples_p_floor_headroom`). See
  [seed headroom census](seed-headroom-census-2026-09.md) and
  [gate thresholds](gate-thresholds.md).[^vs-rs]

## Gate or info: decided before measuring

A row's mode is fixed by a rule stated before its first measurement: if every
column of every seed clears `P_FLOOR` the cell is `gate`, otherwise `info`,
with the failing column, its p-value and its seed in the manifest note and the
disagreement reported rather than tuned.[^n29-e4] The manifest's cell `mode`
and the test's `Row.mode` must agree; the collator fails on a measurement that
disagrees with its declaration. Per-row modes live in
`validation/manifest.toml`; read them there. Rows reported rather than gated,
and why, are collected in
[sigma-row gating exceptions](sigma-row-gating-exceptions.md). The one
standing `samples` cell at `info` is `ee_to_mumua`, whose `pt(a)` column
measures a defect of MadGraph's own banked sample
([backlog](../backlog/validation/ee-mumua-radiative-return-sigma-high.md)).[^manifest]

One field is reported by construction rather than by mode: `AQCDUP` on the six
toy rows whose model declares no `aS` (`UNDECLARED_ALPHA_S_RUNS` in
`vibegraph-lib/tests/common/mod.rs`). MadGraph injects `aS = 0.138` beside
`G = 4.1643` and runs from `G`; we build no coupling and report none. `SCALUP`
still gates on those rows. See [toy UFO models](toy-ufo-models.md).[^vs-rs]

## Never by bytes on a multi-group run

MadGraph regenerates a single-group run's events bit-identically, but a run
with several subprocess directories (`pp_to_jj` has five) has a
scheduling-sensitive unweighting draw, so a re-run of the same card is a
different and equally valid sample. Only its distributions are statements about
it.[^n28-c5]

On `pp_to_jj` two colour checks sit beside the `ICOLUP` χ², each blind to the
other's failure: every leg's occupied `ICOLUP` slots must be legal for its
colour representation (a Les Houches convention check with no reference in it),
and every event's (roles, connectivity) pattern must be one the run's own
`leshouche.inc` admits, at zero tolerance. Both read 0 on MadGraph's own
events first. Legality is not correctness: a legal record with the wrong one of
two flows passes both, which is why the χ² stays.[^n28-c26][^manifest] The
per-member colour-flow tables behind `ICOLUP` are
[per-member colour-flow tables](../events/per-member-colour-flow-tables.md);
the selection rules are
[colour and helicity selection](../events/colour-and-helicity-selection.md).

## What it provably cannot see

| blind to | why | what covers it |
|---|---|---|
| normalisation | every KS and χ² is on normalised distributions | the [σ gate](sigma-gate.md); the absolute spectra |
| correlations between columns | each observable is compared as a marginal | nothing generic |
| a discrepancy confined to a small tail | KS is a maximum CDF gap, least sensitive at the edges | binned spectra (`m_ll`) |
| leg ordering within an assignment | `observables::canonical` sorts outgoing legs by class label then `pT`, and `flavour_key` is taken on that | the subprocess-set comparison against `leshouche.inc` |
| a scale that collapsed to a constant near the right value | σ and shapes stay close while measuring nothing about clustering | `the_llj_parton_rows_take_a_per_event_cluster_scale` asserts the prescription resolved, did not collapse, and was handed channel forests |
| a beam built differently but landing on MadGraph's numbers | the beam columns are an equality against the record | nothing generic |

Two worked cases of the normalisation blind spot. When two `l+ l- j` partonic
rows carried a nearly uniform 5.5% σ deficit, their `samples` cells passed and
said so on the cells: a passing shape cell is not evidence about σ.[^n28-k5b5]
Before the beam columns existed, three massive-incoming toy rows with a 6–7% σ
error cleared the KS floor comfortably for the same reason; when the columns
landed, seven rows (not the three expected) were found writing off-shell beams
(model mass beside light-cone momenta), and every massless row read deviation
exactly 0.[^n36-b2]

A `samples` cell also cannot *attribute*. It says two samples differ; it
cannot separate a sufficient cause from the only cause. Attribution needs the
cell re-measured after the suspected fix, with the columns returning inside
the floor.[^n28-c5]

## Related

[Event-output gates](event-output-gates.md) cover the LHEF round trip,
unweighting and `generate`. `vibegraph check-events` is a different check: it
reads one `.lhe` file and tests its internal consistency (well-formedness,
momentum conservation, mass shells), with no reference and no physics in it
(`vibegraph-cli/src/check.rs`). The event pipeline is under
[generate](../events/generate.md).

[^n25-samples]: Note 25 §3.4.
[^n25-machinery]: Note 25 §5.5.
[^n25-closeout]: Note 25 §10.
[^n27-b4]: Note 27 B4 outcome.
[^n28-k5b5]: Note 28 K5b.5.
[^n28-c5]: Note 28 C.5–C.6.
[^n28-c26]: Note 28 C2.6.
[^n29-e4]: Note 29 E.4(a).
[^n29-rulings]: Note 29 close-out, manager rulings; and addendum A6.
[^n36-b2]: Note 36 B2.
[^n36-b6]: Note 36 B6 item 1.
[^n38-z2]: Note 38 §8.5.
[^vs-rs]: `vibegraph-lib/tests/validate_samples.rs`, module docs and `P_FLOOR`.
[^manifest]: `validation/manifest.toml`, the `pp_to_jj` and `ee_to_mumua` `samples` notes.
