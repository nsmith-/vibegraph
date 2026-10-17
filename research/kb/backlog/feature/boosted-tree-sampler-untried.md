---
type: Backlog Item
title: Bendavid's boosted-decision-tree sampler has never been tried
description: "A non-negative boosted tree ensemble is a sampleable box density that follows correlations (arXiv:1707.00028); untried, and its training set is every point the run drew."
area: feature
state: open
priority: low
closes_when: "A generative-BDT sampler kind runs through --sampler and is measured against default VEGAS and the bin-width-bounded grid PR #48 adds (or whatever default that work leaves) on the comparison harness's rows (≥5 seeds; 20+ on the deciding pair), and the result is recorded as adopt or reject."
blocked_by: [sampler-hardwired-to-vegas-grid, artifact-has-no-sampler-kind, integrate-cannot-select-a-sampler, sampler-comparison-harness-missing]
opened: 2026-10-17
tags: [boosted-decision-trees, machine-learning, sampler, research, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: bendavid, resource: "../../references/papers/bendavid-bdt-dnn-integration.md", title: "Bendavid, arXiv:1707.00028"}
---
The user named BDT samplers (2026-10-17) as a candidate.
[Bendavid's method](../../references/papers/bendavid-bdt-dnn-integration.md)
generalises FOAM's single tree to a boosted ensemble of overlapping boxes with
non-negative weights, sampled by choosing a box `∝ V·a` and a point uniformly in
it, trained alternately with a regression ensemble `e^h ≈ f` on points drawn
from the current model. On the 4-dimensional camel function it reached `σ_w/⟨w⟩` 0.082 (0.077
staged) against VEGAS's 2.82, both at 3·10⁵ evaluations; on the 9-dimensional
one 0.63 (0.31 staged) at 3.2·10⁶ evaluations against VEGAS's 19 at 1.5·10⁶.

What decides whether it fits:

- **The test was unmapped.** Here each channel's map has already flattened the
  peaks it knows; the gain on the residual is the open question, as for VEGAS+.
- **The accumulator is the data.** Training needs every `(u, f)` drawn, not a
  histogram, so memory grows with the budget; the seam's accumulator must
  allow that ([importance-sampler-seam](../../phase-space/importance-sampler-seam.md)).
- **A dependency.** Gradient boosting with this custom loss, squared error in
  log space (eqs. 2.7–2.9), needs a
  library or an in-tree trainer; the AGENTS.md rule on standard primitives
  applies, and the choice is recorded with the requirement it meets.
- **Determinism.** Training must be a function of the points in point order,
  not of thread scheduling, for `-j` byte identity of the artifact.

Measure on the harness's rows against default VEGAS and the
`--grid-max-bin-width` grid, once PR #48 merges, or whatever default it leaves. The paper's generative network is not separately
listed: [normalizing flows](normalizing-flow-sampler-untried.md) subsume it with
a density computable at any point.
