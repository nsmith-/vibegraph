---
type: Backlog Item
title: "The spacelike floor's largest-threshold rule inflates seed spread next to a softer jet"
description: "spacelike_floor caps every peripheral rung at the largest single-leg pT_min²; on p p > a j j at pta 40, σ still matches MadGraph, but per-seed errors are 2–3× larger and heavy-tailed."
area: performance
state: open
priority: medium
closes_when: "Each peripheral rung's support bound comes from the thresholds of the legs it can reach (or is the smallest one), and on the pta 40, ptj 0.1 and ptj 1 cards at fixed neval the 5-seed sd is no worse than the smallest-threshold build's, with σ unchanged against MadGraph."
blocked_by: []
opened: 2026-11-07
tags: [phase-space, cuts, t-channel, variance, multichannel]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: hb1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-B1-report.md", title: "H-B1 report, Found 2"}
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log: the CERN host's CKM agent"}
---
`Cuts::spacelike_floor` (`vibegraph-lib/src/cuts.rs`) takes the largest
single-leg `pT_min` over the final legs and uses its square as each peripheral
rung's support bound, `t ≤ −floor`. On `p p > a j j` with `pta = 40` and a soft
jet threshold, that closes a rung next to the jet far above where the jet's own
cut allows. The estimator stays unbiased while the other channels cover that
region, but they cover it thinly. The likely result is the heavy-tailed seeds
measured below. That mechanism is inferred; no per-channel weight split has
been measured.

**The reference.** Measured by the CERN host on main d87ad3ae, 2026-11-08.
- Card: GJ-2Jets-EWK_Bin-PTG-40's own proc, param and run card, with lhaid
  315200, `draj 0.4`, and `ptj 0.1` or `1`.
- MadGraph 3.7.1: two iseeds per card.
- vibegraph: 5 seeds per row, at niter 3 with a fixed budget. σ/MG is the
  unweighted mean with its sem.

| Card | MadGraph (pb) | Stock, neval 2e5 | Smallest threshold, 2e5 | Stock, 2e4 | Smallest threshold, 2e4 |
|---|---|---|---|---|---|
| ptj 0.1 | 54.27 ± 0.11 | 1.000 ± 0.006 (sd 0.73, χ²/dof 2.05) | 0.995 ± 0.003 (sd 0.34) | 0.995 ± 0.024 (sd 2.90, χ²/dof 3.9) | 1.000 ± 0.011 (sd 1.31) |
| ptj 1 | 53.48 ± 0.12 | 0.992 ± 0.006 (sd 0.76) | 0.995 ± 0.002 (sd 0.22) | 0.973 ± 0.007 (χ²/dof 0.45) | 0.992 ± 0.007 |

- **σ is correct at convergence.** The +6% sprint H attributed to removed
  phase space came from two low stock seeds at neval 2e4.
- **The cost is variance.** At equal budget, stock's per-seed errors are 2–3×
  the smallest-threshold build's, about 4–9× the evaluations for equal
  precision.
- **At low budget it can be confidently low.** At `ptj 1` and neval 2e4,
  stock reads 2.7% low (3.7 sem, with χ²/dof 0.45). The CMS campaign runs at
  neval 1e4.
- **A residual offset is unexplained.** Both builds sit about 0.5% below
  MadGraph at neval 2e5 (1.6–2.5 sem). Five seeds cannot calibrate it; read it
  against a 20-seed spread before treating it as an offset.

A σ test cannot fail on the largest-threshold rule, so the gate is the seed
spread at a fixed neval.
[spacelike-floor-too-loose](spacelike-floor-too-loose.md) wants the floor
tighter where every adjacent leg is hard. This item wants it looser next to a
soft leg. A per-rung bound serves both.
