---
type: Backlog Item
title: The README's 0.87× MATRIX1 ratio predates the evaluator's latest changes
description: "README.md quotes a 0.87× geometric-mean cost against MadGraph's MATRIX1 as current; constant collection, fused scaled sums and bare configuration amplitudes landed after it, unmeasured against MATRIX1."
area: performance
state: open
priority: medium
closes_when: "scripts/mg_perf_compare.sh is rerun over its nineteen processes on the current tree (host recorded), and README.md quotes that run."
blocked_by: []
opened: 2026-10-09
tags: [matrix-element, madgraph-comparison, readme, re-measure]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: readme, resource: "../../../../README.md", title: "README.md 'The matrix element, per point' (~:310-322)"}
  - {id: topdown, resource: "../../topdown-zen4-results.md", title: "topdown-zen4-results §6-§7, the in-process gains since"}
---
`README.md` (~:310-322) states matrix-element evaluation at 0.67×–1.37× of
MadGraph's `matrix1_optim.f`, geometric mean 0.87× over the nineteen processes
of `scripts/mg_perf_compare.sh`, as current. Since then:

- constant collection and weighted JAMP sums;
- bare configuration amplitudes;
- the real constant-product fold

all landed ([topdown-zen4-results](../../topdown-zen4-results.md) §6-§8), with
1.00–1.21× scalar default-target gains measured in-process. None was measured
against MATRIX1. The public number is probably pessimistic, but it is stale
either way. Rerun the comparison and quote it with its host. Found by drafter
D5 and verifier V5.
