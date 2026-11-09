---
type: Backlog Item
title: "The 2→6 info rows show VEGAS iteration χ²/dof of 60 and 560"
description: "At 600000×8 the single-seed 2→6 rows read integration χ²/dof 60.13 (bbx) and 559.83 (uux), so their iterations are not self-consistent and the quoted errors are suspect."
area: validation
state: open
priority: low
closes_when: "The cause of the large iteration χ²/dof on bbx_to_ccx_emmm_qcd0 and uux_to_ccx_emmm_qcd0 is found (adaptation still moving, a heavy tail, or a combination rule), and the rows' quoted error is shown calibrated against their seed spread."
blocked_by: []
opened: 2026-11-09
tags: [sigma, 2to6, vegas, error-calibration]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-09}
sources:
  - {id: iz2, resource: "../../sprints/v02-last-refusals/sessions/I-Z2-report.md", title: "I-Z2 report"}
---
Sprint I's validate-sigma-2to6 on 2b26c109:
- bbx: +0.06%, pull +0.31;
- uux: +0.48%, pull +1.85.

Both are within the recorded 4.8% single-seed band, and both are info-only.
The skipped ladder-2to6 (the user's call for v0.2.0, estimated 6–8 h on a
4-core container) is the natural place to read it.
