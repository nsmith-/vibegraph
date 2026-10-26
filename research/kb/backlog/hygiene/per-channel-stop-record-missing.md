---
type: Backlog Item
title: Per-channel, per-iteration stop data is not observable without patching
description: "ConvergenceReport has min_channel_accepted and per-channel totals, but not each kept iteration's (I, σ², n, accepted), pooled_scale or the running scaled_rel; P1 and B1 both needed throwaway patches."
area: hygiene
state: open
priority: low
closes_when: "A test-only or debug-level per-channel stop record (kept iterations, accepted counts, pooled_scale, max weight) is reachable from a probe, so the next audit is a probe run, not a patch."
blocked_by: []
opened: 2026-10-23
tags: [observability, budget, stop-rule, multichannel]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-23}
sources:
  - {id: p1, resource: "../../sprints/two-to-six-gate/sessions/P1-report.md", title: "two-to-six-gate P1 report, Found 2"}
  - {id: b1, resource: "../../sprints/two-to-six-gate/sessions/B1-report.md", title: "two-to-six-gate B1 report, instrumentation"}
---
B1's throwaway patch (an env-gated `VG_TRACE` per-channel dump and a
`max_weight` field on `BlockIteration`) is the shape such a record would take.

Filed at the close-out of [two-to-six-gate](../../sprints/two-to-six-gate/sprint.md).
