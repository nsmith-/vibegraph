---
type: Backlog Item
title: "Test-side banked readers and census scaffolding are duplicated"
description: "Readers for amplitude tables, param cards, momenta, diagram counts, output dirs, MadGraph dumps and the alpha_s run lists are copied across test files, with same-named helpers behaving differently."
area: hygiene
state: open
priority: low
closes_when: "A tests/common banked module and a census module hold one copy of each reader, and gzip is read through flate2."
blocked_by: []
opened: 2026-10-10
tags: [tests, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-g1-report, resource: "../../sprints/hygiene/sessions/R-G1-report.md", title: "R-G1 report (hygiene sprint)"}
  - {id: r-g2-report, resource: "../../sprints/hygiene/sessions/R-G2-report.md", title: "R-G2 report (hygiene sprint)"}
---
R-G1.17–.20, .22, .27; R-G2.16–.18 (the six `gzip -dc` shell-outs and two `GRID_ALPHA_S_RUNS` definitions included).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-G1](../../sprints/hygiene/sessions/R-G1-report.md), [R-G2](../../sprints/hygiene/sessions/R-G2-report.md).
