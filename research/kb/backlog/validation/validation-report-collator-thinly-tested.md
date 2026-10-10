---
type: Backlog Item
title: "The report collator's cell logic has no tests"
description: "Flipping the collator's severity sort or dropping its fail bump fails nothing; the hygiene sprint added vocabulary and schema tests only."
area: validation
state: open
priority: medium
closes_when: "A fixture harness of manifest and row files tests cell rendering, severity ordering and the mode/tier checks, each shown to fail on a one-line mutation."
blocked_by: []
opened: 2026-10-10
tags: [validation-report, non-vacuity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-f-report, resource: "../../sprints/hygiene/sessions/R-F-report.md", title: "R-F report (hygiene sprint)"}
  - {id: f-cli-report, resource: "../../sprints/hygiene/sessions/F-CLI-report.md", title: "F-CLI report (hygiene sprint)"}
---
R-F.2. F-CLI typed the vocabularies and checked the manifest schema (`validation-report` now has 4 tests).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-F](../../sprints/hygiene/sessions/R-F-report.md), [F-CLI](../../sprints/hygiene/sessions/F-CLI-report.md).
