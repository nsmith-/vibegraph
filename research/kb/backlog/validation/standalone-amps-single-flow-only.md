---
type: Backlog Item
title: Standalone tables bank per-graph AMP() for single-flow rows only
description: "gen_standalone_jamps.py's bank_amps accepts single-flow rows, so gg_to_uux and every other multi-flow row has no per-graph comparison, and colour-versus-Lorentz sign splits there are unseen."
area: validation
state: open
priority: medium
closes_when: "bank_amps banks each multi-flow row's AMP() with its JAMP coefficient matrix, standalone_jamps gates them per graph, and gg_to_uux is rebanked with them."
blocked_by: []
opened: 2026-10-16
tags: [standalone-jamps, colour, amplitudes, oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: t1, resource: "../../sprints/amplitude-signs/sessions/T1-report.md", title: "T1 report, Found 4 and mutation M2"}
---
T1's mutation M2 moved the triple-gluon −1 into `f_to_traces`. Both MadGraph
gates (`standalone_jamps`, `amplitude_oracle`) still passed 49/49: per flow,
a sign moved between colour and Lorentz is invisible. Only T1's new
`triple_gluon` unit tests, on `gg_to_ttx`'s banked `AMP()`, caught it. The
`amplitude_oracle` route compares `gg_to_ttx` per flow only, even though its
table banks `AMP()`.
