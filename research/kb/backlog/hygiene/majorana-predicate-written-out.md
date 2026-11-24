---
type: Backlog Item
title: "The Majorana-fermion test is written out in four places"
description: "spin.abs() == 2 && is_self_conjugate appears in ufo/report.rs, ufo/bind.rs and diagrams/diagram.rs (twice) instead of one Particle method."
area: hygiene
state: open
priority: low
closes_when: "One Particle::is_majorana_fermion serves all four sites."
blocked_by: []
opened: 2026-11-24
tags: [ufo, majorana, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z5-report.md", title: "L-Z5 report: Found 3"}
---
Found by sprint L's hygiene pass L-Z5 (Found 3); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z5-report.md).

Also L-Z2 Found 1. Sites: `ufo/report.rs:281`, `ufo/bind.rs:252`, `diagrams/diagram.rs:668,701`.
