---
type: Backlog Item
title: "bbx_to_ccx_emmm_qcd0 unweighting acceptance measures 2.8e-3, eight times below the 2.2e-2 its manifest note records"
description: "PR 13's 2→6 pricing probe measured 353 trials per event against the 45 recorded on an earlier tip; the maxima and truncation code did not change. It threatens the 400-trial samples budget."
area: validation
state: open
priority: medium
closes_when: "The probe is rerun printing σ/Σw_max and the per-channel peak-to-w_max ratios on both tips (or on the tip that recorded 2.244e-2), the cause is named (grids, the scan, or the earlier measurement), and the 2→6 samples cell's trial budget and its beam pz tolerance are confirmed or re-set."
blocked_by: []
opened: 2026-11-22
tags: [two-to-six, unweighting, samples-gate, acceptance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
sources:
  - {id: ls1, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-S1-report.md", title: "L-S1 report: Found 2"}
---
Sprint L's PR 13 measured the acceptance with `probe_2to6_sample_cost` at
-j 4 on 55b32dbd. The blocked pass doesn't set the maxima, and the unweighting
gate's realised efficiencies match their predictions, e.g. 0.432 against 0.438.
So the drop comes from upstream of accept/reject. Related:
[`bbx-drifts-from-madevent-with-budget`](bbx-drifts-from-madevent-with-budget.md).

**The row's margins are thin** (vg-validator, on the samples pass at
5b546464). Trials per event were 355–363, 89–91% of the 400 budget. The worst
beam-2 `pz` deviation was 4.43e-9 against the 5.00e-9 tolerance, 89% of it.
The row passes, but a slightly different maximum or seed could tip either.
Whatever this item finds about the acceptance should also re-set or justify
both margins.
