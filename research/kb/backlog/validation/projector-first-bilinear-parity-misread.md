---
type: Backlog Item
title: "term_reversed_parity would misread a bilinear written with a projector before its Gamma"
description: "A structure like ProjM(2,-1)*Gamma(3,-1,1) puts a projector at the row leg ahead of the Gamma, and its reversal would read +1; no model in reach writes one so."
area: validation
state: open
priority: low
closes_when: "term_reversed_parity reads such a chain through, or refuses it with a typed error, with a toy-structure test either way."
blocked_by: []
opened: 2026-10-16
tags: [amplitudes, sign-convention, ufo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: r1, resource: "../../sprints/amplitude-signs/sessions/R1-report.md", title: "R1 report, Found"}
---
R1 Found 13. The SM, SMEFTsim and the toy UFOs all write the Gamma first.
