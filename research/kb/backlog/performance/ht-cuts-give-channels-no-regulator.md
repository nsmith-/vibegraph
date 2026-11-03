---
type: Backlog Item
title: "An HT cut gives the channel maps no regulator"
description: "spacelike_floor reads single-leg pT minima only, so an HT-only card builds its peripheral channels unregulated; its p p > b b~ card scatters at χ²/dof 3.76 where the pT twin reads 0.62."
area: performance
state: open
priority: medium
closes_when: "The HT minima imply a bound or scale for the channel maps (each pT ≥ htmin/2 for two final partons, a scale beyond), and a σ ladder on the HT-only cards shows the scatter at χ²/dof ≈ 1."
blocked_by: []
opened: 2026-10-26
tags: [phase-space, ht-cuts, regulator, variance, maps-session]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f1, resource: "../../sprints/ht-cuts/sessions/F1-report.md", title: "ht-cuts F1 report, Found 1"}
  - {id: v1, resource: "../../sprints/ht-cuts/sessions/V1-report.md", title: "ht-cuts V1 report, Found 3"}
---
`Cuts::spacelike_floor` (`vibegraph-lib/src/cuts.rs`) is the largest single-leg
`pT_min` squared, and zero without one. An HT-only card therefore leaves its
peripheral channels with no spacelike floor.

Measured by [F1](../../sprints/ht-cuts/sessions/F1-report.md), five seeds at
300000 × 10 on the same phase space:

| card | σ (pb) | χ²/dof |
|---|---|---|
| `p p > b b~`, `ihtmin 200`, `ihtmax 500` | 4850.96 ± 3.55 (spread) | 3.76 |
| `p p > b b~`, `ptb 100`, `ptbmax 250` | 4847.59 ± 0.94 (spread) | 0.62 |

The HT arm's quoted error is about half its spread. On `p p > j j j` with
`htjmin 400`, 12 of 15 channels accept under a quarter of their points. The
banked gate passes at 75000 × 10 (χ²/dof 1.66, one seed 0.5 % low,
[V1](../../sprints/ht-cuts/sessions/V1-report.md)).

The channel maps belong to the maps design session; this is its input, not a
change for a feature session. With two final partons an HT window is a pT
window, so a bound is provable there; beyond two it can only be a scale. Compare
[spacelike-floor-too-loose](spacelike-floor-too-loose.md), the same floor's
looseness when a pT cut does exist.
