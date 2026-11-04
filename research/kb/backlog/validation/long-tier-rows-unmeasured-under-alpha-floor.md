---
type: Backlog Item
title: Long-tier rows not yet measured under the default α floor and the spine remainder mixture
description: "Long-tier rows unmeasured under the default 1e-4 α floor (#84) and the 2 → 6 rows under the spine remainder mixture (#81), which moves 48 of their channels after #84's ladder."
area: validation
state: open
priority: medium
closes_when: "Each listed row runs once with --alpha-floor off and once with the default, and the cells are diffed: bit-identity means the floor did not touch the row, and any moved cell is read against its banked budget and ladder. The bbx and uux 2 → 6 ladders are rerun on main after #81 at 20 paired seeds (600k, 1.2M), with the per-seed sd ratio and χ²/dof compared with #84's split + floor arm."
blocked_by: []
opened: 2026-10-31
tags: [alpha-floor, long-tier, budget-ladder, multichannel]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
sources:
  - {id: f2, resource: "../../sprints/alpha-share-tails/sessions/F2-report.md", title: "alpha-share-tails F2 report, step 5 and Found 1"}
  - {id: tf2, resource: "../../sprints/spine-fallback/sessions/T-F2-report.md", title: "spine-fallback T-F2 report, the map-key census of every gated row"}
---
F2 measured the banked layer (6 cells moved, all pass) and the MLM σ gate
(only `pp_to_ttx_0j1j_mlm` moved, pass → pass). The 2 → 6 rows are heavily
touched (511–566 of 615 bbx channels under 1e-4 per seed) and were measured
by F1's ladder; their plan-budget gates were not rerun.

**The 2 → 6 rows under the spine remainder mixture.** #81 draws the remainder
invariants between spine rungs from an even flat/1/ŝ mixture. T-F2's map-key
census counts 48 of 579 `uux_to_ccx_emmm_qcd0` channels and 48 of 615
`bbx_to_ccx_emmm_qcd0` channels changed by it. #84's bbx ladder (sd ratio
0.708, χ²/dof 0.66 at 1.2M on the split + floor arm) was measured on the maps
before that change, and neither row was re-gated, since both are long tier.
On the merged tree (`0aed3835`), the long-tier MLM σ gates (whose rows the
mixture also moves) and the EWK e⁺e⁻jj gate all pass.

