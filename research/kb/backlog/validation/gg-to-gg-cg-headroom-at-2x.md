---
type: Backlog Item
title: gg_to_gg_cg sits at exactly 2.0× on rel_tol
description: The worst of five gate seeds reads 2.55e-3 against rel_tol 0.005 (scatter 0.12% a seed); with gux_to_epemux (2.1×) and ll_to_qqx_toy_tensor (2.2×) it is the thinnest σ cell.
area: validation
state: open
priority: low
closes_when: A 20-seed run at the gate budget measures the row's spread and worst-of-five distribution, and the result either shows the 2.0× is a draw or carries a recorded decision on the margin.
blocked_by: []
opened: 2026-10-20
tags: [sigma-gate, headroom, seed-sweep, gate-budget]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
On tree `340683c`, five gate seeds put `gg_to_gg_cg`'s worst `|rel|` at
2.55e-3 against `rel_tol` 0.005. The user kept every gate budget on
2026-10-20, after 4× the points bought `gux_to_epemux` no margin. A
budget increase is not the default fix here either.

Detail: [V1 report, Found 4](../../sprints/sigma-calibration-rerecord/sessions/V1-report.md); [the 2026-10 census](../../validation/seed-headroom-census-2026-10.md).
