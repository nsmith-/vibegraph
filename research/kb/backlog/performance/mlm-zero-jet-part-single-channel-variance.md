---
type: Backlog Item
title: The zero-jet part of pp_to_ll_0j2j_mlm has 1.4× MadEvent's per-point spread
description: "@0's S is 805 against MadEvent's 512 under every map option tried; MadEvent splits it into a γ and a Z channel where vibegraph shares one map across 4 (group, diagram) channels."
area: performance
state: open
priority: low
closes_when: "@0's per-point spread on pp_to_ll_0j2j_mlm is within 1.1× MadEvent's, or the cause is recorded with the reason it is not worth changing."
blocked_by: []
opened: 2026-11-10
tags: [mlm, performance, phase-space-map, drell-yan]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: diag, resource: "../../performance/mlm-integration-cost-vs-madevent.md", title: "MLM integration cost against MadEvent"}
---
Unchanged under `--map-tau breit-wigner` and the split-angle options
([A/B](../../performance/mlm-integration-cost-vs-madevent.md)). MadEvent integrates @0 as a γ channel (G1, 31 pb) and a Z channel
(G2, 632 pb); vibegraph reports "4 (group, diagram) channels share 1 map". Cheap
today, since an @0 point costs a quarter of MadEvent's, so it matters once @2's
cost is cut. Also check plain `dy13_default`, which has the same final state.
