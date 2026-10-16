---
type: Backlog Item
title: How often a fresh MadEvent seed reads low after a suppressed spike
description: One of ten fresh gg_to_gg_cg seeds read 0.8 % low when the mean²/variance combination suppressed one spiky iteration; the frequency is unmeasured and the seed policy does not warn of it.
area: validation
state: open
priority: low
closes_when: The rate of such seeds is measured on two or more rows over ≥ 50 seeds, and the seed policy states it with the rule a reference uses to detect or absorb such a seed.
blocked_by: []
opened: 2026-10-15
tags: [madevent-reference, seeds, iteration-combination]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: v1, resource: "../../sprints/sigma-gg-cg/sessions/V1-report.md", title: "sigma-gg-cg V1, Found 2"}
---
Seed 84 of a fresh-directory batch reported 268510 pb, 0.8 % low. One `G3`
iteration hit 1.2121e6 ± 9.4e5, 4.1 times the mean, and the mean²/variance
combination suppressed it. That one seed lifts its batch's χ²/dof to 11.9.
The [seed policy](../../validation/madevent-reference-seed-policy.md)'s
max(quoted, spread/√n) absorbs it only when enough seeds are banked.
