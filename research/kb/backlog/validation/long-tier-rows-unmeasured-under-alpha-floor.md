---
type: Backlog Item
title: Long-tier rows not yet measured under the default α floor
description: "The 1e-4 α floor is the default since alpha-share-tails F2; ladder-bb, ladder-recarded, validate-mlm-samples, validate-generate-proton and the proton CLI reweight test run under it unmeasured."
area: validation
state: open
priority: medium
closes_when: "Each listed row runs once with --alpha-floor off and once with the default, and the cells are diffed: bit-identity means the floor did not touch the row, and any moved cell is read against its banked budget and ladder."
blocked_by: []
opened: 2026-10-31
tags: [alpha-floor, long-tier, budget-ladder, multichannel]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-31}
sources:
  - {id: f2, resource: "../../sprints/alpha-share-tails/sessions/F2-report.md", title: "alpha-share-tails F2 report, step 5 and Found 1"}
---
F2 measured the banked layer (6 cells moved, all pass) and the MLM σ gate
(only `pp_to_ttx_0j1j_mlm` moved, pass → pass). The 2 → 6 rows are heavily
touched (511–566 of 615 bbx channels under 1e-4 per seed) and were measured
by F1's ladder; their plan-budget gates were not rerun.
