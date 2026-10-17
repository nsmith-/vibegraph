---
type: Backlog Item
title: FOAM's cell-splitting sampler has never been tried as a channel sampler
description: "FOAM (Jadach) splits the hypercube into cells to cut max/mean weight, the quantity that sets unweighting efficiency; whether it beats a per-channel VEGAS grid here is unmeasured."
area: feature
state: open
priority: low
closes_when: "A hyperrectangular FOAM sampler kind runs through --sampler and is measured against default VEGAS and the bin-width-bounded grid PR #48 adds (or whatever default that work leaves) on the comparison harness's rows (≥5 seeds; 20+ on the deciding pair), and the result is recorded as adopt or reject."
blocked_by: [sampler-hardwired-to-vegas-grid, artifact-has-no-sampler-kind, integrate-cannot-select-a-sampler, sampler-comparison-harness-missing]
opened: 2026-10-17
tags: [foam, sampler, research, unweighting, weight-tail, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: foam, resource: "../../references/papers/foam.md", title: "FOAM (physics/0609068, physics/0203033)"}
---
The user asked (2026-10-17) to try [FOAM](../../references/papers/foam.md). It
builds a binary tree of cells in an exploration phase, splitting where the
split most reduces the max/mean weight ratio (event generation) or the variance
(integration), then samples a cell and a point uniformly in it. The max/mean
objective is the one the weight tail spoils here
([grid weight tail](../../phase-space/vegas-grid-weight-tail.md)), and a cell
tree follows correlations a separable grid cannot.

What decides whether it fits:

- **Dimension.** The 2003 FOAM paper reports adaptive integration up to 16
  dimensions on a typical PC, a limit it expects to rise with CPU power; a
  hadronic `2 → n` channel has `2 + (3n − 4)`, so the `2 → 6` rows reach 16 and
  `2 → 2`, `2 → 3` sit well inside it. Start with hyperrectangular cells, the
  only kind mFOAM keeps, whose storage per cell is constant in `n`; simplices
  stored their vertices at a cost `∝ n · N_cells` before version 2.06.
- **Where it sits.** Per channel, in place of the grid, on the residual each
  map leaves; the multichannel decomposition, α survey and budget are unchanged.
- **Determinism.** A point draws a cell then `d` coordinates (`1 + d` draws).
  Splits are sequential; each cell's exploration is parallel, so its statistics
  must be reduced in point order to keep `-j` byte identity.
- **Cost.** Exploration spends evaluations per split before any σ is quoted;
  the harness reports training evaluations apart from the rest.

Check for a maintained Rust implementation before porting; ROOT's `TFoam` is
the C++ reference. Measure on the harness's rows against default VEGAS and the
`--grid-max-bin-width` grid PR #48 measured, once it merges, or whatever default
it leaves.
Design: [importance-sampler-seam](../../phase-space/importance-sampler-seam.md).
