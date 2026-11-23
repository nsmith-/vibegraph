---
type: Backlog Item
title: "dim6top_LO_UFO loads but no amplitude or σ is checked against MadGraph"
description: "dim6top_LO_UFO now loads and tZq-shaped processes enumerate, but no diagram count, amplitude or σ has been compared with MadGraph, and the CMS tZq/ttZ cards were never run."
area: validation
state: open
priority: medium
closes_when: "The CMS tZq_EFT_CPV and ttZ campaign cards run on the CERN host with diagram counts and σ compared to MadGraph's, and one dim6top process gates per diagram and per helicity against MadGraph standalone."
blocked_by: []
opened: 2026-11-22
tags: [dim6top, smeft, ufo, madgraph-parity, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
sources:
  - {id: u1, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-U1-report.md", title: "Sprint L PR 7 report, Found 1"}
---
Sprint L's PR 7 made `dim6top_LO_UFO` load by evaluating its
`configuration.py`-dependent definitions. It enumerated `g g > t t~` (15
diagrams), `u b > t z d` (28) and a tZq-shaped `p p > t z j $$ w+ w-` (1236
diagrams in 60 subprocesses). The third process is the agent's own, because the
CMS `tZq_EFT_CPV_point_1_13p6` card was unreachable from the sprint container.

No amplitude, coupling or parameter value was compared with MadGraph. The
loader's tests pin which conditional arm is taken, but they share the
implementer's reading of Python precedence, so they cannot see a wrong reading
that both sides make.
