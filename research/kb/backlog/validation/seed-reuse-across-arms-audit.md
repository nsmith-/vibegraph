---
type: Backlog Item
title: Seed-for-seed arm comparisons are paired, not independent, and are quoted as independent
description: "A generation seed replayed off two grids gives correlated samples (flavour χ²/dof 0.43 vs 0.95) and llj σ correlates across maps at ρ = 0.45; comparisons that reuse seeds need a paired reading."
area: validation
state: open
priority: medium
closes_when: "The probes and notes that compare maps or grids seed-for-seed (note 37's same-map seed controls among them) are audited, and each either uses disjoint seeds or states and uses the paired error."
blocked_by: []
opened: 2026-10-17
tags: [seed-policy, statistics, maps]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v0, resource: "../../sprints/map-default-guards/sessions/V0-report.md", title: "map-default-guards V0 report, Found 1"}
---
A generation seed fixes the whole trial stream, and two grids trained on one
card differ only slightly. Same-seed cross-grid flavour χ²/dof read 0.43 (20
readings) against 0.95 for different seeds (80); llj_dyn's default and soft-all
at the same seed correlate at ρ = 0.45. V1's seed-paired default − legacy on
`pp_to_llj_fixed` (+0.067 ± 0.073 pb, 80 seeds) is read paired.

Filed at the close-out of [map-default-guards](../../sprints/map-default-guards/sprint.md) (D1 point 4).
