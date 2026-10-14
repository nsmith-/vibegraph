---
type: Backlog Item
title: "Two HELAS reference comparisons sit at 3e-6 and 1e-4"
description: "validate_helas and helas_kernel_composition compare against exact arithmetic at tolerances far above rounding, likely from mismatched parameter provenance."
area: validation
state: open
priority: low
closes_when: "Both sides' parameters are dumped and matched, and the tolerances drop to the arithmetic's own scale."
blocked_by: []
opened: 2026-10-10
tags: [helas, tolerance, provenance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-g1-report, resource: "../../sprints/hygiene/sessions/R-G1-report.md", title: "R-G1 report (hygiene sprint)"}
---
R-G1.8. AGENTS.md "bit-exact oracle first".

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-G1](../../sprints/hygiene/sessions/R-G1-report.md).
