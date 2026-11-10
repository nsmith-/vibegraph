---
type: Backlog Item
title: MLM evaluates flavour-copy groups separately at every point
description: "pp_to_ll_0j2j_mlm's two-jet part clusters and evaluates all 64 flavour groups per point, though u≡c and d≡s copies give 28 classes; sharing them is the largest MLM lever (@2 cost about ×0.54)."
area: performance
state: open
priority: medium
closes_when: "Groups that differ only by a flavour copy with identical masses and couplings share their matrix-element and clustering work per point, with weights unchanged within the MLM gates' tolerances, and pp_to_ll_0j2j_mlm's @2 cost per point and time to accuracy against MadEvent re-measured."
blocked_by: []
opened: 2026-11-10
tags: [mlm, performance, flavour-groups, madgraph-comparison]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: diag, resource: "../../performance/mlm-integration-cost-vs-madevent.md", title: "MLM integration cost against MadEvent"}
---
`per_group_sum` handles every flavour group of a merged part at every point: 64 at
@2 of `pp_to_ll_0j2j_mlm`, 12 at @1. In the default SM (massless u, d, s, c), u≡c
and d≡s copies have the same matrix element and clustering at the same momenta;
the compiled process list gives 28 classes at @2 and 6 at @1.

Per the [diagnosis](../../performance/mlm-integration-cost-vs-madevent.md), the @2 point costs 485 µs against MadEvent's 104 µs and is
the whole of the row's 2.4× gap; sharing work across copies is estimated (from the
profile, not implemented) at @2 cost ×0.54, about 1.6× on the ratio. The class
criterion must be exact (masses, widths, couplings, PDF flavour enters only the
luminosity), and the per-group configuration draw must keep its distribution.
