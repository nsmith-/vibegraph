---
type: Backlog Item
title: Evaluator machine loads are not counted
description: The roofline census counts value traffic only; spills, header reloads and by-value kernel copies add loads that no measurement has counted per row.
area: performance
state: open
priority: low
closes_when: A per-row machine-load count (PMU or instrumented build) is recorded beside the roofline census's value traffic.
blocked_by: []
opened: 2026-10-03
tags: [roofline, profiling, evaluator]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1260-L1278", title: "TODO.md entry T113"}
---
`roofline_census` (`vibegraph-lib/src/helas/eval/roofline.rs`, ignored test)
counts every FP op of `eval_m2` and the arena bytes each instruction moves.
The verdict stands without the load count: arena traffic is 5–9 B/cycle at
scalar width, under a tenth of L1, and the evaluator is not bandwidth-bound.
But the real L1 load rate is higher than the value traffic by an unknown
amount: spills, instruction-header reloads and by-value kernel copies.

The L1 margin (3–20x) makes a reversal unlikely; the count matters for sizing
the operand-load share of the per-instruction cost that the AOT and top-down
studies point at.

Detail: [roofline-census-results §5](../../history/notes/roofline-census-results.md),
[topdown-zen4-results](../../history/notes/topdown-zen4-results.md).
