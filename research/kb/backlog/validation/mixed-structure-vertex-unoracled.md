---
type: Backlog Item
title: No MadGraph row compares a vertex that mixes structures under one coupling order
description: "A vertex mixing Identity with Gamma5, or a plain with a derivative cubic scalar, under one coupling order is pinned only against its own split form; MadGraph's per-vertex reading of it is unchecked."
area: validation
state: open
priority: low
closes_when: "A standalone row (e.g. a merged-order toy lt~ lt > qt qt~ st) with such a vertex is banked and gated per graph against MadGraph."
blocked_by: []
opened: 2026-10-15
tags: [amplitudes, toy-ufo, standalone-jamps, cp-violation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: c1, resource: "../../sprints/amplitude-signs/sessions/C1-report.md", title: "C1 report, Found"}
  - {id: c2, resource: "../../sprints/amplitude-signs/sessions/C2-report.md", title: "C2 report, Found"}
---
A CP-violating Yukawa `y(1 + iκγ⁵)` under one order is the real-model case.
`toy_models.rs::mixed_structure_vertices_equal_their_split_sum` shows the
merged vertex equals its split form to 1e-15, but a sign shared by both
structures is invisible to that check. Banking the row needs the merged toy copy
as an authored model or an order override.
