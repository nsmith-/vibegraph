---
type: Backlog Item
title: The integration comparison against MadEvent is stale and covers no MLM or 2→4 row
description: "Time to 0.1% on σ and throughput against MadEvent predate the evaluator and map changes since cf8b2b7; re-measure with the MATRIX1 re-run, adding an MLM and a 2→4 row."
area: performance
state: open
priority: medium
closes_when: "integration-vs-madgraph.md carries fresh time-to-0.1% and throughput tables at the v0.2.0 tag on the hosts of the MATRIX1 re-measurement (an x86 host and the M3 at least), over the five existing time-to-accuracy rows plus at least one MLM-matched multi-jet row and one 2→4 row, each with ≥3 seeds, MadGraph's version and flags recorded, and a wall-clock column at equal core count beside the cumulated_time one; README.md's time-to-accuracy and throughput sections quote the new tables."
blocked_by: []
opened: 2026-11-04
tags: [performance, madgraph-comparison, integration, time-to-accuracy, throughput, re-measure, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: int, resource: "../../performance/integration-vs-madgraph.md", title: "Integration against MadGraph: throughput and time to accuracy"}
  - {id: protocol, resource: "../../performance/end-to-end-timing-protocol.md", title: "End-to-end timing protocol"}
---
The tables in [integration against MadGraph](../../performance/integration-vs-madgraph.md)
(time to 0.1% on σ: 1.67× Cascade Lake, 3.84× M3 Max; throughput 3.97× / 8.76×)
predate constant collection and fused scaled sums, and the map work since
(the windowed decay angle halved `pp_to_llj`'s evaluations to 0.1%), and none
has been re-measured since.

Needs, beyond a re-run:
- **The same commit and hosts as
  [the MATRIX1 re-measurement](matrix-element-vs-madgraph-stale.md)**, so the
  per-point and end-to-end ratios can be quoted side by side.
- **Representative rows.** The five time-to-accuracy rows have no MLM-matched
  sample and nothing above 2→3. Add one MLM row (e.g. `p p > l+ l-` + 0–2 jets
  at the mlm gate's settings) and one 2→4 row; a user's workload looks like
  these, not like `ee_to_mumu`.
- **A wall-clock column.** `<cumulated_time>` carries a host-dependent O(1)
  factor ([madevent-cumulated-time-host-dependence](madevent-cumulated-time-host-dependence.md));
  both codes at equal core count, wall clock to the same δ, is the number a
  user experiences and is immune to it.
- `pp_to_llj` re-measured here closes step 1 of
  [llj-map-lower-edge-on-cut-edge](llj-map-lower-edge-on-cut-edge.md).

**When (user, 2026-11-05).** Run after v0.2.0 is cut, at that tag, together with
the other three comparisons, then update `README.md` from the results.
