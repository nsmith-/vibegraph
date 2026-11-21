---
type: Backlog Item
title: "An integrand that no point passes runs to the max-points cap before refusing"
description: "On GJ-4Jets HT-40to100 (σ = 0 at LO by its cuts) integrate ran 181 iterations to the 400M-point cap: ~520 of 741 s on iterations that could not change the outcome."
area: performance
state: open
priority: medium
closes_when: "integrate refuses with the existing 'none passed the cuts' refusal at the first kept iteration in which no point has passed the cuts, with a test, and cards with passing points are bit for bit."
blocked_by: []
opened: 2026-11-19
tags: [vegas, refusal, batch, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Found by K-G1 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-G1-report.md), Found 1).
