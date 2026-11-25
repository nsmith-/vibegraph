---
type: Backlog Item
title: "A UFO dict propagator without key 0 is silently read as no propagator"
description: "propagator_ref treats a dict propagator lacking key 0 as no propagator; MadGraph's value[0] raises there."
area: feature
state: open
priority: medium
closes_when: "propagator_ref refuses such a propagator by name, with a test."
blocked_by: []
opened: 2026-11-24
tags: [ufo, propagators, refusal]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z2-report.md", title: "L-Z2 report: Found 3"}
---
Found by sprint L's hygiene pass L-Z2 (Found 3); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z2-report.md).

A silent fallback changes the amplitude without a message.
