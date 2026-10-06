---
type: Backlog Item
title: nhel = 1 run cards are refused, leaving wpwm_to_wpwmz_cw's σ and events unreachable
description: The run-card parser rejects nhel = 1 (Monte Carlo over helicities), which MadGraph chose for wpwm_to_wpwmz_cw, so that row's integrals and samples cells are uncovered.
area: validation
state: open
priority: low
closes_when: Helicity Monte Carlo sampling (nhel = 1) is supported and wpwm_to_wpwmz_cw's integrals and samples cells are measured, or the user records that nhel = 1 stays out of scope and the item is dropped.
blocked_by: []
opened: 2026-09-07
tags: [run-card, helicity-sampling, coverage, non-sm-ufo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L464-L470", title: "TODO.md entry T032"}
---
For the 222-diagram `wpwm_to_wpwmz_cw`, MadGraph itself chose `nhel = 1`:
Monte Carlo sampling over helicities in place of the explicit sum. The row's
`.mg5` script does not ask for it.

`vibegraph-lib/src/runcard.rs` lists `nhel` with default 0 and refuses a
non-default value as `UnsupportedField`. The refusal is correct: the setting
changes both the estimator and the per-event weight. Its consequence is that the
reference's banked σ and its 10000 events are unreachable from the card that
produced them, so the row's `integrals` and `samples` cells are `uncovered`.

Even once reachable, both cells would be informational until the row's
amplitudes agree. Today they disagree at |M|² 2.79e1; see
[wpwmz-cw-ow-five-vector-residual](wpwmz-cw-ow-five-vector-residual.md).
