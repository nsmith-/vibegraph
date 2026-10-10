---
type: Backlog Item
title: "The parked egraph module pulls egglog and ordered-float into every build"
description: "helas/eval/egraph.rs has no production consumer, yet egglog and ordered-float are non-optional dependencies."
area: performance
state: open
priority: low
closes_when: "The module and both dependencies sit behind a feature (or cfg(test)), and a default build no longer compiles them."
blocked_by: []
opened: 2026-10-10
tags: [evaluator, dependencies, build-time]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-a-report, resource: "../../sprints/hygiene/sessions/R-A-report.md", title: "R-A report (hygiene sprint)"}
---
R-A.3. Related: [egraph-sharing-rewrites-need-global-extractor](egraph-sharing-rewrites-need-global-extractor.md).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-A](../../sprints/hygiene/sessions/R-A-report.md).
