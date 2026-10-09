---
type: Paper
title: VEGAS (Lepage 1978)
description: "J. Comput. Phys. 27 (1978) 192: the adaptive importance-sampling grid that vibegraph's integrator implements, with one departure in how iterations are combined."
resource: "https://doi.org/10.1016/0021-9991(78)90004-9"
status: draft
tags: [vegas, integration, importance-sampling, paper, monte-carlo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n01-vegas, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L155-L172", title: "Note 01, classic VEGAS summary"}
  - {id: n01-lips, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L183-L196", title: "Note 01, phase-space integration and relevance"}
  - {id: n00-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/00-overview.md#L57-L69", title: "Note 00, references"}
  - {id: vegas-rs, resource: "vibegraph-lib/src/vegas.rs", title: "VegasGrid and the iteration combination"}
---

G. P. Lepage, "A new algorithm for adaptive multidimensional integration",
J. Comput. Phys. 27 (1978) 192–203. Pre-arXiv; the DOI resolves to it.

## The algorithm

1. Divide each dimension into `N_g` bins (typically about 1000), initially of
   equal width.
2. Map `x ∈ [a, b]` to `y ∈ [0, 1]` so that equal steps in `y` cover the
   variable-width bins in `x`; the Jacobian is `J(y) = N_g · Δx_{i(y)}`.
3. Draw points uniformly in `y`. Narrow bins receive as many points as wide
   ones, so points concentrate where the bins are narrow. This is importance
   sampling, not stratification.
4. After each iteration, reshape the bins: shrink them where `|f|` is large,
   widen them where it is small, aiming at equal `J²/Δxᵢ · ∫f²` per bin.
5. Repeat; combine the iterations' estimates weighted by `1/σᵢ²`.

The grid is separable, a product of one-dimensional densities, so it cannot
represent correlations between coordinates. VEGAS adapts to the propagator
peaks of a phase-space integrand (the `1/(p² − m²)²` near a resonance), but
only along the axes it is given. An `n`-body final state has `3n − 4`
independent coordinates[^n01-vegas][^n01-lips].

## Relevance to vibegraph

`vibegraph-lib/src/vegas.rs` is classic Lepage VEGAS: a separable grid with
Lepage's damped refinement, no stratification
([VEGAS integrator](../../phase-space/vegas-integrator.md)). Every integrand
reaches it through a phase-space map, and each multichannel term gets its own
grid because a separable density cannot follow a correlation between
coordinates ([per-channel grids](../../phase-space/per-channel-vegas-grids.md)).

Step 5 is where the code departs from the paper. Weighting iterations by
`1/σᵢ²` correlates the weights with the estimates and biases `σ` low at small
budgets, and turns a missed region into a confidently wrong answer; vibegraph's
default is the unweighted mean after two warm-up iterations, with
inverse-variance kept for golden comparisons
([iteration combination](../../phase-space/vegas-iteration-combination.md)).
Adapted grids also produce a heavy weight tail
([grid weight tail](../../phase-space/vegas-grid-weight-tail.md)). The
stratified extension is [VEGAS+](vegas-plus.md).

[^n01-vegas]: Note 01, classic VEGAS summary.
[^n01-lips]: Note 01, "Phase space integration".
