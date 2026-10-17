---
type: Backlog Item
title: Per-point scale, coupling, cut and weight chain is f64-only
description: "Only the matrix element and the map are generic in F, and the map is built at f64; scales, αs, cuts, samplers and the record path are f64, so a lane batch would scatter/gather at every boundary."
area: performance
state: open
priority: medium
closes_when: "An audit lists every f64-only function on the per-point chain, the map's construction included, as necessary, by default and straight-line, or by default and branching on a value; the straight-line by-default ones are generic in F: Real, with byte-identical output at F = f64."
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
the cuts and the samplers, and the record path. The map stage is generic,
`PhaseSpaceMap<F: Real>` (`phasespace/channel.rs:50`), but `hadronic.rs`
builds it only at `f64` (`Sampler::Flat(RamboChannel<f64>)`,
`Multi(MultiChannel<f64>)`, `hadronic.rs:1732-1734`). A lane-batched
evaluation needs map, scale, coupling, cut and weight in one scalar type, or
each batch pays a scatter/gather at every `f64`-only boundary.

First step is the audit: enumerate the `f64`-only functions on that chain and
separate those that are `f64` by necessity (LHEF's printed fields, the PDF
grid's own storage) from those that are `f64` by default, and among the latter
mark the ones that branch on a value. The straight-line ones go generic here.
This is a prerequisite of [lane-batching-not-in-production](lane-batching-not-in-production.md).

The branching ones (cuts first) cannot go generic until `Real` has lane-wise
comparisons and selects; their port is
[per-point-branches-f64-only](per-point-branches-f64-only.md), which waits on
[real-has-no-lane-masks-or-bin-search](real-has-no-lane-masks-or-bin-search.md).
Clustering, the PDF lookup, the sampler and per-lane αs have their own items,
listed in the capstone's `blocked_by`.
