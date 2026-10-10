---
type: Backlog Item
title: "Production draws configurations from the pruned AMP2, which is never compared with MadGraph"
description: "amplitude_oracle compares only the unpruned AMP2; the pruned-vs-unpruned gap is reported, not bounded (39.5% on gg_to_ttx when introduced)."
area: validation
state: open
priority: medium
closes_when: "The pruned AMP2 is asserted against MadGraph's, or the gap is bounded with a stated reason."
blocked_by: []
opened: 2026-10-10
tags: [amplitudes, configuration-draw, oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-g1-report, resource: "../../sprints/hygiene/sessions/R-G1-report.md", title: "R-G1 report (hygiene sprint)"}
---
R-G1 Found 1; the module doc now says the comparison is measured, not asserted.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-G1](../../sprints/hygiene/sessions/R-G1-report.md).
