---
type: Backlog Item
title: "A two-pass generate restart recomputes the blocked pass from block 0"
description: "On multi-process cards the two-pass strategies restart, and the restart recomputes every block it already consumed, doubling accept/reject there."
area: performance
state: open
priority: low
closes_when: "The restart's cost is measured on a multi-@N card, and either the consumed blocks' prepared points are cached so the second pass reuses them (output unchanged), or the measured cost is recorded with the reason it stays."
blocked_by: []
opened: 2026-11-23
tags: [performance, generate, multi-process, unweighting]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: ls1, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-S1-report.md", title: "L-S1 report: Found 6"}
---
The two-pass write strategies restart on cards with more than one process id
(`Buffer` with several `@N`). The restart replays the blocked pass from
block 0, recomputing every block. Caching the consumed blocks' prepared
points would make the second pass free, at about one event record per event
of memory.
