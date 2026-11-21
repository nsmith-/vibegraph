---
type: Backlog Item
title: "Fixed-energy beams refuse any card whose subprocesses differ in external particles"
description: "e+ e- > l+ l- with l = e mu is refused by InconsistentExternals even though every mass agrees; MadGraph runs it. The proton path now splits such subprocesses into summed parts."
area: feature
state: open
priority: low
closes_when: "A fixed-beam card whose subprocesses differ in external particles integrates as a sum of parts (by content or by masses, as the proton path splits), with σ matching MadGraph by the multi-seed criterion on e+ e- > l+ l- with l = e mu ta."
blocked_by: []
opened: 2026-11-17
tags: [fixed-energy, flavour-groups, refusal]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Found by sprint K's K-X1 session (its report, `sessions/K-X1-report.md` in the
sprint folder). `HadronicError::InconsistentExternals` (`compile_subprocesses`,
`hadronic.rs`) compares external particle IDs. It is a named refusal, not a wrong
answer, and it also covers `generate`.
