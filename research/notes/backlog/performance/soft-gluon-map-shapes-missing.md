---
type: Backlog Item
title: Soft-gluon structures the phase-space maps do not shape
description: q* → q g has no one-sided 1/E_g shape, and llj's 1/(ŝ − ŝ_rest) soft-gluon structure on the spine remainder is unmapped.
area: performance
state: open
priority: low
closes_when: Both shapes are built and measured at ≥20 seeds (g g > g u u~ and pp_to_llj respectively), or rejected on a measurement; for llj, after a weight-tail decomposition binned in ŝ − ŝ_rest.
blocked_by: []
opened: 2026-09-23
tags: [phase-space-map, soft-gluon, llj, map-options]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1013-L1020", title: "TODO.md entry T095"}
---
Two shapes, each a map change measured on its own row:

- **`q* → q g`, one-sided `1/E_g`.** The symmetric soft map also shapes the
  quark end, which `P_qq` lacks. `g g > g u u~` reads 1.02 ± 0.09 under
  `soft-all` against the soft-emission baseline (twenty seeds), so the
  symmetric map buys nothing there.
- **llj's soft gluon.** Its structure is a `1/(ŝ − ŝ_rest)` on the spine's
  remainder invariant, the same variable as the `Z/γ*` pole, so the two compete.
  If built at all, it would be a second channel per spine. First decompose the
  weight tail binned in `ŝ − ŝ_rest` to see whether that region carries it.

Detail: [note 37 §4 and §6](../../37-madevent-map-survey-and-soft-angle.md).
