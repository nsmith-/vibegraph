---
type: Backlog Item
title: "4-jet and γ + jets cards read 5–15% low at the campaign budget, with errors too small, on seeds that miss a region"
description: "Card 252 reads 0.907 ± 0.016 over 5 seeds at 1×, against 1.012 ± 0.027 at 4×. Every small-error seed is low and every seed at or above 1 has a large error. 993 and 232 scatter the same way."
area: validation
state: open
priority: medium
closes_when: "The region these seeds miss is localised (channel, cut boundary or phase-space map). Sampling is fixed so that card 252 reads within 2σ of MadGraph at the campaign budget over 20 seeds with seed χ²/dof near 1, or the campaign budget for this family is raised with a 20-seed ladder showing the chosen rung converged."
blocked_by: []
opened: 2026-11-27
tags: [vegas, multichannel, confidently-wrong, multi-jet, photon, cms, budget]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-27}
---
Found by `vg-validator`'s v0.2.3 pre-tag stage B and its follow-ups on the fix
build 2431b587. None of these cards ran in v0.2.2.

| card | rung | seeds | unweighted mean vg/MG | spread |
|---|---|---|---|---|
| 252 GJets-4Jets dRGJ-0p2 HT40-70 | 1× | 4, plus 1 refused | 0.907 ± 0.016 | 0.032 |
| 252 | 4× | 5 | 1.012 ± 0.027 | 0.061 |
| 993 GJets_DR-0p4 HT-100to200 | 1× | 5 | 0.958 ± 0.060 | 0.135 |
| 232 GJ-4Jets HT-600to1000 PTG-100to200 | 1× | partial | 0.65–0.84, one refused | |

On stage B's one seed each, about 20 independent 4-jet rows (GJ-4Jets,
GJets-4Jets, WBtoLNuB-4Jets, DYBto2LB-4Jets, GJets_HT/DR) averaged about 0.93.

On 252 at 4×, the two seeds with a small quoted error read 0.951 (1.2%) and
0.961 (1.6%), and the seeds that read above 1 carry 7–11% errors. That is the
missed-region signature.

Five seeds per rung cannot calibrate the 0.907 → 1.012 shift on a row this
heavy-tailed, so 20 seeds per rung are needed. These cards use
`sde_strategy = 1`, so the MadGraph channel-weight difference
(`scales-pdf/channel-cut-madgraph-versions`) does not apply.
