---
type: Backlog Item
title: "σ seed calibrations and hadronic gate readings no longer match their records"
description: "Seed-headroom readings moved since the 2026-09 census (ee_to_ee worst 1.44e-3 → 3.24e-3) and hadronic manifest readings no longer reproduce; identical before and after the hygiene sprint."
area: validation
state: open
priority: high
closes_when: "plan_for's seed figures, the census page and the hadronic manifest gate notes are re-recorded from one current run, and the cause of the drift since the census is identified."
blocked_by: []
opened: 2026-10-10
tags: [sigma-gate, calibration, seed-sweep]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: f-g2-report, resource: "../../sprints/hygiene/sessions/F-G2-report.md", title: "F-G2 report (hygiene sprint)"}
---
F-G2 Found 1–3. The hygiene sprint's manager ran `probe_gate_row_seed_headroom` at `669f3fa` (before the sprint) and at `3f401f0`: all 68 lines identical, so the drift predates the sprint. F-G2 set tightened `rel_tol` bounds from the larger of census and tip readings. Related: [sigma-calibration-comments-stale](sigma-calibration-comments-stale.md).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [F-G2](../../sprints/hygiene/sessions/F-G2-report.md).
