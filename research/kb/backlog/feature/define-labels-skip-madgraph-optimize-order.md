---
type: Backlog Item
title: "define keeps the written member order, where MadGraph sorts it with optimize_order"
description: "MadGraph sorts a defined label's members (massless first, then colour 8 > 3 > 1 > 3̄); vibegraph keeps the written order, so subprocess labels and LHE particle order differ. σ is the same."
area: feature
state: open
priority: low
closes_when: "Label resolution applies optimize_order's sort (and the five-flavour p/j rewrite matches MadGraph's member order), the decay-chain census keys match MadGraph's on Z_l1, and banked references that key on leg order are checked."
blocked_by: []
opened: 2026-11-07
tags: [proc-card, labels, madgraph-parity, lhef, census]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hd3, resource: "../../sprints/cms-gridpack-coverage/sessions/H-D3-report.md", title: "H-D3 report, Found: define does not apply optimize_order"}
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log: #93 Found triage, item 2"}
---
Found by H-D3 (#93 Found 2; the manager's triage files it).

- MadGraph sorts every `define`d label's members, massless first, then colour
  8 > 3 > 1 > 3̄, so `define p = 21 2 4 1 3 -2 -4 -1 -3 5 -5` lists b among the
  quarks.
- vibegraph keeps the written order. The five-flavour p/j rewrite also differs:
  vibegraph appends `b b~`, while MadGraph's `j` is
  `[21, 2, 4, 1, 3, 5, -2, -4, -1, -3, -5]` straight after the import.

Effect: different leg orders for the same subprocess, hence different subprocess
labels and LHE particle order, with σ unchanged (H-D3 §4, item 1: 20 of Z_l1's
census keys). The core-leg-order difference on crossed b̄ initial states is a
separate cause: [crossed-initial-state-leg-order-differs](crossed-initial-state-leg-order-differs.md).
