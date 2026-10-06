---
type: Backlog Item
title: pp_to_jj's samples flavour χ² holds back MadEvent's τ rule as the default
description: MadEvent's τ map saves ~23% of dijet evaluations, but as the default it takes pp_to_jj's flavour χ² to p 2.8e-5 on one seed against a 1e-4 floor.
area: validation
state: needs-user
priority: medium
closes_when: The pp_to_jj samples flavour statistic is matched to its calibration (larger generated sample or pooled-seed statistic, never a lower floor), and the τ rule can become the default with the cell passing.
blocked_by: []
opened: 2026-09-23
tags: [phase-space-maps, tau-map, samples-gate, calibration, pp-to-jj]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L351-L360", title: "TODO.md entry T023"}
---
MadEvent's `τ` rule (`--map-tau inverse-square`) takes 0.77 ± 0.03 of the
evaluations on dijets. `MapOptions::resolve` still defaults to `log`. As the
default, it fails `pp_to_jj`'s samples cell, the flavour composition
χ²-compared against MadGraph's banked 10 000 events per seed (floor 1e-4): one
seed reads p = 2.8e-5 (128.1 / 70 dof), and the three-seed sum is 325 / 212,
against 267 / 210 under the log map. Both read above expectation; the
reference's own flavour χ² already sits high.

The map does not move the composition: our own 200 000-event samples under
`log` and `inverse-square` agree at χ² 40.5 / 47 (same-map seed controls
47.6 / 47 and 47.7 / 46). Effective sample sizes are ≈ 19 700 of 20 000 on both.

How to match the statistic to its calibration is the user's decision: a larger
generated sample or a pooled-seed statistic, never a lower floor. Then flip the
default in `resolve`. Detail:
[note 37 §6.2–6.3](../../37-madevent-map-survey-and-soft-angle.md).
