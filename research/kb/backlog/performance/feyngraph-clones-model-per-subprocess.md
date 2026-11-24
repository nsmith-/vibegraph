---
type: Backlog Item
title: "feyngraph clones the whole model twice for every subprocess it enumerates"
description: "DiagramGenerator::new and assign_topologies each clone the model per surviving subprocess; that clone is the ~11 ms floor measured on empty subprocesses."
area: performance
state: open
priority: medium
closes_when: "The pinned feyngraph revision shares the model (an Arc, upstream or on a fork) instead of cloning it per subprocess, and the per-subprocess enumeration time on p p > e+ e- j j is measured before and after."
blocked_by: []
opened: 2026-11-23
tags: [performance, feyngraph, diagram-enumeration, upstream]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: lt3, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-T3-report.md", title: "L-T3 report: Found 2"}
  - {id: lt2, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-T2-report.md", title: "L-T2 report: 11 ms per empty subprocess"}
---
`DiagramGenerator::new` takes `model.topo.clone()`, and
`assign_topologies` clones it again into its container
(`Arc::new(self.model.as_ref().clone())`). Both run once per surviving
subprocess. Sprint L measured about 11 ms per *empty* subprocess, and this
clone is that floor. The flavour prune removed the empty subprocesses, but
every subprocess that survives still pays it, so it matters most on
multi-jet cards with thousands of subprocesses.

feyngraph is a git dependency pinned by `rev`, so the fix is an upstream
change or a fork plus a rev bump, as for
[feyngraph-assign-counts-allocation](feyngraph-assign-counts-allocation.md).
