---
type: Backlog Item
title: The bbx ladder's excess comes from tails in channels sized by their α share
description: "On bbx_to_ccx_emmm_qcd0 the per-seed sd does not fall with budget (1.22 from 600k to 1.2M); floor-held channels carry ~2% of the variance, and channel 613's tail point does not shrink."
area: validation
state: open
priority: high
closes_when: "A per-channel tail study of 613, 614 and 590 says why their quoted variance does not fall as points double, and either a sampler change makes the bbx ladder's per-seed sd fall about 1/√2 from 600k to 1.2M over 20 paired seeds, or a stated heavy-tail error model replaces the √n reading for this row."
blocked_by: []
opened: 2026-10-28
tags: [two-to-six, multichannel, heavy-tail, budget-ladder, vegas]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
sources:
  - {id: s1, resource: "../../sprints/scaling-floor/sessions/S1-report.md", title: "scaling-floor S1 report, ladder, falsifier and Found 1–2"}
  - {id: p1, resource: "../../sprints/floor-bound-neyman/sessions/P1-report.md", title: "floor-bound-neyman P1 report, Neyman ruled out"}
---
Neither fix aimed at the floor-bound channels changed the ladder. Neyman
moved no points above the floors (#75), and a floor scaling with `neval`
doubled every floor-held channel's points but left the spread where it was:
sd ratio 1.245 against 1.220, χ²/dof 1.70 against 1.43, for 1.48× the points
(S1, 20 paired seeds). At 1.2M the ~570 floor-held channels quote 5.7e-10 pb
of error and the ~46 α-share channels 3.6e-9.

- Seed 55 draws a 1.18e-8 pb point in channel 613, an α-share channel whose
  variance does not fall over the run when points double (×1.00 overall,
  ×20 517 on seed 55). Without seed 55 the sd ratio is 0.905.
- **Read the falsifier distributionally.** "Variance falls at every seed"
  fails on heavy-tailed terms even when points double (channel 551 under the
  scaled floor: geometric-mean ratio ×0.39, but up on 6 of 20 seeds). Use the
  geometric-mean ratio against 1/2, with its spread.
- Rerun the control on the tree being measured: the α survey drifts at the
  last digit between trees, so a trace from another commit does not replay.

Blocks [two-to-six-integrals-info-on-scattering-references](two-to-six-integrals-info-on-scattering-references.md).
