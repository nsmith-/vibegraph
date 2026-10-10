---
type: Backlog Item
title: "MadGraph line citations in run-card, LHEF and cuts code are unverified at the pin"
description: "About 33 citations in runcard, lhef and generate.rs and the myamp.f/setcuts.f citations in cuts.rs predate the 3.7.1 pin."
area: hygiene
state: open
priority: low
closes_when: "Every MadGraph file:line citation in those modules names the line at b7687064."
blocked_by: []
opened: 2026-10-10
tags: [citations, madgraph]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-f-report, resource: "../../sprints/hygiene/sessions/R-F-report.md", title: "R-F report (hygiene sprint)"}
  - {id: f-d-report, resource: "../../sprints/hygiene/sessions/F-D-report.md", title: "F-D report (hygiene sprint)"}
---
R-F Found; F-D Found 2 lists the drifted `cuts.rs` sites (`myamp.f:170/164/179`, `setcuts.f:431`). The earlier item for the claimed sites closed in the hygiene sprint.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-F](../../sprints/hygiene/sessions/R-F-report.md), [F-D](../../sprints/hygiene/sessions/F-D-report.md).
