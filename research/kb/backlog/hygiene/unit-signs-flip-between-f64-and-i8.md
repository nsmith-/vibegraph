---
type: Backlog Item
title: "±1 signs convert between f64 and i8 in the evaluator"
description: "root_lorentz accumulates signs as f64 and converts by comparison; root_diagram's fermion sign is i8 cast to f64 in lower."
area: hygiene
state: open
priority: low
closes_when: "A small Sign type carries ±1 end to end and the casts go."
blocked_by: []
opened: 2026-10-10
tags: [evaluator, types]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-a-report, resource: "../../sprints/hygiene/sessions/R-A-report.md", title: "R-A report (hygiene sprint)"}
---
R-A.18.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-A](../../sprints/hygiene/sessions/R-A-report.md).
