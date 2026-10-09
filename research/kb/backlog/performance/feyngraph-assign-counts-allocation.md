---
type: Backlog Item
title: FeynGraph assignment allocates a HashMap per candidate vertex
description: AssignWorkspace::assign() calls itertools .counts() per candidate vertex, ~340M allocations for pp→qq̃4l, which also makes small enumerations lose under threading.
area: performance
state: open
priority: medium
closes_when: The pinned feyngraph revision pre-computes per-vertex counts in AssignWorkspace::new(), and p p > j j j no longer runs slower on 16 threads than on one.
blocked_by: []
opened: 2026-07-17
tags: [feyngraph, diagram-enumeration, allocation, upstream]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1149-L1160", title: "TODO.md entry T106"}
---
In FeynGraph, `AssignWorkspace::assign()` (`src/diagram/workspace.rs`, ~L122)
builds a fresh `HashMap` via itertools `.counts()` per candidate vertex per
topology per subprocess: ~340M allocations for pp→qq̃4l. Fix: pre-compute the
per-vertex counts in `AssignWorkspace::new()`.

FeynGraph is a git dependency pinned by `rev` in `vibegraph-lib/Cargo.toml`
(and mirrored as the `research/refs/feyngraph` submodule), so the fix is an
upstream change or a fork plus a rev bump: a dedicated session.

Already mitigated on this side: topology caching per `(n_ext, n_loops)` and a
charge-conservation pre-filter (~86% of candidates eliminated).

Enumeration defaults to one thread (`EnumerationPool::Serial`,
`vibegraph-lib/src/diagrams/mod.rs:264`); `--parallel-diagrams` opts into the
`-j` pool. FeynGraph's internal fan-out is contended: 16 threads vs 1 is
0.137 s vs 0.083 s on `p p > j j j` (worse) and 3.36 s vs 8.90 s on
`p p > e+ e- j j j` (2.6x better). Removing the allocation is what would let
the small case parallelise too, and possibly let `Ambient` become the default.

Detail: [note 20](../../history/notes/20-eval-perf-2-plan.md),
[note 02](../../history/notes/02-reference-implementations.md).
