---
type: Backlog Item
title: The τ map rule is decided per process, while MadEvent decides it per channel
description: "One diagram with a whole-state resonance keeps every channel on the log τ map: pp_to_bb_qcd2 and pp_to_bb_fixed are bit-identical across the 1/τ² default flip."
area: performance
state: open
priority: low
closes_when: "The τ draw is chosen per channel, measured over ≥5 seeds on p p > b b~ QCD=2 and on an MLM card with a Drell–Yan part, or the per-process rule is kept with that measurement recorded."
blocked_by: []
opened: 2026-10-17
tags: [phase-space-maps, tau-map, madevent]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v1, resource: "../../sprints/map-default-guards/sessions/V1-report.md", title: "map-default-guards V1 report, Found 1"}
---
`whole_state_resonance` is OR'd over every diagram in `MapOptions::resolve`,
and `union_shape` ORs it again over multiplicities. MadEvent's rule reads each
channel's own topology.

Filed at the close-out of [map-default-guards](../../sprints/map-default-guards/sprint.md).
