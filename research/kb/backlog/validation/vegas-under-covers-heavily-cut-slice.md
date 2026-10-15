---
type: Backlog Item
title: VEGAS reads a heavily cut slice 3 % low at the default budget while reporting its target met
description: On u u~ > w+ b w- b~ / t t~ with both W's at |cos θ| > 0.99 (3 % acceptance), five default-budget seeds agree with each other and sit −3 % low; --neval 1e6 recovers the true value.
area: validation
state: open
priority: medium
closes_when: The per-channel floor or allocation that under-covers the cut region is identified, and the convergence stop detects it (or the default budget covers it), shown on this slice against the non-adaptive estimate.
blocked_by: []
opened: 2026-10-15
tags: [vegas, convergence, cuts, multichannel]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: v2, resource: "../../sprints/sigma-uux-wbwb-veto/sessions/V2-report.md", title: "sigma-uux-wbwb-veto V2, Found 3"}
---
A run whose seeds agree with each other but are collectively low is the
"confidently wrong" failure the
[seed sweeps](../../validation/seed-sweeps-and-budget-ladders.md) warn of, and
here it is on this side. With a probe cut keeping both W's at |cos θ| > 0.99,
five default seeds read −3 % against `--neval 1e6` and a dedicated
non-adaptive sampler, while each run reported its error target met. On the
full process the effect is about 0.09 %. The scratch
(`uux/v2/`, probe `VG_PROBE_FWD`) is described in the V2 report. The same
session family saw seed χ²/dof 3.9 on the both-tops-inside slice of the `$`
row.
