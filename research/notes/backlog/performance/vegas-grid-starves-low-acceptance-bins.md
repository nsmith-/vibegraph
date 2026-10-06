---
type: Backlog Item
title: VEGAS grid starves bins that mostly fail the cuts
description: Per-bin Σ(f·w)² adaptation leaves low-acceptance bins wide, so the few points passing there carry the bin width into the weight tail.
area: performance
state: open
priority: medium
closes_when: A grid-adaptation rule (acceptance rescale, Σ|w| adaptation or a bin-width bound) is measured on the banked rows at ≥5 seeds and adopted or rejected on that record.
blocked_by: []
opened: 2026-09-30
tags: [vegas, weight-tail, unweighting, mlm, madevent-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L945-L951", title: "TODO.md entry T085"}
---
The grid adapts on `Σ(f·w)²` per bin. A bin that mostly fails the cuts is
starved, and the points that do pass in it carry its width. On every channel of
`pp_to_llj_mlm` that was dumped, the heaviest weights sit in single wide bins of
the first two coordinates. `@0`'s edge bins are 17× the average width (large
lepton-pair rapidity under `etal = 2.5`), and `@1`'s are 10–40× and 13–19×.

MadEvent adapts on `Σ|w|` (`dsample.f:1890`) and rescales each bin by the
inverse of its own acceptance, capped at 10⁴ (`dsample.f:2106-2124`). Adding
that rescale to `adapt_blocks_iteration`'s histogram, as a probe only, moved
`pp_to_llj_mlm`'s median Hill index from 1.8 to 2.2, and its worst channel's
from 1.0–1.2 to 1.5–1.6 (two seeds each, 150k × 10). It helps without curing
the tail, and the rest of the tail has not been located.

Every integration moves under any of these rules. Measure the acceptance
rescale, `Σ|w|` adaptation and a bin-width bound on the banked rows (σ pulls,
ε_unw, Hill index, ≥5 seeds) before adopting one.

Detail: [note 41 §4, "F-A Landed"](../../41-mlm-feature-sprint-plan.md).
