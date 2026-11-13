---
type: Backlog Item
title: The clustering rebuilds a momentum-independent PDG table per group per point
description: "walk_tree rebuilds a 2ⁿ-mask PDG table from MergeTable.ipdgcl BTreeMap lookups per flavour group per point; it depends only on channel and graph, and costs 10.7% of the MLM VEGAS phase."
area: performance
state: open
priority: low
closes_when: "The table is built once per (group, configuration) and reused, with clustering output bit-identical on the MLM dump oracles, and the share re-profiled on pp_to_ll_0j2j_mlm."
blocked_by: []
opened: 2026-11-10
tags: [mlm, performance, clustering, caching]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: diag, resource: "../../performance/mlm-integration-cost-vs-madevent.md", title: "MLM integration cost against MadEvent"}
---
In the [MLM profile](../../performance/mlm-integration-cost-vs-madevent.md), 10.7% of `pp_to_ll_0j2j_mlm`'s VEGAS-phase samples are
BTreeMap lookups in `walk_tree`, which rebuilds a 64-entry (at @2) PDG table per
group per point. Estimated gain from caching it: @2 cost ×0.89 (inferred). The
change is order-preserving, so the kT-cluster and MLM dump oracles must stay
bit-identical.
