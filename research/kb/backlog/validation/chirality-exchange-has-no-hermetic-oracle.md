---
type: Backlog Item
title: "No hermetic test catches a broken chirality exchange"
description: "Mutation M9 (no chirality swap left of a gamma) survives the hermetic suite; only the extended-validation MSSM rows catch it, so CI's test job is blind to it."
area: validation
state: open
priority: medium
closes_when: "A committed toy model with a chiral scalar vertex (ProjM/ProjP with different couplings, the t~ b H+ shape) and a banked AMP() table for it make CI's test job fail on M9."
blocked_by: []
opened: 2026-11-24
tags: [amplitudes, chirality, tests, mutation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z2-report.md", title: "L-Z2 report: Found 2"}
---
Found by sprint L's hygiene pass L-Z2 (Found 2); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z2-report.md).

PR 2's g2HDM chirality bug was exactly this class, and it shipped in v0.2.2.
