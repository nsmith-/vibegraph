---
type: Backlog Item
title: Clustered scale path allocates per event
description: The clustered-scale path heap-allocates several Vecs per event, 2–3 times per event, and costs more than the matrix element it serves.
area: performance
state: open
priority: medium
closes_when: setclscales/cluster run on a reused scratch struct with no per-event heap allocation, probe_scale_cost shows the drop, and event bytes at fixed seed are unchanged.
blocked_by: []
opened: 2026-07-17
tags: [scales, ckkw, allocation, setclscales]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1136-L1143", title: "TODO.md entry T104"}
---
The scale path costs 1 857–2 802 ns per point against a 581–1 524 ns matrix
element. The merge tables are already hoisted (`ClusterInput::tables`), but the
per-event work still allocates: `ScaleChoice::cluster_history` builds its
momentum `Vec` per call (`vibegraph-lib/src/coupling/scales.rs:561`), and the
clustering in `vibegraph-lib/src/coupling/cluster/setclscales.rs` allocates
several `Vec`s per call. Under matching it runs 2–3 times per event.

Fix: thread a scratch struct through `setclscales`/`cluster` and reuse it across
events. A real refactor, worth its own session.

Gate: bit-for-bit event bytes at fixed seed. Instrument: `probe_scale_cost`
(`vibegraph-lib/tests/validate_sigma.rs`).

Detail: [note 30 §7.2](../../30-perf-baseline-timings.md),
[note 31 §E4](../../31-perf-sprint-3-plan.md),
[note 32 S5](../../32-perf-addendum-plan.md).
