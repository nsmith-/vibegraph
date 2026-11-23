---
type: Backlog Item
title: "The α survey spends 6 × 40 000 points on every multiplicity or mass part, whatever its share"
description: "MultiplicitySum::adapt_alphas runs a full survey per part, so mass-split VBF and VBS cards spend 340–910 s surveying before VEGAS starts."
area: performance
state: open
priority: medium
closes_when: "The survey's points are budgeted across parts (by channel count or the previous survey's spread) with a per-part floor. The survey wall time on 655 or 636 is measured, and σ is unchanged within the multi-seed criterion on the MLM rows and one mass-split VBF card."
blocked_by: []
opened: 2026-11-23
tags: [alpha-survey, multichannel, vbf, vbs, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
`MultiplicitySum::adapt_alphas` runs `ADAPT_ITERS × survey_points(neval)`
per part: 655 (6 parts) 1.44M points in 912 s; 636 (15 parts) 3.6M points
in 338 s; 14 (3 parts) 720k points in 844 s. On single-part cards a survey
point costs what a VEGAS warm-up point costs (195: 2.29 against 2.19 ms;
744: 0.36 against 0.35), so the survey's cost is points × integrand cost.

Related: [`mass-split-parts-thin-the-per-group-budget`](../performance/mass-split-parts-thin-the-per-group-budget.md),
[`adapt-survey-bounds-untested-at-hundreds-of-channels`](../performance/adapt-survey-bounds-untested-at-hundreds-of-channels.md).
Evidence: [L-T2 report](../../sprints/v023-orders-majorana-timeouts/sessions/L-T2-report.md), [phase sample](../../sprints/v023-orders-majorana-timeouts/v022-1009-phase-sample.md).

Production evidence: on simshop's e+e- + 4j MLM run (v0.2.2, 2 CPUs) the
α survey alone took about 2 h of an 8 h slot.
