---
type: Backlog Item
title: The MLM budget split ignores each part's cost per point
description: "Parts are sized by αsσ, not by S/√c; @2 points cost 38× @0's, so the cost-aware optimum gives @2 about 10% of points instead of 24–26%. Opposite in direction to the @2 tail item."
area: performance
state: open
priority: low
closes_when: "The part split uses each part's measured per-point cost as well as its spread, reconciled with mlm-at2-tail-unpriced-in-allocation, and pp_to_ll_0j2j_mlm's cost × variance at fixed seed count improves by more than the seed scatter (≥10 seeds)."
blocked_by: []
opened: 2026-11-10
tags: [mlm, performance, allocation, vegas]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: diag, resource: "../../performance/mlm-integration-cost-vs-madevent.md", title: "MLM integration cost against MadEvent"}
---
Minimising cost × variance gives n ∝ S/√c per part. On `pp_to_ll_0j2j_mlm` the
current split is 1.40–1.46 above that optimum ([diagnosis](../../performance/mlm-integration-cost-vs-madevent.md)); closing it would
gain up to 1.4×, or about 1.2× if @2's true sd is 1.4× its quoted, as the σ gate's
fixed-budget seeds suggest. [mlm-at2-tail-unpriced-in-allocation](mlm-at2-tail-unpriced-in-allocation.md)
argues @2 needs more points for its tail; this argues fewer for its cost. Both are
real; settle the net with one measurement. Fewer @2 points per iteration also feed
@2's grid adaptation less.
