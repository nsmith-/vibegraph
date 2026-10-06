---
type: Backlog Item
title: u16 operand indices are unmeasured as a stream-size lever
description: u16 operand indices would halve the evaluator's instruction records and bound indices by type, but have not been measured.
area: performance
state: open
priority: low
closes_when: A u16-index build of the evaluator is timed against the current one on the eval_strategies rows, and kept or rejected on that measurement.
blocked_by: []
opened: 2026-10-04
tags: [evaluator, dispatch, bounds-checks, instruction-stream]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1319-L1326", title: "TODO.md entry T115"}
---
A `u16` operand index into a fixed `[T; 65536]` arena is checked by type, with
no extra instruction. Its larger effect is halving the instruction records,
which shrinks the instruction stream: a separate lever from bounds checks,
and it should be measured as one.

Context that bounds the bounds-check part: removing every check
(`unchecked-study` feature) is worth only 3.5–5.5% on Emerald Rapids, and
clamp/mask variants cannot beat that. Programs with more than 65 536 values
per arena would need a fallback.

Detail: [note 17 §10](../../17-bounds-check-elimination.md).
