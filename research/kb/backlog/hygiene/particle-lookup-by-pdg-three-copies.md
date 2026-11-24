---
type: Backlog Item
title: "Particle lookup by PDG code exists three times, and one copy panics"
description: "widths.rs, diagrams/resolve.rs and diagrams/diagram.rs each look up particles or antiparticles differently; diagram.rs:1019 expects the antiparticle and panics on a malformed model."
area: hygiene
state: open
priority: low
closes_when: "One particle lookup and one antiparticle lookup on UFOModel, with the failure behaviour chosen once and no panic on a malformed model."
blocked_by: []
opened: 2026-11-24
tags: [ufo, duplication, panic]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z3-report.md", title: "L-Z3 report: F13"}
---
Found by sprint L's hygiene pass L-Z3 (F13); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z3-report.md).

Copies: `widths.rs` `particle_with_code` and `anti`; `diagrams/resolve.rs:110` `particle_by_pdg`; `diagrams/diagram.rs:1019` `antiparticle`.
