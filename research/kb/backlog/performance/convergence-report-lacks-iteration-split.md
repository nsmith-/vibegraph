---
type: Backlog Item
title: ConvergenceReport does not expose each iteration's point split
description: "Per-iteration points, floors and floor-bound counts per channel exist only inside adapt_grids_budget, so a probe must patch in a trace to read where an allocation spent its budget."
area: performance
state: open
priority: low
closes_when: "ConvergenceReport (or a per-iteration record beside it) carries each iteration's per-channel points and floors and the floor-bound count, and the 2 → 6 ladder probe's TSV records them without local instrumentation."
blocked_by: []
opened: 2026-10-26
tags: [multichannel, allocation, observability, probe]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: p1, resource: "../../sprints/floor-bound-neyman/sessions/P1-report.md", title: "floor-bound-neyman P1 report, Method and Found 3"}
---
P1 read the floor-bound counts and the fraction of points above the floors
from an uncommitted `VG_TRACE` patch to `budget.rs` and `vegas.rs`; the
committed TSV records only the last iteration's α-share floor count. Any
allocation or floor change is judged by those numbers, so they belong on the
report the run already returns.
