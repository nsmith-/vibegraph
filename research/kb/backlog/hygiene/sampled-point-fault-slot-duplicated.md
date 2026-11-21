---
type: Backlog Item
title: "The sampled-point fault slot and its refusal are written twice"
description: "ProtonIntegrand and FixedBeamIntegrand hold identical OnceLock<SampledPointFault> slots, and refuse_sampled_fault is written in both the CLI and generate.rs."
area: hygiene
state: open
priority: low
closes_when: "One FaultSlot type serves both integrands and one conversion on SampledPointFault serves both refusals; σ bit for bit."
blocked_by: []
opened: 2026-11-19
tags: [refusal, duplication, integrate, generate]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Found by K-H3 (hygiene pass on #143, #152 and #156, PR #159), Found 1 (finding H9).
