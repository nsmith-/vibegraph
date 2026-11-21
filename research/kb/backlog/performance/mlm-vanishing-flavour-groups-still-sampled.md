---
type: Backlog Item
title: "Flavour groups whose matrix element vanishes at the card are still compiled, surveyed and sampled"
description: "Groups whose couplings vanish at the card skip clustering but are still surveyed and sampled: ≥ 170 of 2176 channels and ≥ 7.4% of iteration points on g2HDM M300."
area: performance
state: open
priority: medium
closes_when: "Vanishing groups are dropped before the phase-space maps are built, σ is bit for bit on cards with none and unchanged within the multi-seed criterion on a g2HDM card, and the wall-time saving is measured."
blocked_by: []
opened: 2026-11-19
tags: [performance, mlm, flavour-groups, vegas, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
At M300, at least 3 of the 13 decompositions are entirely zero (23% of the
survey) and at least 87,040 of 1,172,281 first-iteration points land in
vanishing groups' floor allotment. The 2142 floor-bound channels as a whole
are a budget effect, not this.

Found by K-M1 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-M1-report.md), Found 1).
