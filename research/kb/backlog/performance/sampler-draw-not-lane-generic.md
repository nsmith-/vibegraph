---
type: Backlog Item
title: The VEGAS draw and accumulate run at f64 only, not across lanes
description: "The sampler trait's F: Real draw and accumulate are implemented only at f64; VegasGrid needs per-lane bin index, edge gather and scatter, and refine vectorised over bins, for a lane batch to draw."
area: performance
state: blocked
priority: medium
closes_when: "VegasGrid and the uniform sampler implement the trait's draw and accumulate at LaneField<N>, with a lane-vs-scalar test giving the same u, 1/p and bin index per lane from the same draws; refine is vectorised over bins in f64 as the design concept specifies; and at F = f64 the accumulate order is unchanged, so grid.bin.zst, LHE and validation report stay byte-identical."
blocked_by: [real-has-no-lane-masks-or-bin-search, sampler-hardwired-to-vegas-grid]
opened: 2026-10-17
tags: [vegas, sampler, simd, lanes, real-trait, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: seam, resource: "../../phase-space/importance-sampler-seam.md", title: "The proposed importance-sampler seam"}
---
[sampler-hardwired-to-vegas-grid](../feature/sampler-hardwired-to-vegas-grid.md)
gives the sampler trait signatures generic over `F: Real` but implements them
at `f64` only. This item makes the VEGAS grid draw and accumulate across lanes,
on the extension
[real-has-no-lane-masks-or-bin-search](real-has-no-lane-masks-or-bin-search.md)
designs.

Only two VEGAS steps run per point:

- **Draw.** `(pos as usize).min(nbins − 1)` (`vegas.rs:585`) is a per-point
  integer bin, then a gather of two edges (`:587-588`). On `LaneField` an
  integer conversion goes through lane 0, so it needs the per-lane bin index
  and gather. The uniform is already `u64_to_uniform`, pinned by the trait item.
- **Accumulate.** A scatter-add by bin index (`vegas.rs:667`, `:910`). The
  `-j` contract rests on its point order, so at `F = f64` the order must not
  move: a lane scatter-add that regroups the histogram sums would change the
  grid bytes. The lane form must add in point order, or be proven to give the
  same sums.

**Refine stays `f64`.** Damping (`vegas.rs:695-705`), smoothing (`:707-713`),
the edge rebuild (`:715-742`) and PR #48's `bound_bin_width` run once per
iteration over bins, not per point. They are vectorised across bins or
dimensions in `f64`, in whatever form the design session settles.

Feeds [lane-batching-not-in-production](lane-batching-not-in-production.md).
