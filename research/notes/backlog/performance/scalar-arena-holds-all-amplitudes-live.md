---
type: Backlog Item
title: Scalar arena holds every configuration amplitude live
description: Every Metric amplitude is a pinned configuration value and the level order keeps them all live, so the scalar arena cannot shrink below that.
area: performance
state: open
priority: low
closes_when: AMP2 is accumulated inside the evaluation pass and the order retires a level's amplitudes before the next fills, with the 2→6 scalar arena measurably smaller and AMP2 still matching MadGraph.
blocked_by: []
opened: 2026-10-05
tags: [evaluator, arena, amp2, cache]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1279-L1318", title: "TODO.md entry T114"}
---
The `Configs` bundle pins each bare `Metric` amplitude so `eval_amp2` can read
it after the pass, with its constant applied as a pool weight. The level order
then holds all of them live at once. Folding the constant chains cut the
2→6's arena 36%, but reading the configuration amplitudes bare (instructions
21 815 → 17 163) did not shrink the scalar arena further.

Shrinking it needs `AMP2` accumulated inside the pass, so an amplitude need not
outlive its JAMP sum, plus an order that retires a level's amplitudes before
the next level fills. Arena size is what put the 2→6 at lanes8 over the 2 MiB
L2 on Emerald Rapids (3.6 MiB), so this matters most for large processes at
wide lane widths.

Detail: [topdown-zen4-results §7](../../topdown-zen4-results.md),
[roofline-census-results](../../roofline-census-results.md).
