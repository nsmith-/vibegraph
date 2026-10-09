---
type: Session Brief
title: "R-A: hygiene review of the evaluator (helas/eval)"
description: "Review helas/eval for maintainability, test non-vacuity, visibility and reusable abstractions."
status: draft
agent: claude (Opus), read-only
depends_on: [V1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [review protocol](review-protocol.md). Review the sprint branch
at V1's commit (the dispatch names the commit).

**Cluster.** `vibegraph-lib/src/helas/eval/` (about 26k lines) and its in-module tests. For context only: `benches/` and the study features (`bench-internals`, `eval-schedule-study`, `unchecked-study`).

**Already claimed, read and don't re-report:** evaluator-doc-comments-stale.

**Look especially at:** the size of `compile.rs`, `root_diagram.rs` and `layout.rs` (which functions want splitting); study-feature code that has outlived its study; kernel tests whose tolerance is far above `FUSED_TOL`-scale error. Also give a disposition for lorentz-coefficients-still-f64 (migrate, keep with a recorded decision, or file a refactor), as a finding with evidence. This module is the performance hot path, so a *fix here* proposal must not change the emitted op order or kernel bodies.
