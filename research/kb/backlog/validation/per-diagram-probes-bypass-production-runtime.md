---
type: Backlog Item
title: "Per-diagram evaluator probes never run the production Program"
description: "eval_single_diagram_slot lowers with the test-only lower::lower and evaluates with run_forward_slot; lower_flows, layout and fill_arenas are never exercised per diagram."
area: validation
state: open
priority: medium
closes_when: "The per-diagram probes route through Program/fill_arenas, or a hermetic test bridges the two paths on a process with an internal fermion."
blocked_by: []
opened: 2026-10-10
tags: [evaluator, oracle, blind-spot]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-a-report, resource: "../../sprints/hygiene/sessions/R-A-report.md", title: "R-A report (hygiene sprint)"}
---
R-A.2 (the docs were corrected in the hygiene sprint). The only bridge today is `test_whole_amplitude_equals_diagram_sum_eemumu`, with no internal fermion.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-A](../../sprints/hygiene/sessions/R-A-report.md).
