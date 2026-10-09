---
type: Backlog Item
title: VEGAS+ adaptive stratification has never been measured here
description: The integrator is classic Lepage VEGAS; whether VEGAS+ within-channel stratification improves convergence on these integrands is unmeasured.
area: feature
state: open
priority: low
closes_when: VEGAS+ stratification is measured against classic VEGAS on the sigma gates' rows at matched points (seed sweep, chi2/dof), and the result is recorded as adopt or reject.
blocked_by: []
opened: 2026-08-06
tags: [research, vegas, integration]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L777-L786", title: "TODO.md entry T062"}
---
VEGAS+ (arXiv:2009.05112) adds adaptive stratified sampling within the
importance grid and reports 2–19× on integrands with multiple peaks or
diagonal structure. Here the channel decomposition already handles diagonal
structure and the budget is already stratified across channels, so the
within-channel gain is an open question.

Measure on the sigma gates' rows at matched points, over a seed sweep with
chi2/dof, before deciding. Stratification changes the sampling order, so it
cannot be bit-for-bit against banked artifacts; gates must be statistical.
