---
type: Backlog Item
title: VEGAS+ adaptive stratification has never been measured here
description: The integrator is classic Lepage VEGAS; whether VEGAS+ within-channel stratification improves convergence, unweighting efficiency or the weight tail on these integrands is unmeasured.
area: feature
state: open
priority: low
closes_when: "VEGAS+ runs as a sampler kind and is measured on the comparison harness's rows against default VEGAS and the bin-width-bounded grid PR #48 adds (--grid-max-bin-width K, or whatever default that work leaves) at matched points: σ spread and χ²/dof over ≥5 seeds (20+ on the deciding pair), ε_unw and the Hill tail index; the result is recorded as adopt or reject."
blocked_by: [sampler-hardwired-to-vegas-grid, artifact-has-no-sampler-kind, integrate-cannot-select-a-sampler, sampler-comparison-harness-missing]
opened: 2026-08-06
tags: [research, vegas, integration, sampler, unweighting, weight-tail]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L777-L786", title: "TODO.md entry T062"}
---
VEGAS+ (arXiv:2009.05112) adds adaptive stratified sampling within the
importance grid and reports 2–19× on integrands with multiple peaks or
diagonal structure. Here the channel decomposition already handles diagonal
structure and the budget is already stratified across channels, so the
within-channel gain is an open question.

It lands as a sampler kind behind the seam
([sampler-hardwired-to-vegas-grid](sampler-hardwired-to-vegas-grid.md)): its
trained state is the grid plus the per-hypercube allocation, which the artifact
must hold ([artifact-has-no-sampler-kind](artifact-has-no-sampler-kind.md)), and
the allocation must be a function of the iteration's plan, not of thread order,
to keep `-j` byte identity.

Measure with the [comparison harness](../validation/sampler-comparison-harness-missing.md)
on the sigma gates' rows at matched points. Two baselines: default VEGAS, and
the grid with PR #48's bin-width bound (`--grid-max-bin-width K`, which PR #48's
grid-rule measurements found at `K = 4` raising ε_unw 1.2–1.6× and the tail index on every tail
row), or whatever default that work leaves once it merges. Report σ spread and χ²/dof,
unweighting efficiency ε_unw and the Hill tail index, since a gain in variance
that fattens the tail is not a gain for event generation. Stratification
changes the sampling order, so the gates are statistical, never bit-for-bit
against banked artifacts.
