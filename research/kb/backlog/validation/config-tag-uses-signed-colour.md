---
type: Backlog Item
title: "The configuration tag uses the signed particle colour where MadGraph uses the unsigned one"
description: "compile.rs config_tag folds particle.color into the tag; MadGraph's IdentifyConfigTag uses the unsigned property, so u and u~ lines at one split tag differently."
area: validation
state: open
priority: low
closes_when: "The tag uses the unsigned colour, or a test with an antiquark propagator against configs.json shows the two agree."
blocked_by: []
opened: 2026-10-10
tags: [configurations, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: f-c-report, resource: "../../sprints/hygiene/sessions/F-C-report.md", title: "F-C report (hygiene sprint)"}
---
F-C Found 2 (after make_anti stopped negating singlets and octets, only triplets and sextets differ). No banked gate moved.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [F-C](../../sprints/hygiene/sessions/F-C-report.md).
