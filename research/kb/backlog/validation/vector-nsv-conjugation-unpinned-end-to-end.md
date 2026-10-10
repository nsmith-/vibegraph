---
type: Backlog Item
title: "No banked row is known to pin incoming-vector nsv conjugation"
description: "The unit test now checks ε_out = conj(ε_in) at nhel = ±1; whether any MG-banked row has an incoming vector at a helicity where nsv matters is unchecked."
area: validation
state: open
priority: low
closes_when: "A banked row exercising an incoming massive and massless vector at nhel = ±1 is identified (or added), and making nsv inert fails it."
blocked_by: []
opened: 2026-10-10
tags: [helas, oracle, coverage]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-b-report, resource: "../../sprints/hygiene/sessions/R-B-report.md", title: "R-B report (hygiene sprint)"}
  - {id: f-b-report, resource: "../../sprints/hygiene/sessions/F-B-report.md", title: "F-B report (hygiene sprint)"}
---
R-B Found 3; R-B.4 fixed the unit test.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-B](../../sprints/hygiene/sessions/R-B-report.md), [F-B](../../sprints/hygiene/sessions/F-B-report.md).
