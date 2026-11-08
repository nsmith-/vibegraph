---
type: Backlog Item
title: "The EWK e+ e- j j long-tier row was retired; bank a regulated variant"
description: "pp_to_epemjj_qcd0 (ptj = 0, etaj = -1) diverges at tree level and is now refused, so the peripheral, no-fiducial-scale EWK regime has no σ row; re-bank it with an etaj or ptj cut."
area: validation
state: open
priority: medium
closes_when: "A manifest row for u d > e+ e- u d QCD=0 (and its family) on a card with an etaj or small ptj cut is banked on the M3 with a seeded record, and its long-tier σ gate passes by the multi-seed criterion."
blocked_by: []
opened: 2026-11-08
tags: [validation, banked-rows, ewk, long-tier, ir-safety]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: d1, resource: "../../sprints/v02-last-refusals/decisions/D1-ewk-lljj-refused.md", title: "Sprint I D1"}
---
Sprint I's initial-collinear guard refuses `pp_to_epemjj_qcd0`: with
`ptj = 0` and `etaj = -1`, a quark leaving its beam through a t-channel photon
has an uncut equivalent-photon pole (|M|²·|t| constant over 8 decades of t), so
σ diverges logarithmically. The banked 0.36% agreement with MadGraph was two
samplers' pole floors agreeing ([D1](../../sprints/v02-last-refusals/decisions/D1-ewk-lljj-refused.md)).
The row, its long-tier gate (`validate_ewk_lljj_sigma`), its pixi task and its
bank-script group were removed. Its seed record stays in refdata-13, unused.

The regime it covered (VBF-like peripheral channels with no fiducial pT scale,
where the spacelike floor is the channel's own default) still wants a σ row:
bank the same process family on a card with a rapidity maximum (e.g.
`etaj = 5`) or a small `ptj`, which the guard accepts.
