---
type: Backlog Item
title: "VBFRadionTo2W M1800 integrates to 2.2e-15 pb with iteration χ²/dof 8–26"
description: "Card 636 (VBFRadionTo2W_Inclusive_narrow M1800) integrates to 2.2e-15 pb with inconsistent iterations; a TeV VBF radion is ordinarily ~fb, so this may be a silent wrong answer or a vanishing coupling."
area: validation
state: open
priority: medium
closes_when: "Card 636's σ is compared with its gridpack's MadGraph σ, and either the difference is diagnosed per event or per diagram, or the value is confirmed and the iteration inconsistency is attributed to a channel."
blocked_by: []
opened: 2026-11-23
tags: [sigma, vbf, cms-gridpacks, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Card 636 (15 parts, 1544 flavour groups, 236 channels) ran 11 VEGAS
iterations in 802 s to 5.65e-24 GeV⁻² (2.2e-15 pb). The estimate jumped
about +2σ between iterations 4 and 5, with χ²/dof 26.5 falling to 8.0; at
the stop rule's widened error it was ~3% from a 1% target.

A 1.8 TeV VBF radion would ordinarily be ~fb, so the value may be a silent
wrong answer or a parameter card with a vanishing coupling (cf.
[`param-card-zero-couplings-keep-dead-diagrams`](../performance/param-card-zero-couplings-keep-dead-diagrams.md)).
Evidence: [L-T2 report](../../sprints/v023-orders-majorana-timeouts/sessions/L-T2-report.md), [phase sample](../../sprints/v023-orders-majorana-timeouts/v022-1009-phase-sample.md).
