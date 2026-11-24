---
type: Backlog Item
title: "integrate reports no per-member cross section"
description: "A flavour member's σ share can only be checked by integrating each member alone, which brings its own closure error."
area: feature
state: open
priority: medium
closes_when: "integrate records each flavour member's σ (from the share accumulator) in the artifact and its report, and generate compares each member's sample share against it with a pull."
blocked_by: []
opened: 2026-11-24
tags: [integrate, flavour-groups, generate, validation-oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
---
Found by sprint L's L-X3 investigation of per-member event weights (report:
[L-X3 report](../../sprints/v023-orders-majorana-timeouts/sessions/L-X3-report.md)).

L-X3 had to integrate all 16 members of `p p > e+ e-` + `p p > e+ e- j` separately to test the sample's flavour shares, and those integrations close against the mixed run only to −1.2σ on the 1-jet part.
