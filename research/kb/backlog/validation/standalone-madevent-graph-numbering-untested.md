---
type: Backlog Item
title: Standalone and MadEvent graph numbering are assumed identical, not tested
description: "wpwm_to_wpwmz_cw's MadEvent cell reuses the standalone table's diagram pairing because the amplitude-closing CALL lines of the two matrix.f files matched in one diff; no test checks it."
area: validation
state: open
priority: low
closes_when: "gen_standalone_jamps.py banks each graph's closing HELAS call and coupling beside its AMP(), and a check compares them with the MadEvent matrix1_orig.f for every row that shares a pairing."
blocked_by: []
opened: 2026-10-15
tags: [amplitudes, standalone-jamps, pairing, oracle-gap]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: f2, resource: "../../sprints/amplitude-signs/sessions/F2-report.md", title: "F2 report, Found"}
---
F1 found the same gap from the generator side: matching graphs to couplings
needed `matrix.f` parsed separately.
