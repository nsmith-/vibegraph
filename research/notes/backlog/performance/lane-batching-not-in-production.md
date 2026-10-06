---
type: Backlog Item
title: Lane-batched evaluation is not used in production
description: LaneField lanes beat scalar per event on every x86 target, but no integrator batches points, so the release assets run scalar evaluation only.
area: performance
state: blocked
priority: medium
closes_when: The x86-64-v3 release asset evaluates lane-batched (N = 4) behind a lanes4 σ/event gate against the scalar path, and whether the baseline asset batches too is decided.
blocked_by: [per-point-chain-f64-only, lane-eval-shares-one-alpha-s]
opened: 2026-09-23
tags: [simd, lanes, release, x86-64-v3]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1186-L1210", title: "TODO.md entry T110"}
---
`LaneField<N>` (`vibegraph-lib/src/helas/eval/lane_field.rs`) is a
`wide`-backed field whose ops all inline. On Emerald Rapids, median per-event
cost vs that build's own scalar:

| build | lanes2 | lanes4 | lanes8 |
|---|--:|--:|--:|
| `target-cpu=native` (AVX-512) | 0.57x | 0.32x | 0.25x |
| `x86-64-v3` (AVX2) | 0.59x | 0.33x | 0.34x |
| baseline x86-64 (SSE2) | 0.18x | 0.17x | 0.18x |

The baseline row predates `Real::mul_add_fast`, which made baseline scalar
`forward` 2.3–3.8x faster; its lane ratio must be re-measured before deciding
whether the baseline asset batches.

To adopt for the `x86-64-v3` asset (`release.yml`'s `-v3` musl leg): N = 4 is
the natural width there (lanes8 is two ymm halves and buys nothing). CI already
builds and tests under v3, so the lane-vs-scalar tests run there bit-exact.
Missing: a consumer (a batched integrator, which needs the per-point chain
generic in `F` and, for dynamic scales, per-lane `αs`) and a `lanes4` σ/event
gate against scalar. The best width is host-dependent: on Zen 4 width 8 wins on
every row ([topdown-zen4-results](../../topdown-zen4-results.md) §5).

Detail: [x86-avx2-perf-study-results](../../x86-avx2-perf-study-results.md),
AVX-512 section and "The two x86 release builds".
