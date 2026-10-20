---
type: Backlog Item
title: The ℓℓj-family rows all read about +0.2% high against their seed records
description: pp_to_llj, llj_dyn, llj_fixed, uux/ddx_to_epemg and the MLM rows read +0.15–0.23% against MadEvent seed records with 0.07–0.22% errors, flat in budget and across scale choices.
area: validation
state: blocked
priority: medium
closes_when: The +0.2% is attributed (vibegraph physics, reference, or statistics) using the M3 uux/ddx_to_epemg 20 + 20 seed ladder, and either fixed or recorded with its cause.
blocked_by: []
opened: 2026-10-20
tags: [sigma-gate, llj, drell-yan, seed-sweep, refdata]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
Waits on the user's M3 banking batch: the 20 + 20 seed ladder on
`uux_to_epemg`/`ddx_to_epemg` requested in the comment on PR #42.

Readings on tree `340683c` against the refdata-10 seed records:
`pp_to_llj` +0.20%, `pp_to_llj_dyn` +0.21% (80 seeds), `pp_to_llj_fixed`
+0.23% (80), `uux_to_epemg` +0.19% / +0.23% (1× / 4×), `ddx_to_epemg`
+0.21% / +0.15%, `pp_to_llj_mlm` +0.23%, `pp_to_llj_xqcut_only` +0.19%.
The ladders are flat, so it is not budget, and it appears under both fixed
and dynamical scales. Read the M3 ladder against this list.

Detail: [V1 report, Found 2](../../sprints/sigma-calibration-rerecord/sessions/V1-report.md).
