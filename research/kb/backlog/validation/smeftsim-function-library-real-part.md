---
type: Backlog Item
title: "SMEFTsim's function library takes the real part where the grammar uses the SM definitions"
description: "SMEFTsim defines sec/csc/asec/acsc (and reglog, cot) on z.real; the expression grammar evaluates the SM forms on the complex argument."
area: validation
state: open
priority: low
closes_when: "Each model's function_library.py decides the evaluation, or a test shows no loaded model passes a complex argument."
blocked_by: []
opened: 2026-10-10
tags: [ufo, smeftsim, expressions]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: f-c-report, resource: "../../sprints/hygiene/sessions/F-C-report.md", title: "F-C report (hygiene sprint)"}
---
F-C Found 3; R-C.4.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [F-C](../../sprints/hygiene/sessions/F-C-report.md).
