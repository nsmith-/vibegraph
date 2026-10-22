---
type: Backlog Item
title: "WEIGHTED is computed three ways with different missing-order behaviour"
description: "diagrams/mod.rs ignores an order missing from the hierarchy, diagrams/chain.rs treats it as 0, and helas/eval/stitching.rs panics."
area: hygiene
state: open
priority: low
closes_when: "One Diagram::orders(model) plus one weighted helper serve all three sites, with the missing-order rule stated and tested."
blocked_by: []
opened: 2026-10-10
tags: [diagrams, coupling-orders, consistency]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-c-report, resource: "../../sprints/hygiene/sessions/R-C-report.md", title: "R-C report (hygiene sprint)"}
---
R-C.15.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-C](../../sprints/hygiene/sessions/R-C-report.md).

**A fourth site (2026-10-20).** The [weighted-cap sprint](../../sprints/weighted-cap/sprint.md)
added `ufo::weighted::order_weight` (the per-interaction value for the
automatic search's cap and start), with the missing-order rule (counts 0,
joins no class; MadGraph raises `KeyError`, `base_objects.py:897`, `:1448`)
stated once there. It is the natural single helper for the other three.
