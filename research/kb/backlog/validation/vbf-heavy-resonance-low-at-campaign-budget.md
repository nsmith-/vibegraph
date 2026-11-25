---
type: Backlog Item
title: "VBF heavy-resonance cards 665 and 653 read 3–5% low at the campaign's fixed budget on both builds"
description: "At neval 120000 × 3, five seeds put card 665 (VBF Z'→WW, M1000) at 0.957–0.972 of MG and 653 (VBF W'→WH, M7000) at about 0.95, before and after PR 5; the released 1% proxy runs read about 1.00."
area: validation
state: open
priority: medium
closes_when: "A budget ladder (1x, 4x, and the 1% target) on 665 and 653, 5+ seeds per rung read against the measured seed spread, shows whether the fixed-budget deficit closes with budget. If it doesn't close, the missed region is named. If it does, the campaign budget for these cards is raised or their rows carry the proxy target."
blocked_by: []
opened: 2026-11-24
tags: [vbf, sigma, budget-convergence, cms, campaign]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
---
The sprint L validator ran these on PR 5's set B (#184, 2091a26f) and on main
7b36722c, from before PR 5. Both used the campaign argv: `--fixed-budget --neval
120000 --niter 3 --parallel-diagrams`, with 5 seeds per build.

| card | MG | PR 5 vg/MG | main vg/MG | per-seed (PR 5) | per-seed (main) |
|---|---|---|---|---|---|
| 665 (VBF Z'→WW, M1000) | 54.27 ± 0.16 | 0.9570 ± 0.0127 | 0.9719 ± 0.0110 | 0.915, 0.974, 0.979, 0.975, 0.941 | 1.008, 0.952, 0.966, 0.982, 0.952 |
| 653 (VBF W'→WH, M7000) | 2.78e-5 | 0.9843 ± 0.0406 | 0.9490 ± 0.0122 | 0.947, 1.142, 0.941, 0.922, 0.970 | 0.990, 0.935, 0.940, 0.930, 0.950 |

For 653 on PR 5, seed 2 is a tail seed (±12.5%), and χ²/dof is 7.96. Without it
the mean is about 0.945.

The two builds agree: on 665, PR 5 − main is −0.015 ± 0.017. So this is not a
PR 5 effect.

The released campaign read about 1.00 on both cards, but its `vg_run` rows were
`proxy:target_rel=0.01` with `fixed_budget=0`, so they are not comparable.

Five mutually consistent seeds reading low together is the budget-convergence
axis in AGENTS.md, not a seed-scatter question. The ladder is the test.
