---
type: Backlog Item
title: Standalone and MadEvent table generators duplicate a reader and disagree on a format
description: "read_masses is written in both gen_standalone_jamps.py and gen_amplitude.py, and jamp_coefficients is reals in standalone tables but [re, im] pairs in amplitude tables."
area: hygiene
state: open
priority: low
closes_when: "gen_standalone_jamps.py imports read_masses from one place, and both table kinds write jamp_coefficients in one format (at the next regeneration of the tables that carry it)."
blocked_by: []
opened: 2026-10-15
tags: [duplication, validation-scripts, table-format]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: qh, resource: "../../sprints/amplitude-signs/sessions/Q-H-report.md", title: "Q-H report, Found"}
---
The JAMP coefficient parser itself was unified in the same review; these two
remain. Changing the coefficient format means regenerating `ee_to_ee_4f.json`.
