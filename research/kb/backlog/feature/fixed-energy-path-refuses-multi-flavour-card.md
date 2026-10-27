---
type: Backlog Item
title: "The fixed-energy path refuses a card whose subprocesses differ in external particles"
description: "compile_subprocesses requires identical externals (InconsistentExternals), so e+ e- > l+ l- with l = e, mu or e, ta is refused at lpp = 0, though the proton path now sums such parts."
area: feature
state: open
priority: low
closes_when: "A fixed-beam card with several final-state flavours integrates as a sum over parts (as MultiplicitySum does on the proton path) with σ checked against MadGraph, or its refusal names the cause."
blocked_by: []
opened: 2026-10-26
tags: [fixed-beam, flavour-groups, refusal, multi-process]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f1, resource: "../../sprints/flavour-mixed-masses/sessions/F1-report.md", title: "flavour-mixed-masses F1 report, Found 1"}
---
`compile_subprocesses` (`vibegraph-lib/src/hadronic.rs`, the
`InconsistentExternals` return near :997) requires every subprocess to have
identical external particles. `e+ e- > l+ l-` with `l ∋ ta` is refused at
`lpp = 0`, and so is `l = e, mu`, whose masses agree. It is a refusal, not wrong
physics.

The proton path splits such a card by outgoing masses and sums one integrand per
part (`multiplicity::split_by_outgoing_masses`, `MultiplicitySum`). The fixed-beam
integrand has no composite like it
([F1](../../sprints/flavour-mixed-masses/sessions/F1-report.md)).
