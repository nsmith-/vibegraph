---
type: Backlog Item
title: "14 cards newly able to run exceed the 24 h batch cap at the campaign budget, so their σ is unmeasured"
description: "v0.2.3 stage B removed cards 15, 16, 136, 137, 202–204, 297, 299 and 724–728 at the 24 h wall cap, with no panic, OOM or refusal; which phase each was in is not known."
area: validation
state: open
priority: medium
closes_when: "Each of the 14 cards has a timed local or batch run that names the phase it spends its time in (enumerate, survey, refine, unweight), and either reads a σ against MadGraph or is attributed to a filed cost item."
blocked_by: []
opened: 2026-11-28
tags: [timeouts, batch, cms, vbs, eft, budget]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-28}
---
Found by `vg-validator`'s v0.2.3 pre-tag stage B: 1-seed integrations of 105
cards that v0.2.2 could not run, on the dev build of new 40e87aa8. 85 completed,
6 were refused (MadGraph's σ is 0 there) and 14 were removed by
`SYSTEM_PERIODIC_REMOVE` at the 24 h wall cap (86497–86499 s for 724–728).

The batch proxy keeps no partial log for a removed job, so the phase each
job was in when it was cut is unknown. Since v0.2.3 the run's progress
records name the phase, so a rerun that streams them does name it.

Cards 724–728 are VBS WZ EFT-interference cards with a signed MadGraph σ:

| card | process | MadGraph σ (pb) |
|---|---|---|
| 724 | VBS_WZ_M4Int_Zmumu_Wtavt | −6.29e-7 ± 6.5e-8 |
| 725 | VBS_WZ_M2Int_Zmumu_Weve | −4.82e-7 ± 3.0e-8 |
| 726 | VBS_WZ_M4Int_Zee_Weve | −6.62e-7 ± 1.2e-7 |
| 727 | VBS_WZ_M2Int_Zee_Wtavt | −4.60e-7 ± 5.8e-8 |
| 728 | VBS_WZ_T6Int_Ztata_Wmuvm | +1.056e-5 ± 3.7e-8 |

A signed σ near zero with a relative error target may need far more points
than a positive one; whether the run was converging slowly or stuck in a phase
is the first thing to measure. A timed local probe of one card is the proposed
first step.
