---
type: Backlog Item
title: "A --target-rel run costs at least six iterations of n_channels × 512 points, whatever --neval"
description: "With --min-iters 6 and a 512-point per-channel floor, 1068-channel MLM cards need 1.8–3.3× the 1200 s campaign cap before the stop rule can fire, and cards already below target keep iterating."
area: performance
state: open
priority: high
closes_when: "integrate reports, after warm-up 1, the minimum wall time its stop rule implies (channels × floor × min-iters × measured ns/eval) as a warning. Then either the warm-up and floor schedule for wide splits is changed, with σ within the multi-seed criterion on the MLM σ rows and a 1068-channel GJ or TT card, and cost × variance recorded over ≥10 seeds, or the schedule is kept and the measurement recording why is in a performance concept."
blocked_by: []
opened: 2026-11-23
tags: [vegas, sampling, mlm, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
`--min-iters` defaults to 6 (two warm-ups), and the first iteration
spends `Σ max(shareⱼ, 512)`. On 1068-channel MLM cards warm-up 1 is ~610k
points against `--neval 120000`; later iterations are re-floored, and
ns/eval rises 1.2–4× after warm-up 1 as more points pass the cuts (744:
0.35 → 1.76 ms).

Lower bounds on the wall needed, from the measured rates: 32 2454 s, 25
3957 s, 42 3407 s, 46 2419 s, 992 2480 s, 195 3087 s, 590 2150 s. Cards
196 and 744 were already below the 1% target (0.39%, 0.43%) and kept
running only for `--min-iters`.

Per-point cost is the other factor:
[`mlm-every-group-matrix-element-evaluated-per-point`](../performance/mlm-every-group-matrix-element-evaluated-per-point.md),
[`mlm-matrix-element-evaluated-twice-per-group`](../performance/mlm-matrix-element-evaluated-twice-per-group.md). Related:
[`niter-3-campaigns-have-no-chi2-check`](../validation/niter-3-campaigns-have-no-chi2-check.md). Evidence:
[L-T2 report](../../sprints/v023-orders-majorana-timeouts/sessions/L-T2-report.md), [phase sample](../../sprints/v023-orders-majorana-timeouts/v022-1009-phase-sample.md).
