---
type: Backlog Item
title: "Diagram generation and reweight planning functions are long"
description: "generate_sets_inner (179 lines) and generate_undecayed (144) and several chain/engine/topo/parse functions exceed 110 lines."
area: hygiene
state: open
priority: low
closes_when: "generate_sets_inner and generate_undecayed are split along R-C.17's proposed boundaries, and generate_undecayed resolves each leg once."
blocked_by: []
opened: 2026-10-10
tags: [maintainability, diagrams]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-c-report, resource: "../../sprints/hygiene/sessions/R-C-report.md", title: "R-C report (hygiene sprint)"}
---
R-C.17 names the split points; the other long functions (`glue`, `chain_sets`, `stitch_set`, `polynomial_group`, `plan_subprocess`, `build_feyngraph_model`, `extract_process`) are candidates.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-C](../../sprints/hygiene/sessions/R-C-report.md).
