---
type: Backlog Item
title: Real has no lane-wise comparisons, selects or bin search
description: "Real reduces a LaneField pack to one answer when comparing or indexing, so branchy code (VEGAS bins, jet clustering, cuts) cannot serve f64 and lanes alike; needs a design session."
area: performance
state: needs-user
priority: medium
closes_when: "A design concept, agreed with the user in a dedicated design session, specifies the extension to Real (or a companion trait) for lane-wise comparison masks, select/blend, masked reductions (any/all, argmin) and a searchsorted-style bin index, with its f64 and LaneField<N> semantics and the bit-for-bit contract at F = f64; and a first implementation lands with tests pinning both implementors."
blocked_by: []
opened: 2026-10-17
tags: [simd, lanes, real-trait, generics, design, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: seam, resource: "../../phase-space/importance-sampler-seam.md", title: "The proposed importance-sampler seam"}
---
The user asked (2026-10-17) for this to be designed in its own session, ahead
of the pluggable-sampler refactor, because the same trait extension is what
lets jet clustering run lane-parallel.

`Real` (`vibegraph-lib/src/helas/repr/mod.rs:97`) is `num_traits::Float` plus
constants. For `LaneField<N>` that leaves every data-dependent decision without
a per-lane form (`lane_field.rs:20-26`): `==` holds only when every lane is
equal, `<`/`>`/`partial_cmp` compare the pack lexicographically, the `Float`
predicates reduce with any/all, and `ToPrimitive` converts through lane 0. Code
that branches on a value, or turns a value into an index, is therefore correct
only at N = 1.

**What needs a per-lane form**, and the consumers that need it:
- **Comparison masks and select/blend** (`a < b` per lane, `mask.select(x, y)`):
  cuts, the VEGAS damping's branches, `max`/`min`-style guards, and every
  `if` in jet clustering's distance measures (`coupling/cluster/kt.rs`
  `dj`, `djb`, the massive-flavour test).
- **Masked reductions** (`any`, `all`, per-lane `argmin` over a candidate set):
  clustering's "smallest pair distance" step, and early exit when every lane
  has failed a cut.
- **Bin search** (a `searchsorted`-style per-lane index into a sorted edge
  array, and the gather that follows): VEGAS's bin lookup
  (`vegas.rs:585-590`), the PDF grid's x/Q² interpolation, histogramming.

**Design questions for the session** (not decided here): whether masks are a
new associated type on `Real` or a companion trait; how `PartialOrd`'s
pack-level meaning coexists with per-lane comparison without silent misuse;
the index type a lane-wise bin search returns, and how a gather is expressed;
how divergent control flow (different events taking different clustering
paths, or merging in a different order) is handled — masking, compaction into
same-path batches, or leaving it scalar; and the GPU mapping the same
abstraction must not preclude.

**Contract.** At `F = f64` every new operation must reduce to today's scalar
code bit-for-bit, so adopting it anywhere is a byte-identity refactor.

Consumers waiting on it:
[sampler-hardwired-to-vegas-grid](../feature/sampler-hardwired-to-vegas-grid.md),
[jet-clustering-f64-only](jet-clustering-f64-only.md),
[pdf-lookup-f64-only](pdf-lookup-f64-only.md), and the cut part of [per-point-chain-f64-only](per-point-chain-f64-only.md).
