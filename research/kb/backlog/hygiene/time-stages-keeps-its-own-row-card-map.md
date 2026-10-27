---
type: Backlog Item
title: "time_stages.py keeps its own row-to-card map for the dy13 rows"
description: "validation/madgraph/time_stages.py maps dy13_default and dy13_mmll_60_120 to run cards by hand and does not know dy13_beams_6500_4000 or dy13_etau_default."
area: hygiene
state: open
priority: low
closes_when: "time_stages.py reads each row's run card from the manifest (or the banked run's Cards/run_card.dat) and times every dy13 row."
blocked_by: []
opened: 2026-10-26
tags: [tooling, madgraph, timing, manifest]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: fm-v1, resource: "../../sprints/flavour-mixed-masses/sessions/V1-report.md", title: "flavour-mixed-masses V1 report, Found 2"}
  - {id: ub-v2, resource: "../../sprints/unequal-beams/sessions/V2-report.md", title: "unequal-beams V2 report, Found 2"}
---
`validation/madgraph/time_stages.py:221-222` maps `dy13_default` and
`dy13_mmll_60_120` to their run cards, and `time_dy13` drives MadGraph from that
map. The two dy13 rows sprint D banked (`dy13_beams_6500_4000`,
`dy13_etau_default`) are not in it. Two sessions found it independently
([flavour-mixed-masses V1](../../sprints/flavour-mixed-masses/sessions/V1-report.md),
[unequal-beams V2](../../sprints/unequal-beams/sessions/V2-report.md)).
