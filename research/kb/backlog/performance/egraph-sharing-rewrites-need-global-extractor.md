---
type: Backlog Item
title: E-graph sharing rewrites are invisible to tree-cost extraction
description: The remaining e-graph rule families are sharing rewrites that tree-cost extraction cannot see, so the egglog path cannot pay off without a global extractor.
area: performance
state: blocked
priority: low
closes_when: A global (ILP) extractor with a compute-aware WorkCost selects a sharing rewrite on a process with three or more consumers and beats the manual lowering, or the egglog path is retired.
blocked_by: []
opened: 2026-07-11
tags: [egraph, egglog, cse, extraction]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1161-L1165", title: "TODO.md entry T107"}
---
The rule families left to try are *sharing* rewrites, whose benefit appears
only when a subterm is reused; tree-cost extraction scores each use separately
and never picks them. Getting to a yes needs three things that do not exist:
a global/ILP extractor, a compute-aware `WorkCost`, and a demo process with at
least three consumers of a shared subterm.

Substrate in the tree: the egglog round-trip skeleton
(`vibegraph-lib/src/helas/eval/egraph.rs`, parked) and the DAG-cost extractor.

Caveat for any measurement: the lowering path emits a ±1-CSE-node AST depending
on the hash seed; it is correctness-neutral (same |M|²) but moves node counts.

Detail: [note 14](../../history/notes/14-egglog-notes.md),
[note 15 §4–5](../../history/notes/15-eval-optimization-plan.md).
