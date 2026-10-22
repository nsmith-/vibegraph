---
type: Backlog Item
title: Without coupling_orders.py the order hierarchy falls back differently from MadGraph
description: "MadGraph sets every order's hierarchy to 1 (QED 2 only when the orders are exactly QCD and QED) when a UFO lacks coupling_orders.py; vibegraph always uses QCD=1, QED=2 and counts other orders 0."
area: feature
state: open
priority: low
closes_when: "ParsedModel::parse applies MadGraph's fallback hierarchy (base_objects.py:1379-1386) when coupling_orders.py is absent, by a test on a copy of a toy UFO without the file."
blocked_by: []
opened: 2026-10-20
tags: [ufo, coupling-orders, weighted, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: f1, resource: "../../sprints/weighted-cap/sessions/F1-report.md", title: "weighted-cap F1 report, Found 2"}
  - {id: v1, resource: "../../sprints/weighted-cap/sessions/V1-report.md", title: "weighted-cap V1 report, Found 2"}
---
No model in reach lacks the file. On a copy of `vibegraph_toy_weighted_UFO`
without it, `fw~ fw > fw~ fw` gives 4 diagrams here against 2 under
MadGraph's default (`fw~ fw > fw~ fw xw`: 16 against 8), measured in the
[weighted-cap sprint](../../sprints/weighted-cap/sprint.md).
