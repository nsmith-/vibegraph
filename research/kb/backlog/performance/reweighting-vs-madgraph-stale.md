---
type: Backlog Item
title: The reweighting cost comparison against MadGraph is one host, one commit, order of magnitude
description: "The 50–150× (polynomial up to 320×) reweighting figures are from f6b1936 on one cloud VM; re-measure at the shared re-measurement commit on the same hosts, with an operator-basis row."
area: performance
state: open
priority: medium
closes_when: "reweighting-cost-vs-madgraph.md carries a re-run of bench-reweight on the x86 host at the v0.2.1 tag (the first release with the musl allocator fix) and on the M3 at v0.2.0 or later, including an operator-basis row: n SMEFT operators at NP=1, (n+1)(n+2)/2 hypotheses (n = 8 → 45), timed on both the exact and polynomial paths, with MadGraph measured at that H, not extrapolated, and README.md quotes the result with its host."
blocked_by: []
opened: 2026-11-04
tags: [performance, madgraph-comparison, reweighting, re-measure, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: rw, resource: "../../performance/reweighting-cost-vs-madgraph.md", title: "Reweighting cost against MadGraph's reweight module"}
---
[Reweighting cost against MadGraph](../../performance/reweighting-cost-vs-madgraph.md)
was measured at `f6b1936` on one 4-vCPU VM and is labelled an order-of-magnitude
comparison. It needs the same treatment as the other comparisons:

- the commit and hosts of [the MATRIX1 re-measurement](matrix-element-vs-madgraph-stale.md);
- an operator-basis row, the way EFT reweighting is used: n operators at
  NP=1 make |M|² quadratic in the coefficients, so a basis is
  (n+1)(n+2)/2 hypotheses (n = 8 → 45) on a SMEFTsim process. The polynomial
  path needs only K = 1 + n amplitude evaluations for it, against MadGraph's
  one per hypothesis. Every row today has H ≤ 11, and MadGraph's side is
  extrapolated as setup + H × loop because its multi-launch path mislabels a
  weight in 3.7.1 ([MadGraph defects](../../validation/madgraph-defects.md));
  this row measures it at the real H, or works around the defect and says how;
- both sides at `change helicity False`, recorded, since vibegraph has only the
  helicity-summed ratio ([reweight-helicity-summed-only](../feature/reweight-helicity-summed-only.md)).

**When (user, 2026-11-05).** Run after v0.2.0 is cut, at that tag, together with
the other three comparisons, then update `README.md` from the results.

**Plan (user, 2026-11-10).** The x86 v0.2.0 run (CERN Ice Lake VM, recorded in the
measurement concept) timed the musl release assets, which carried the allocator's
single-thread and thread-contention penalty. Rerun this comparison on the same host with
the v0.2.1 assets, keep the M3 run on the macOS build, and only then quote it in the README.
