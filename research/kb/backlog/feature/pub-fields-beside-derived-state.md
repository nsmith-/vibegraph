---
type: Backlog Item
title: "Public fields sit beside derived state they can desynchronise"
description: "EvaluatedModel.param_values, UFOModel.particles/vertices, Diagram's fields, ColorBasis.elements and RunCard's typed fields are pub while derived state beside them is not."
area: feature
state: needs-user
priority: low
closes_when: "Each listed field is private behind a read accessor (or the derived state is computed on read), decided together with the supported library surface."
blocked_by: []
opened: 2026-10-10
tags: [api, visibility, invariants]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-c-report, resource: "../../sprints/hygiene/sessions/R-C-report.md", title: "R-C report (hygiene sprint)"}
  - {id: r-g1-report, resource: "../../sprints/hygiene/sessions/R-G1-report.md", title: "R-G1 report (hygiene sprint)"}
  - {id: r-f-report, resource: "../../sprints/hygiene/sessions/R-F-report.md", title: "R-F report (hygiene sprint)"}
  - {id: r-a-report, resource: "../../sprints/hygiene/sessions/R-A-report.md", title: "R-A report (hygiene sprint)"}
---
Sites: R-C.11, R-G1.16, R-F.19. Related surface questions: `pub fn`s returning types from private modules (R-A.11, `EvalError`, `RescaleFallback`, `PoolTagCensus`) and the empty `ParsingOptions` parameter (R-C.12). Decide with [lib-pub-api-surface-unaudited](../feature/lib-pub-api-surface-unaudited.md).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-C](../../sprints/hygiene/sessions/R-C-report.md), [R-G1](../../sprints/hygiene/sessions/R-G1-report.md), [R-F](../../sprints/hygiene/sessions/R-F-report.md), [R-A](../../sprints/hygiene/sessions/R-A-report.md).
