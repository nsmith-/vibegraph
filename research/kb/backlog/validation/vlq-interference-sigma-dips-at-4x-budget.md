---
type: Backlog Item
title: "VLQ interference σ on 612/614 dips at 4× budget and returns at 8×; few-seed rung spreads understate the scatter"
description: "On main, 612 reads 1×/4×/8× = −8.89/−8.62/−8.79e-4 and 614 −3.11/−2.84/−3.02e-4 (5 seeds each). The 4× rows have χ²/dof 0.44 and 0.12, and 4×→8× moves 2.3–2.5σ on their spreads."
area: validation
state: open
priority: medium
closes_when: "20 seeds each at 4× and 8× on 614 (and 612 if 614 shows a dip) give rung means whose difference is read against the measured 20-seed spreads. Either the dip is within that spread, or it is real and the channel or VEGAS mechanism that undersamples the cancelling interference at 4× is named."
blocked_by: []
opened: 2026-11-25
tags: [vegas, interference, budget-convergence, vlq, seed-spread]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
sources:
  - {id: l-i1, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-I1-report.md", title: "L-I1 report: VLQ interference sign"}
---
Sprint L's validator ran main e6a4cba8 at campaign settings with only neval
changed (120000 / 480000 / 960000), 5 seeds per rung, signed σ in pb:

| card | 1× | 4× | 8× | MG (scheme off) |
|---|---|---|---|---|
| 612 | −8.886e-4 (sd 6.1%, χ² 1.98) | −8.618e-4 (sd 1.5%, χ² 0.44) | −8.786e-4 (sd 0.8%, χ² 0.28) | −8.912e-4 (3 seeds) |
| 614 | −3.114e-4 (sd 23.2%, χ² 2.38) | −2.842e-4 (sd 3.2%, χ² 0.12) | −3.024e-4 (sd 4.8%, χ² 0.73) | −3.078e-4 (8 seeds) |

On both cards 4× is the low point, and 8× comes back to 1× and to MG. The 4× →
8× step is about 2.3σ (612) and 2.5σ (614) on the 5-seed spreads. A χ²/dof far
below 1 means the per-seed errors overstate the scatter. Five seeds cannot
calibrate a rung-to-rung difference, and few-seed spreads have understated a
heavy-tailed row's true spread by 2–5× before (AGENTS.md).

σ is the real part of an interference that cancels across a 20%-width T' pole.
A budget-dependent undersampling of one sign's region would read exactly
like this. The 20-seed ladder tells that apart from scatter. It costs about
35 min locally.
