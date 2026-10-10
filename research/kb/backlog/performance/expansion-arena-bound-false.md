---
type: Backlog Item
title: "The helicity-expansion arena bound fails on three of four processes"
description: "expansion_bounds_arenas asserts peak live slots < nodes/2; peaks are 72-83% of nodes because read-out scalars are pinned live, already at the pre-sprint base."
area: performance
state: open
priority: low
closes_when: "The bound excludes pinned read-out slots (or is replaced by a recorded per-process reading) and the test runs un-ignored."
blocked_by: []
opened: 2026-10-10
tags: [evaluator, arenas]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: f-a-report, resource: "../../sprints/hygiene/sessions/F-A-report.md", title: "F-A report (hygiene sprint)"}
---
F-A Stopped (peaks 80/111, 2009/2487, 162188/194371). The kb's 149k live-slot figure for the unpruned 2→6 is now 162k. Related: [scalar-arena-holds-all-amplitudes-live](scalar-arena-holds-all-amplitudes-live.md).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [F-A](../../sprints/hygiene/sessions/F-A-report.md).
