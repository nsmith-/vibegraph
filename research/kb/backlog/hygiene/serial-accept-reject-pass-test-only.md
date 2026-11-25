---
type: Backlog Item
title: "The serial accept/reject pass is reached only from tests"
description: "Unweighter::trial, next_event and their stats and scale_draw state are no longer reached from production after the blocked pass."
area: hygiene
state: open
priority: low
closes_when: "Non-probe users move onto BlockedPass and the rest is deleted, or the serial pass is recorded as the census probe's reference."
blocked_by: []
opened: 2026-11-24
tags: [generate, unweighting, dead-code]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z8-report.md", title: "L-Z8 report: F10"}
---
Found by sprint L's hygiene pass L-Z8 (F10); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z8-report.md).

Users: the `unweight.rs` tests, `hadronic.rs:5933`, `validate_lhef.rs:531`, `validate_samples.rs:2712` (the census probe), `decay_chain_ladder.rs:208`.
