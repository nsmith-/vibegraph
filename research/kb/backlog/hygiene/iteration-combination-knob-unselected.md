---
type: Backlog Item
title: "IterationCombination is threaded through integrate_channels but never selected"
description: "All five production callers pass IterationCombination::default(); InverseVariance only serves tests, and combine_kept ignores its rule when a channel's point counts vary."
area: hygiene
state: open
priority: low
closes_when: "The combination is a test-only grid setting and integrate_channels drops the parameter, or a production caller selects it and combine_kept honours the rule for varying point counts (pinned by a test)."
blocked_by: []
opened: 2026-10-10
tags: [vegas, budget, dead-knob]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-d-report, resource: "../../sprints/hygiene/sessions/R-D-report.md", title: "R-D report (hygiene sprint)"}
---
R-D.4 and R-D Found 2. The `Target` refusal and the module-doc precondition in `budget.rs` exist only for the unselected arm.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-D](../../sprints/hygiene/sessions/R-D-report.md).
