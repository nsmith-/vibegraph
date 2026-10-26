---
type: Backlog Item
title: "The proton path's massless-beam residuals against MadGraph are not recorded"
description: "MadGraph's √stot uses 0.938 GeV beams: it equals 2√(E1E2) at equal beams and differs by 1.0e-9 at 6500 x 4000; its lab boost differs by about 2e-8 at equal beams too."
area: validation
state: open
priority: low
closes_when: "A phase-space or proton concept states both residuals, with the expression for each, so a per-event LHEF comparison tighter than 1e-8 knows what to expect."
blocked_by: []
opened: 2026-10-26
tags: [proton, kinematics, beams, tolerance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f1, resource: "../../sprints/unequal-beams/sessions/F1-report.md", title: "unequal-beams F1 report, Found 2"}
  - {id: v1, resource: "../../sprints/unequal-beams/sessions/V1-report.md", title: "unequal-beams V1 report, Found 4"}
---
By decision, this crate's proton √s is `2√(E₁E₂)` and its lab boost uses
massless beams. MadGraph's `stot` and `get_betaz` use 0.938 GeV beams
(`genps.f:653-676`, `unwgt.f:645-656` at `b7687064`). V1 corrected F1's single
"≈2e-8" into two numbers:

- √stot: identical at equal beams (`stot = 4E²`), −1.0e-9 relative at
  6500 × 4000;
- the lab boost's rapidity: about 2e-8, present at equal beams too.

Both are far below every tolerance. No concept records them; the doc on
`collider_energy` (`coupling/scales.rs`) has the √stot expression only
([F1](../../sprints/unequal-beams/sessions/F1-report.md),
[V1](../../sprints/unequal-beams/sessions/V1-report.md)).
