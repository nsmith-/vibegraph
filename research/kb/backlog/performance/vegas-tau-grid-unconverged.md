---
type: Backlog Item
title: The trained VEGAS τ grid looks unconverged
description: "A one-shot refit of each channel's τ marginal from frozen-grid points gives 0.85× row variance on pp_to_jj and 0.95× on ℓℓj against the trained grid, not at equal budget."
area: performance
state: open
priority: low
closes_when: "An equal-budget comparison (the same total evaluations) of the default training against variants that could close the gap — damping α, iteration count, nbins on τ — at ≥5 seeds per arm, read by spread and χ²/dof, either shows a variant beating the default (then 20+ seeds on the deciding pair before any default moves) or shows the replay's gain was the extra points."
blocked_by: []
opened: 2026-10-22
tags: [vegas, tau, convergence, measurement]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
sources:
  - {id: design, resource: "../../phase-space/sampler-and-map-seams.md", title: "The importance sampler and the phase-space map, designed together (D3, Found)"}
---
In the shared-tier replay (`probe_shared_outer_tier_replay`, five seeds), the
`own` arm refits each channel's `τ` marginal as `√G` on 128 fine bins, bounded at
`K = 6`, from 100 000 frozen-grid points per channel, and evaluates on 100 000
independent ones. Against the trained grid the row variance is 0.855 ± 0.013
(`pp_to_jj`), 0.946 ± 0.009 (`pp_to_llj_fixed`) and 0.952 ± 0.011
(`pp_to_llj_mlm`); refitting `y` gains nothing. The refit spends extra points,
so this may be budget, not convergence; the equal-budget test decides.
See [the joint seam design](../../phase-space/sampler-and-map-seams.md), D3.
