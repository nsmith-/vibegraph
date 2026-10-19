---
type: Backlog Item
title: pp_to_llj_dyn's scatter guard has no 80-seed calibration under the shipped soft-all + 1/τ² maps
description: "V0 calibrated soft-all with log τ; the flip ships soft-all with 1/τ², whose only reading is the banked gate quintet (χ²/dof 1.03)."
area: validation
state: open
priority: medium
closes_when: "probe_llj_dyn_scatter_guard_calibration is run on the shipped defaults (80 seeds, about 96 min) and its χ²/dof, sd/quoted and trip rate are recorded beside the fixed-scale twin's."
blocked_by: []
opened: 2026-10-17
tags: [scatter-guard, pp-to-llj, calibration, maps]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v1, resource: "../../sprints/map-default-guards/sessions/V1-report.md", title: "map-default-guards V1 report, Found 2"}
---
The fixed-scale twin reads χ²/dof 0.92 over 80 seeds under either map (trip
rate at 4.0: 0.02% new, 0.20% legacy). V0's probe names changed meaning with
the default: the "default" probe is now soft-all + 1/τ², V0's default arm is
`…_legacy`.

Filed at the close-out of [map-default-guards](../../sprints/map-default-guards/sprint.md).
