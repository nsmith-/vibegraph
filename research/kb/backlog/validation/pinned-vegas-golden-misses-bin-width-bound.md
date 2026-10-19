---
type: Backlog Item
title: The pinned-seed VEGAS golden never exercises the default bin-width bound
description: "test_pinned_seed_regression_grid_adapt passes unchanged under K = 6 because the bound never binds on its integrand, so the bound's arithmetic has no seeded golden."
area: validation
state: open
priority: low
closes_when: "A pinned-seed golden on an integrand where the default bound binds (checked: widest unbounded bin above 6 uniform widths) pins the refined grid, with tolerances at the algorithm's scale."
blocked_by: []
opened: 2026-10-18
tags: [vegas, golden, grid-rule]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-18}
sources:
  - {id: p3, resource: "../../sprints/vegas-low-acceptance-bins/sessions/P3-report.md", title: "vegas-low-acceptance-bins P3 report, Found"}
---
Filed at the close-out of [vegas-low-acceptance-bins](../../sprints/vegas-low-acceptance-bins/sprint.md).
