---
type: Backlog Item
title: Check MadEvent CP-mirror channel pairs as a free reference audit
description: At symmetric beams, mirror channels G8 and G9 of u u~ > w+ b w- b~ / t t~ must integrate equally, yet MadEvent's values differ by up to 5 % within one run.
area: validation
state: open
priority: low
closes_when: Reference runs at symmetric beams record each mirror pair's ratio, and a reference whose pairs disagree beyond their quoted errors is flagged.
blocked_by: []
opened: 2026-10-15
tags: [madevent-reference, audit, channels]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: v2, resource: "../../sprints/sigma-uux-wbwb-veto/sessions/V2-report.md", title: "sigma-uux-wbwb-veto V2, Found 2"}
---
Seed 20260925 read G8 = 0.011267 and G9 = 0.011890 on a process where the two
are CP mirrors at symmetric beams. A per-run mirror-pair check costs nothing,
and it flags a reference's coverage failure without any external estimator.
