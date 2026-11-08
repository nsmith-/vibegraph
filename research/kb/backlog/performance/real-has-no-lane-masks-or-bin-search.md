---
type: Backlog Item
title: Real has no lane-wise comparisons, selects or bin search
description: "Real reduces a LaneField pack to one answer when comparing or indexing, so branchy code (VEGAS bins, jet clustering, cuts) cannot serve f64 and lanes alike; needs a design session."
area: performance
state: needs-user
priority: medium
closes_when: "A design concept, agreed with the user in a dedicated design session, specifies the extension to Real (or a companion trait) for lane-wise comparison masks, select/blend, masked reductions (any/all, argmin), a searchsorted-style bin index with its gather and an order-preserving scatter-add, with its f64 and LaneField<N> semantics and the bit-for-bit contract at F = f64, and how VEGAS refine is vectorised over bins in f64; and a first implementation lands with tests pinning both implementors."
blocked_by: []
opened: 2026-10-17
tags: [simd, lanes, real-trait, generics, design, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: seam, resource: "../../phase-space/importance-sampler-seam.md", title: "The proposed importance-sampler seam"}
---
The user asked (2026-10-17) for this to be designed in a dedicated design
session with the user, ahead of the sampler's lane-generic draw, because the
same trait extension is what lets jet clustering run lane-parallel.

`Real` (`vibegraph-lib/src/helas/repr/mod.rs:97`) is `num_traits::Float` plus
constants. For `LaneField<N>` that leaves every data-dependent decision without
a per-lane form (`lane_field.rs:20-26`): `==` holds only when every lane is
equal, `<`/`>`/`partial_cmp` compare the pack lexicographically, the `Float`
predicates reduce with any/all, and `ToPrimitive` converts through lane 0, so
code that branches on a value or turns it into an index is correct only at N = 1.

**What needs a per-lane form**, and its consumers:
- **Comparison masks and select/blend**: cuts, `max`/`min`-style guards, every
  `if` in clustering's distance measures (`coupling/cluster/kt.rs` `dj`, `djb`).
- **Masked reductions** (`any`, `all`, per-lane `argmin`): clustering's
  smallest-pair step, early exit when every lane has failed a cut.
- **Bin search, gather and scatter-add**: the VEGAS per-point draw (bin index
  `vegas.rs:585`, edge gather `:587-588`) and accumulate (`:667`, `:910`), the
  PDF grid's x/Q² lookup, histogramming.

**Draft design:** [lane-scalar-trait](../../performance/lane-scalar-trait.md)
(2026-10-26, for the user's review) answers the session's questions: masks,
`PartialOrd` on lanes, index and gather, divergent control flow, scatter-add
order and the GPU mapping. **Separately**, VEGAS refine runs once per
iteration over bins, not per point, and stays `f64`: how its damping, smoothing,
serial edge rebuild and PR #48's `bound_bin_width` are vectorised across bins
or dimensions (`vegas.rs:695-742`).

**Contract.** At `F = f64` every new operation must reduce to today's scalar
code bit-for-bit, so adopting it anywhere is a byte-identity refactor.

Consumers waiting on it:
[sampler-draw-not-lane-generic](sampler-draw-not-lane-generic.md) (draw,
accumulate, refine over bins), [jet-clustering-f64-only](jet-clustering-f64-only.md),
[pdf-lookup-f64-only](pdf-lookup-f64-only.md),
[per-point-branches-f64-only](per-point-branches-f64-only.md) (cuts), and the map's
cut-aware windows ([seam design](../../phase-space/sampler-and-map-seams.md) D6).
