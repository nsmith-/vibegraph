---
type: Backlog Item
title: "Seed χ²/dof reads far below 1 on clean fixed-beam cards"
description: "On clean e+ e- > e+ e- $$ z control cards, seed χ²/dof reads 0.28 over 20 seeds (etal 5, 160k × 6) and 0.61 (ptl 40): quoted errors may overstate the scatter."
area: validation
state: open
priority: low
closes_when: "40-seed ensembles and the within-seed covariance between iterations are measured on a clean card, and the low χ²/dof is either nominal or attributed."
blocked_by: []
opened: 2026-10-17
tags: [vegas, error-calibration, seeds]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v1, resource: "../../sprints/ee-ee-nsz-low-budget/sessions/V1-report.md", title: "ee-ee-nsz-low-budget V1 report, Found 2"}
---
A χ²/dof far below 1 is as loud a warning as one far above (AGENTS.md). The
ee_to_ee_nsz probe's control cards read it low at ×4: 0.280 over 20 seeds on
etal 5 and 0.61 on ptl 40
([V1](../../sprints/ee-ee-nsz-low-budget/sessions/V1-report.md)). One
possibility is that grid refinement anticorrelates successive iterations, so
summing their variances overstates the run's error. That is unmeasured.
