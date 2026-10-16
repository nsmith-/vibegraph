---
type: Backlog Item
title: The per-diagram dump, pairing and GF(2) tools every amplitude diagnosis used are not committed
description: "W1, F1, C1, Q1 and B′ each rebuilt a per-diagram dump for standalone and SMEFTsim rows, a MadGraph pairing and a GF(2) sign solve in the scratchpad; none is in the tree."
area: validation
state: open
priority: medium
closes_when: "A committed probe dumps per-(diagram, colour chain) values for any standalone row, including SMEFTsim and toy models; compare_amps.py (or a successor) pairs them from a table's AMP(); and gen_standalone_jamps.py can evaluate at another table's points."
blocked_by: []
opened: 2026-10-16
tags: [tooling, amplitudes, diagnosis, standalone-jamps]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: w1, resource: "../../sprints/amplitude-signs/sessions/W1-report.md", title: "W1 report, Found 4"}
  - {id: c1, resource: "../../sprints/amplitude-signs/sessions/C1-report.md", title: "C1 report, Found 4"}
  - {id: q1, resource: "../../sprints/amplitude-signs/sessions/Q1-report.md", title: "Q1 report, Found 3"}
  - {id: b, resource: "../../sprints/amplitude-signs/sessions/B-report.md", title: "B′ report, Found"}
---
`probe_process_diagrams` reads only SM MadEvent work areas and colour chain zero
([colour-free-probe-sees-chain-zero-only](colour-free-probe-sees-chain-zero-only.md)).
B′ notes a chain-zero shortcut once produced a false difference on
`u u~ > t t~ g`; a committed tool must use real colour chains.
