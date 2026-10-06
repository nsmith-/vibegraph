---
type: Backlog Item
title: The α-survey on wide splits is iteration-limited
description: "Independent surveys land 0.15–0.74 apart in α L1 at any budget and the last step is still O(0.1–0.4): six ADAPT_ITERS binds, not points."
area: performance
state: open
priority: low
closes_when: If α quality becomes worth buying, a damped variable-iteration-count survey is measured against the six-iteration one (α L1 across seeds, σ spread at ≥5 seeds) and adopted or rejected.
blocked_by: []
opened: 2026-08-06
tags: [multichannel, alpha-survey, adapt-iters, wide-splits]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1036-L1043", title: "TODO.md entry T097"}
---
On wide channel splits, independent α-surveys at any budget land a tenth to a
third of the mixture mass apart: within-rung α L1 across survey seeds is
0.15–0.74. The last survey step is still O(0.1–0.4) at every rung. Six
`ADAPT_ITERS` (`vibegraph-cli/src/integrate.rs:100`) is the binding limit, not
the survey's point count. σ is insensitive to which α draw it runs under, which
is why the survey cap costs nothing.

The lever, if α quality is ever worth buying, is to vary the iteration count,
with damping, and leave the points alone. Nothing measured is limited by this
today.

Detail: [note 34 §2 S1](../../34-draw-followup-plan.md).
