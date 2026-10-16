---
type: Backlog Item
title: A single contact diagram's amplitude has no production readout
description: "Contact diagrams get no configuration under MadGraph's get_amp2_lines rule, so neither the probe bridge nor amplitude_oracle can compare one alone, though MadGraph's AMP() exists for it."
area: validation
state: open
priority: medium
closes_when: "A test-only per-amplitude readout of contact diagrams lets the probe bridge and a MadGraph per-diagram comparison see a single contact diagram (every colour chain), with no change to the production instruction stream."
blocked_by: []
opened: 2026-10-15
tags: [evaluator, oracle-gap, contact-vertex, per-diagram]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: p1, resource: "../../sprints/amplitude-signs/sessions/P1-report.md", title: "P1 report, Found"}
---
Contact diagrams enter the bridge only inside flow sums, so errors that cancel
between a contact's colour chains go unseen there, and a contact diagram has
no per-diagram MadGraph comparison anywhere. The readout needs a `Program`
change, which the bridge's session was barred from.
