---
type: Backlog Item
title: Spacelike floor is 10–100x looser than the fiducial floor
description: Cuts::spacelike_floor() = pT_min² is provable but sits 10–100x below where cut-surviving |t| starts, which limits the bounded-t_max variance win.
area: performance
state: open
priority: low
closes_when: A tighter, provably valid spacelike floor derived from the active cuts replaces pT_min², and the bounded-t_max variance ratio on the llj card is re-measured against it.
blocked_by: []
opened: 2026-08-06
tags: [phase-space, cuts, t-channel, variance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1144-L1148", title: "TODO.md entry T105"}
  - {id: todo-t065, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L810-L813", title: "TODO.md entry T065"}
---
`Cuts::spacelike_floor()` (`vibegraph-lib/src/cuts.rs:645`) returns the largest
single-leg `pT_min` squared: 400 GeV² on the banked llj card. The D3
measurement found the cut-surviving region starts at `|t| ≈ 4 000–40 000 GeV²`,
10–100x above it.

The bounded-`t_max` window measured a 1.67–1.83x variance reduction over the
floored pole at the current floor; a tighter derived bound would scale that
win. The bound must stay provable (a floor that cuts accepted configurations
biases σ).

Detail: [note 28 §S2.5](../../history/notes/28-kt-spine-feature-sprint-plan.md).
