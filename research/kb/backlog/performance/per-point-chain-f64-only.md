---
type: Backlog Item
title: Per-point scale, coupling, cut and weight chain is f64-only
description: Only the matrix element is generic in F; scales, αs, cuts, samplers and the record path are f64, so a lane batch would scatter/gather at every boundary.
area: performance
state: open
priority: medium
closes_when: "An audit lists every f64-only function on the per-point chain as necessary or by default, and the by-default ones are generic in F: Real."
blocked_by: []
opened: 2026-09-20
tags: [generics, simd, lanes, real-trait]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1354-L1362", title: "TODO.md entry T118"}
---
The matrix element is generic in `F: Real`, but the rest of the per-point
chain is `f64`: `RunningAlphaS::eval(&self, q: f64) -> f64`
(`vibegraph-lib/src/coupling/alphas.rs`), `coupling/scales`, the code around
the cuts and the samplers, and the record path. A lane-batched evaluation needs
scale, coupling, cut and weight in one scalar type, or each batch pays a
scatter/gather at every `f64`-only boundary.

First step is the audit: enumerate the `f64`-only functions on that chain and
separate those that are `f64` by necessity (LHEF's printed fields, the PDF
grid's own storage) from those that are `f64` by default. This is a
prerequisite of [lane-batching-not-in-production](lane-batching-not-in-production.md).
