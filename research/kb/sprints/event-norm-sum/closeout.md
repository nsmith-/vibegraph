---
type: Sprint Record
title: "event_norm = sum and unity in the written events"
description: "generate honours event_norm: sum writes IDWTUP -3 with weights summing to σ, unity IDWTUP -3 with unit weights, average is unchanged; pinned per event against two banked MadEvent samples, and MadGraph's own unity path crashes."
closed: 2026-10-21
status: draft
tags: [lhef, event-norm, unweighting, cms, sprint-d, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: v1, resource: "sessions/V1-report.md", title: "V1 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/66", title: "nsmith-/vibegraph#66"}
---
One PR ([nsmith-/vibegraph#66](https://github.com/nsmith-/vibegraph/pull/66),
merged 2026-10-21) closing `event-norm-sum-refused`, which blocked 20 of the
survey's 65 LO CMS cards. Sessions: [F1](sessions/F1-report.md) (feature-dev,
Sonnet), [HY1](sessions/HY1-report.md) (hygiene-dev, Sonnet) and
[V1](sessions/V1-report.md) (validation-dev, Sonnet). The M3 banked the
per-event samples.

## What was established

- **The rule, from MadGraph's source at `b7687064`.** `MultiEventFile.unweight`
  (`lhe_parser.py:1225-1239`): `sum` writes strategy 3 with weight
  `Σ/event_target`, `average` strategy 4 with `Σ`; `define_init_banner` writes
  `IDWTUP = −|strategy|` and `XMAXUP` = that weight. The LO card allows `sum`,
  `average`, `unity` (`banner.py:4298`). *Manager-verified:* both citations read.
- **What generate writes.** `average`: `IDWTUP −4`, weights with mean σ, as
  before. `sum`: `IDWTUP −3`, the same weights over the event count, summing to σ.
  `unity`: `IDWTUP −3`, every `XWGTUP` 1. `XSECUP`/`XERRUP` are σ in all three;
  the integration is untouched. Stochastic rounding refuses `sum` and `unity`.
  At merge, the run card stayed in the artifact's fingerprint, so a different
  `event_norm` at generate was refused (F1, pinned by a test).
- **Where this writer differs from MadGraph.** MadGraph flattens the overweight
  tail to one constant weight; this writer keeps it, so only the sum or mean of
  the weights is comparable, not one event's.
- **Banked evidence.** Five MadEvent `<init>` blocks already banked under `sum`
  (`decay_chain_events_reference.json`: `IDWTUP −3`,
  `XMAXUP = ΣXSECUP/10000`), and two samples the M3 banked at `9743133`
  (`event_norm_reference.json`, `e+ e- > mu+ mu-` on `ee500`, 1000 events each
  under `sum` and `average`). `event_norm_weights_match_madevents_per_event`
  reads them: with no overweight tail every header value and all 1000 weights
  equal MadEvent's; with a tail, the sum (or mean) is σ and the tail/bulk ratio
  is the generator's. Swapping the scalings, flattening the tail, or moving
  weight between events each fail it (V1's mutations). *Manager-verified:* the
  per-event mutations fail the test.
- **MadGraph's `unity` path crashes.** The card admits `'unity'` but the code
  compares `'unit'` (`lhe_parser.py:1232`); the M3 run ended in
  `UnboundLocalError` on `new_wgt` at `:1238` with no event file. By the user's
  decision `unity` is kept, implemented as the intended unit-weight branch, and
  the crash is a row in `validation/madgraph-defects.md` (not reported upstream).
- **HY1's refactor was identity-checked.** One `norm_rule` now carries IDWTUP,
  the per-part scale and the flatten flag, and `StochasticRounding::honours` is
  the one refusal predicate. The emitted files for three norms × two plans were
  byte-identical before and after (sha256 `cbcaa699…`). The new multi-part test
  catches a per-part denominator mutation the old tests missed.

The manager re-ran the library tests, `lhef::emit` (17), `cli_generate` (13) and
`cli_reference_docs` from the committed tree after F1 and HY1.

## Backlog

- Deleted: `event-norm-sum-refused`.
- No Found entry was filed:
  - F1 Found 1 (the `unity` crash): observed by the M3 and recorded in the
    defects catalogue. Its upstream report is proposed as an addition to
    `madgraph-defect-reports-unfiled` rather than a new item;
  - F1 Found 2 (`include=False` on `<MGRunCard>`): not reproduced. `include`
    governs the Fortran `run_card.inc`, and MadGraph's own committed cards write
    the field (HY1 H7, confirmed by the manager);
  - V1's Found are a closed item and a planning note (the workspace suite takes
    about 20 minutes on the shared host).

## Process notes

Two user decisions shaped the PR: keep `unity` although MadGraph's own path
cannot run, and close the item only after the per-event bank. A struck Found
entry (F1 Found 2) shows the value of the hygiene pass reading the premise, not
only the code.
