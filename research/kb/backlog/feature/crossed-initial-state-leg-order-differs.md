---
type: Backlog Item
title: "Core leg order on crossed initial states with b̄ differs from MadGraph"
description: "MadGraph writes s~ b~ > w+ s~ c~ where vibegraph writes s~ b~ > w+ c~ s~ (W_l14, W_l25, Z_l2 census keys); it comes from MadGraph's crossing reuse and blocks a Wto2Q census row."
area: feature
state: open
priority: low
closes_when: "vibegraph's core leg order on crossed initial states matches MadGraph's crossing reuse in generate_multi_amplitudes, and the W_l14, W_l25 and Z_l2 census keys match, so a Wto2Q census row can be added."
blocked_by: []
opened: 2026-11-07
tags: [diagrams, crossing, madgraph-parity, census]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hd3, resource: "../../sprints/cms-gridpack-coverage/sessions/H-D3-report.md", title: "H-D3 report, Found: core leg order on crossed initial states"}
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log: #93 Found triage, item 3"}
---
Found by H-D3 (#93 Found 3; the manager's triage files it). MadGraph writes
`s~ b~ > w+ s~ c~` where vibegraph writes `s~ b~ > w+ c~ s~`: 96 keys on W_l14,
408 on W_l25 and 40 on Z_l2 (H-D3 §4, item 2). The source is MadGraph's crossing
reuse in `generate_multi_amplitudes` (the "Check for crossed processes" block),
which vibegraph does not mirror. It changes labels and record order, not σ
(inferred from H-D3's census, which compares contents). It also blocks a Wto2Q
census row. The label-sorting difference is
[define-labels-skip-madgraph-optimize-order](define-labels-skip-madgraph-optimize-order.md).
