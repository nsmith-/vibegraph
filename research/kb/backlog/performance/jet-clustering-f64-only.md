---
type: Backlog Item
title: Jet clustering is f64-only and cannot run lane-parallel
description: "The kT clustering behind dynamic scales and MLM (coupling/cluster) is f64 with data-dependent branches and merge order, so a lane batch must drop to scalar at every clustered event."
area: performance
state: blocked
priority: low
closes_when: "The clustering in coupling/cluster is generic over F: Real using the lane-mask extension, with a lane-vs-scalar test showing identical scales and merge histories per lane on the clustered rows, and byte-identical output at F = f64; or a measurement records that clustering is too small a share of per-event time to be worth it."
blocked_by: [real-has-no-lane-masks-or-bin-search]
opened: 2026-10-17
tags: [simd, lanes, clustering, scales, mlm, real-trait, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
---
The user asked (2026-10-17) for jet clustering to be parallelisable on the
same footing as the matrix element and the sampler.

`coupling/cluster` (`kt.rs`, `setclscales.rs`, `rewgt.rs`; ~5000 lines) takes
`[f64; 4]`/`[f64; 5]` momenta and returns `f64` scales. Its hot path is
branchy: `dj`/`djb` pick a measure by flavour and mass (`kt.rs:238-290`), each
step takes the smallest pair distance, and the merge order — so the number of
steps and which particles survive — differs event by event. That divergence is
the core design problem; a lane mask alone does not make it vectorise.

**First step:** measure clustering's share of per-event time on the clustered
rows (`pp_to_llj_dyn`, the MLM rows, `pp_to_jj`). If it is small next to the
matrix element, the item can close on that measurement with clustering left
scalar behind a gather. Otherwise port it once
[real-has-no-lane-masks-or-bin-search](real-has-no-lane-masks-or-bin-search.md)
has settled how comparisons, per-lane argmin and divergent paths are written.

Related: [per-point-chain-f64-only](per-point-chain-f64-only.md) (the rest of
the per-point chain), [lane-batching-not-in-production](lane-batching-not-in-production.md).
