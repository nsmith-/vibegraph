---
type: Backlog Item
title: Core leg order on crossed initial states with b̄ differs from MadGraph's
description: "Where both beams are antiquarks and one is b̄, MadGraph orders the core final-state legs as the crossed process it reuses does (s~ b~ > w+ s~ c~); vibegraph uses product order (s~ b~ > w+ c~ s~)."
area: validation
state: open
priority: low
closes_when: "vibegraph orders the legs of a crossed process as MadGraph's crossing reuse in generate_multi_amplitudes does, and the decay-chain census matches MadGraph's keys on Zto2Q's @2 line and Wto2Q's @1/@4 and @2/@5 lines, with a Wto2Q census row added."
blocked_by: []
opened: 2026-11-05
tags: [diagrams, crossing, leg-order, madgraph-parity, lhef]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
sources:
  - {id: report, resource: "https://github.com/nsmith-/vibegraph/blob/02bea373ea0e6287f745ad43d3d3f832dbd4bc9a/research/kb/sprints/cms-gridpack-coverage/sessions/H-D3-report.md", title: "Sprint H session H-D3 report, §4 item 2 and Found"}
  - {id: census, resource: "../../../../vibegraph-lib/tests/decay_chain_census.rs", title: "vibegraph-lib/tests/decay_chain_census.rs"}
---
In MadGraph, ME447 of the as-run Wto2Q card is `s~ b~ > w+ s~ c~`, final legs
`[2, -5, -3, -4]`. vibegraph gives `[2, -5, -4, -3]`. The core `j j` order
(s̄ c̄) is not the first one `itertools.product` meets over `j` (c̄ s̄). The
difference appears only where both beams are antiquarks and one is b̄, which
points at MadGraph reusing a crossed process's leg order (the "Check for
crossed processes" step of `generate_multi_amplitudes`), not at the order of
product generation.

The census misses 40 keys on Z_l2, 96 on W_l14 and 408 on W_l25. σ is
unchanged; labels and the LHE particle order differ. This also blocks a
Wto2Q census row. Measured by sprint H (H-D3 report, §4 item 2).
