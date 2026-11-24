---
type: Backlog Item
title: "DarkPhoton card 78 (MZd 40): some seeds read about 2% low with sub-percent errors"
description: "Card 78 at 4x budget is bimodal: three seeds at 1.000–1.007 of MG and two at 0.978 with 0.44–0.55% errors (χ²/dof 6.98). At 1x, 17 of 20 seeds read low and one tail seed lifts the mean."
area: validation
state: open
priority: medium
closes_when: "Card 78 at 4x budget over 20 seeds has χ²/dof near 1 and a mean within 1σ of MG, either after a sampler or channel fix or with the missed region identified and shown benign. The low seeds' channel shares are compared against the high seeds' to name the region."
blocked_by: []
opened: 2026-11-24
tags: [darkphoton, vegas, multichannel, sampling, confidently-wrong, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
---
Found by the sprint L validator on PR 11's head 8a651703, with the campaign
argv, against gpdb MG 0.2167 ± 0.0005 pb. The per-seed rows are in the PR 11
gate addendum of the [L-X1 report](../../sprints/v023-orders-majorana-timeouts/sessions/L-X1-report.md).

| rung | seeds | vg/MG | sd | χ²/dof | shape |
|---|---|---|---|---|---|
| 1x (neval 120000) | 20 | 0.9926 ± 0.0045 | 1.75% | 1.27 | 17/20 below MG. The median seed is about 0.990. One seed reads 1.052 with ESS 696. |
| 4x (neval 480000) | 5 | 0.9942 ± 0.0069 | 1.47% | 6.98 | Seeds 1, 2 and 4 read 1.007, 1.007 and 1.001. Seeds 3 and 5 read 0.979 ± 0.55% and 0.978 ± 0.44%, with ESS 30k and 45k. |

The mean moves toward 1 with budget, so this is not a stable deficit.

The shape is the "confidently wrong σ" signature in AGENTS.md: a region usually
under-sampled and occasionally hit. At 4x, the low seeds report the smallest
errors and the largest ESS. Any 1/σ² combination would favour them.

The other DarkPhoton cards (79, 418, 419) read 0.991–0.998 at 5 seeds. Only
MZd 40 is affected.

Run 20 seeds at 4x first (about 2.4 h). Then dump the per-channel σ shares for a
low seed against a high one.
