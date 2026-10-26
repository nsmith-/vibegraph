---
type: Backlog Item
title: "validated_ew on the q q̄ → e⁺e⁻g rows waits on a re-recorded bundle"
description: "An adaptation-free estimate put uux/ddx_to_epemg on MadEvent's equal-weight value, so D1 sets validated_ew on them and on ee_to_mumu_tata_qcd0; the flag lives only in the banked records."
area: validation
state: needs-user
priority: medium
closes_when: "A bundle with validated_ew set on uux_to_epemg, ddx_to_epemg and ee_to_mumu_tata_qcd0 is pinned, and the fixed-beam σ gate's run on it is recorded with those rows reading their equal-weight references."
blocked_by: []
opened: 2026-10-17
tags: [sigma, madevent-reference, iteration-combination, refdata]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v2, resource: "../../sprints/sigma-pulls-lean-high/sessions/V2-report.md", title: "sigma-pulls-lean-high V2 report"}
  - {id: d1, resource: "../../sprints/sigma-pulls-lean-high/decisions/D1-fixed-beam-seeds-and-ew.md", title: "D1, point 2"}
---
[V2](../../sprints/sigma-pulls-lean-high/sessions/V2-report.md) integrated
`uux_to_epemg` and `ddx_to_epemg` with a fixed density over 20 × 3M points.
The pulls are +3.01 / +2.67 against MadEvent's reported mean and +0.45 / −0.19
against its equal-weight mean. That meets
[D1](../../sprints/sigma-pulls-lean-high/decisions/D1-fixed-beam-seeds-and-ew.md)'s
condition, so the user's decision sets `validated_ew` on the two rows and on
`ee_to_mumu_tata_qcd0`. The gate reads the flag only from
`seeded_sigma/<row>/seed_record.json`, so the change is a re-record, item 3 of
the banking request on PR #42. Expected pulls afterwards: +0.81, +0.00, +0.23.
`ee_to_mumu_tata_qcd0` is included on the family call, not on its own
estimate. Its record has two channels at seed χ²/dof 4.2 and 3.0 (corrected
p 0.045), and a fixed-density 2 → 4 check on it would cover 28 channels.

Progress 2026-10-22 (M3, relayed by the planner): `seed-record --validated-ew`
re-recorded the three rows into the local refdata-11 candidate. The gate
passes, but the pulls read −0.14, −0.10 and −0.56 (uux, ddx, ee_to_mumu_tata),
against the expected +0.81, +0.00 and +0.23. Explain the difference before the
bundle is pinned. The first suspect is which reference each expected pull was
read against.
