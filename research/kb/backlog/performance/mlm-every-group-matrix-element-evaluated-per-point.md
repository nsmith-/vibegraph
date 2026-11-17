---
type: Backlog Item
title: MLM evaluates every group's matrix element at every point, where MadEvent evaluates one
description: "A pp_to_ll_0j2j_mlm two-jet point makes 56 matrix-element evaluations (28 groups, two each); MadEvent clusters every term but evaluates one selected matrix element."
area: performance
state: open
priority: medium
closes_when: "A matched multi-jet point evaluates the matrix element of one (group, ordering) term drawn by its clustered luminosity weight, with the estimator unbiased, the MLM σ and dump gates passing, and pp_to_ll_0j2j_mlm's cost × variance against the per-group sum recorded over ≥10 seeds; or the per-group sum is kept and the measurement recording why is in the MLM cost concept."
blocked_by: []
opened: 2026-11-11
tags: [mlm, performance, matrix-element, pooling, madgraph-comparison]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-11}
sources:
  - {id: diag, resource: "../../performance/mlm-integration-cost-vs-madevent.md", title: "MLM integration cost against MadEvent"}
---
Pooling sums every flavour group at every point, and under the per-term scale path
each group draws a configuration from its own matrix element (`eval_amp2`) and then
evaluates it again after moving αs (`eval_m2`): 56 evaluations and 64 member
reweights per two-jet point, about 40% of its 485 µs ([diagnosis](../../performance/mlm-integration-cost-vs-madevent.md)).

MadEvent clusters every (configuration, subprocess, mirror) term in its luminosity
pass, about 33 per point, but then selects one term and evaluates its matrix element
once. The analogue here: compute every group's clustered scale and PDF weight, draw
one term by that weight, evaluate its matrix element. The draw adds the variance of a
discrete choice (MadEvent adapts it with its grouped-process grid) and gives up the
per-group expectation, so the net is cost × variance, not cost alone. The configuration
draw itself needs `AMP2`, which is the first matrix-element pass, so the saving
depends on drawing the configuration without it or reusing it.
[mlm-matrix-element-evaluated-twice-per-group](mlm-matrix-element-evaluated-twice-per-group.md)
is the smaller lever inside each term.
