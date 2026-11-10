---
type: Backlog Item
title: MLM evaluates the matrix element twice per group per point
description: "Each flavour group calls eval_amp2 for the configuration draw and eval_m2 after moving αs, at the same momenta; one pass rescaled by the αs ratio would save about 14% of a two-jet point."
area: performance
state: open
priority: low
closes_when: "One matrix-element pass per group per point serves both the configuration draw and the weight, with the αs change applied as a rescaling that handles vetoed_m2 correctly, and the MLM σ and dump gates unchanged within tolerance."
blocked_by: []
opened: 2026-11-10
tags: [mlm, performance, matrix-element, alpha-s]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: diag, resource: "../../performance/mlm-integration-cost-vs-madevent.md", title: "MLM integration cost against MadEvent"}
---
`eval_amp2` (17.6% of the MLM VEGAS phase) and `eval_m2` (14.4%) run at the same
momenta for each group ([profile](../../performance/mlm-integration-cost-vs-madevent.md)). At fixed coupling-order structure the αs
move is an overall factor per coupling-order class, so a rescaling of the first
pass may replace the second. Check whether every term scales the same way
(mixed QCD/QED orders do not) and what `vetoed_m2` needs. Estimated gain: @2 cost
×0.86 (inferred).
