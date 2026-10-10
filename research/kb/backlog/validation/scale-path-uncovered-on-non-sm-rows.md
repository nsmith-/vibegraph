---
type: Backlog Item
title: "No test covers the αs-scaling path on non-SM rows"
description: "SMEFTsim rows take ScaleAwareAmplitude's reference path (not a monomial in G) and the toy models carry no aS, so validate_scale_couplings covers SM rows only."
area: validation
state: open
priority: low
closes_when: "A test asserts which non-SM rows fall back to the reference path and why, or the SMHLOOP couplings are tagged so they scale."
blocked_by: []
opened: 2026-10-10
tags: [scales, smeftsim, coverage]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: f-g1-report, resource: "../../sprints/hygiene/sessions/F-G1-report.md", title: "F-G1 report (hygiene sprint)"}
---
F-G1 Found 2.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [F-G1](../../sprints/hygiene/sessions/F-G1-report.md).
