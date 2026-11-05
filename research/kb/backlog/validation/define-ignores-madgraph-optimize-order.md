---
type: Backlog Item
title: define keeps its members in written order where MadGraph sorts them
description: "MadGraph sorts a define's members (massless first, then colour 8, 3, 1, 3̄) and the 5F p/j rewrite; vibegraph keeps the written order and appends b b~, so leg order differs."
area: validation
state: open
priority: low
closes_when: "Label resolution applies MadGraph's optimize_order to every define and to the five-flavour p/j rewrite, the decay-chain census matches MadGraph's keys on the as-run Zto2Q and Wto2Q @0/@3 lines with the define as written, and the banked references that key on leg order are checked or regenerated."
blocked_by: []
opened: 2026-11-05
tags: [proc-card, define, leg-order, madgraph-parity, lhef]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
sources:
  - {id: report, resource: "https://github.com/nsmith-/vibegraph/blob/02bea373ea0e6287f745ad43d3d3f832dbd4bc9a/research/kb/sprints/cms-gridpack-coverage/sessions/H-D3-report.md", title: "Sprint H session H-D3 report, §4 item 1 and Found"}
  - {id: mg-optimize-order, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/madgraph_interface.py#L5643", title: "MadGraph madgraph_interface.py optimize_order, applied by do_define"}
  - {id: resolve, resource: "../../../../vibegraph-lib/src/diagrams/resolve.rs", title: "vibegraph-lib/src/diagrams/resolve.rs, model_aliases and rewrite_default_multiparticles"}
---
MadGraph's `do_define` stores a label's members through `optimize_order`:
massless first, then colour octet, triplet, singlet, antitriplet. So
`define p = 21 2 4 1 3 -2 -4 -1 -3 5 -5` is stored as
`[21, 2, 4, 1, 3, 5, -2, -4, -1, -3, -5]`, and the five-flavour `j` comes out the
same way straight after `import model`. vibegraph keeps the written order and
appends `b b~` in its p/j rewrite.

σ is unchanged. Subprocess labels and the LHE particle order differ: run with
the define as written, the as-run Zto2Q and Wto2Q cards miss 20 of Z_l1's
census keys and 36 of W_l03's. With the define written in MadGraph's stored
order, both match fully. Measured by sprint H (H-D3 report, §4 item 1).
