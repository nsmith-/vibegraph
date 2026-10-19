---
type: Backlog Item
title: pp_to_jj's flavour composition reads in excess against MadGraph's one banked sample under either τ map
description: "The jj flavour χ² against MadGraph's banked 10 000 events reads p ≈ 2e-3 – 4e-2 under both τ maps, while our own grids agree with each other; not yet split into ours vs the reference's draw."
area: validation
state: open
priority: medium
closes_when: "This side's per-subprocess-directory σ is compared with MadGraph's per-channel integrated σ (Accumulated results in each banked P1_*/G*/run_01_log.txt) and the excess is attributed; a second banked jj sample is requested only if that does not settle it."
blocked_by: []
opened: 2026-10-17
tags: [pp-to-jj, samples-gate, flavour-composition]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v0, resource: "../../sprints/map-default-guards/sessions/V0-report.md", title: "map-default-guards V0 report, Part A and Found 5"}
  - {id: v1, resource: "../../sprints/map-default-guards/sessions/V1-report.md", title: "map-default-guards V1 report, second grid pair"}
---
The gate passes (minimum p 8.2e-3 – 4.2e-2 against a 1e-4 floor), but both maps
read the same excess against MadGraph's one 10 000-event draw while our own
grids agree with each other. Every banked event carries `IDPRUP = 1`, so the
sample cannot be split by subprocess; each `P1_*/G*/run_01_log.txt` prints the
integrated per-channel σ ("Accumulated results: Integral = …"), which compares
this side's composition with MadGraph's integration rather than its draw.

Filed at the close-out of [map-default-guards](../../sprints/map-default-guards/sprint.md) (D1 point 3).
