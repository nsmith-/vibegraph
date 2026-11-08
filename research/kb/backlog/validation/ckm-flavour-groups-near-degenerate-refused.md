---
type: Backlog Item
title: "CKM-model flavour groups separated by 1e-6 to 1e-8 are refused as rounding"
description: "The 9.08e-7 split is physical and kept apart (#103); the 3.88e-8 separation reported for CKM card 089 has never been reproduced or attributed."
area: validation
state: open
priority: medium
closes_when: "Card 089 is run and its closest flavour-group pair is identified and shown physical (kept apart) or rounding (merged at GROUP_REL_TOL), with the pair named in a test or in the run record."
blocked_by: []
opened: 2026-10-20
tags: [flavour-groups, ckm, proton, triage, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
`derive_flavor_groups` no longer refuses near-degenerate groups (#103):
- pairs closer than `GROUP_REL_TOL` = 1e-9 merge as rounding (the worst
  measured is 1.48e-10);
- the rest are kept apart.

H-N1 showed the 9.08e-7 γ+jets split (`u b~ > a u b~` against `u~ b > a u~ b`)
to be the reflection-odd part of |M|², carried by the W width: CP holds to
3e-16, and the split vanishes at Γ_W = 0. Forcing that pair into one group
moves σ by −0.13% ± 0.47% ([H-B1](../../sprints/cms-gridpack-coverage/sessions/H-B1-report.md)).

Still open: the **3.88e-8** separation that the
[CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md) reported
for card 089. None of H-N1's reproductions shows it. The closing campaign
`main-202a540-200-pc` did not sample card 089, and no run records which pair it
kept apart ([flavour-group-splits-not-recorded](../feature/flavour-group-splits-not-recorded.md)).
Given #103's rule, either outcome is handled. What remains is to confirm the
pair is physical, or else rounding the tolerance missed.
