---
type: Backlog Item
title: A MadGraph diagram pairing declared for both amplitude gates is not checked for agreement
description: "MG_DIAGRAM_ORDER (amplitude_oracle) and MG_GRAPH_DIAGRAMS (standalone_jamps) now share one convention; a row declared in both, as wpwm_to_wpwmz_cw will be, has nothing checking the two agree."
area: validation
state: open
priority: low
closes_when: "A row's MadGraph diagram pairing is declared once (e.g. in tests/common) and read by both gates, or a test asserts the two declarations are equal for every row in both."
blocked_by: []
opened: 2026-10-15
tags: [amplitudes, standalone-jamps, pairing, non-vacuity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: qh, resource: "../../sprints/amplitude-signs/sessions/Q-H-report.md", title: "Q-H report, Found"}
---
Found by the hygiene review of the per-graph standalone route. Both
declarations give, for each of our diagrams, its 0-based MadGraph graph.
