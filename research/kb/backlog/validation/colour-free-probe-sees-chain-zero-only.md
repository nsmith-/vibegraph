---
type: Backlog Item
title: The colour-free per-diagram probe sees colour chain zero only
description: "compile_diagram_ast roots every diagram at chain [0; n], so a per-diagram dump built on it misses the other colour structures of a contact (16 amplitudes across g g > g g and g g > t t~ g)."
area: validation
state: open
priority: low
closes_when: "probe_process_diagrams (and any per-diagram dump) iterates the root_amplitudes keys, so every (diagram, colour chain) amplitude can be dumped and compared."
blocked_by: []
opened: 2026-10-15
tags: [evaluator, probe, colour, oracle-gap]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: p1, resource: "../../sprints/amplitude-signs/sessions/P1-report.md", title: "P1 report, Found"}
---
Found while bridging the probes to production. The bridge itself compares
every chain through `run_config_amps`; only the diagnostic dump is limited.
