---
type: Backlog Item
title: Cuts and other branchy per-point code are f64-only
description: "Cuts and the per-point code that branches on a value cannot be generic in F: Real until Real has lane-wise comparisons and selects; the per-point audit lists them."
area: performance
state: blocked
priority: medium
closes_when: "Every function the per-point audit lists as f64 by default and branching on a value (cuts first) is generic in F: Real on the lane-mask extension, with a lane-vs-scalar test per function giving the same pass/fail and values per lane, and byte-identical output at F = f64."
blocked_by: [real-has-no-lane-masks-or-bin-search, per-point-chain-f64-only]
opened: 2026-10-17
tags: [generics, simd, lanes, real-trait, cuts]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
---
[per-point-chain-f64-only](per-point-chain-f64-only.md) audits the per-point
chain and makes its straight-line `f64`-by-default functions generic. The rest
branch on a value: a cut compares an observable to a threshold, and a failed
point takes a different path from a passing one. On `LaneField<N>` a
comparison reduces the pack to one answer, so this code is correct only at
N = 1 until
[real-has-no-lane-masks-or-bin-search](real-has-no-lane-masks-or-bin-search.md)
lands comparison masks, selects and any/all reductions.

The port needs per-lane pass/fail masks, a select for the weight of failed
lanes, and an early exit when every lane has failed. Which functions are in
scope is the audit's list, not a guess made here.

Feeds [lane-batching-not-in-production](lane-batching-not-in-production.md).
