---
type: Backlog Item
title: The reweighting cost comparison against MadGraph is one host, one commit, order of magnitude
description: "The 50–150× (polynomial up to 320×) reweighting figures are from f6b1936 on one cloud VM; re-measure at the shared re-measurement commit on the same hosts, with more hypotheses per row."
area: performance
state: open
priority: medium
closes_when: "reweighting-cost-vs-madgraph.md carries a re-run of bench-reweight at the commit and on the hosts of the MATRIX1 re-measurement (an x86 host and the M3), including one row at ≥50 hypotheses so the setup + H × loop estimate is checked against a measured large-H run on our side."
blocked_by: []
opened: 2026-11-04
tags: [performance, madgraph-comparison, reweighting, re-measure, presentation, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: rw, resource: "../../performance/reweighting-cost-vs-madgraph.md", title: "Reweighting cost against MadGraph's reweight module"}
---
[Reweighting cost against MadGraph](../../performance/reweighting-cost-vs-madgraph.md)
was measured at `f6b1936` on one 4-vCPU VM and is labelled an order-of-magnitude
comparison. It backs the "up to two orders of magnitude for workflows with many
weights" claim, so it needs the same treatment as the other comparisons:

- the commit and hosts of [the MATRIX1 re-measurement](matrix-element-vs-madgraph-stale.md);
- a row with many hypotheses (≥50, the regime EFT scans use), since every row
  today has H ≤ 11 and MadGraph's side is extrapolated as setup + H × loop
  (its multi-launch path mislabels a weight in 3.7.1, see
  [MadGraph defects](../../validation/madgraph-defects.md));
- both sides at `change helicity False`, recorded, since vibegraph has only the
  helicity-summed ratio ([reweight-helicity-summed-only](../feature/reweight-helicity-summed-only.md)).
