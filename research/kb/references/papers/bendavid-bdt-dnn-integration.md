---
type: Paper
title: "Monte Carlo integration with boosted decision trees and generative networks"
description: "arXiv:1707.00028 (Bendavid 2017): a FOAM-like box density from a non-negative boosted tree ensemble, and a generative DNN; 4- and 9-dimensional camel tests against VEGAS and FOAM."
resource: "https://arxiv.org/abs/1707.00028"
status: draft
tags: [machine-learning, boosted-decision-trees, integration, paper, monte-carlo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: bendavid, resource: "https://arxiv.org/abs/1707.00028", title: "J. Bendavid, Efficient Monte Carlo Integration Using Boosted Decision Trees and Generative Deep Neural Networks (2017)"}
---

J. Bendavid, "Efficient Monte Carlo Integration Using Boosted Decision Trees
and Generative Deep Neural Networks" (2017). Two importance samplers for
integrands that do not factorise into one-dimensional densities[^bendavid].

## Generative boosted trees

A single binary tree is a FOAM-like density: non-overlapping boxes `θⱼ` with
weights `aⱼ`, sampled by choosing a box with probability `∝ Vⱼaⱼ` and then
uniformly inside it ([FOAM](foam.md)). The paper replaces it with a boosted
ensemble, whose boxes overlap; the density stays sampleable as long as every
`aᵢⱼ` is non-negative, which the loss is built to guarantee (§2): squared error
in log space, `Σ (ln f − ln g)²` (eq. 2.7), taken against the residual left by
earlier trees (eq. 2.8). Because a
later tree cannot correct an earlier one's overshoot, a second, ordinary
regression ensemble `h` with `e^h ≈ f` is trained alongside and stands in for
`f` in the generative ensemble's loss. Training alternates: sample from the
current model, evaluate `f`, add trees, repeat until the integral stops moving.
A "staged" mode integrates by sampling `e^h`, with the generative ensemble used
only for a secondary unweighting.

## Generative network

A network maps a prior sample `z` to `x` of the same dimension, trained on a
KL divergence whose density term is the Jacobian determinant of the map. Its
density is known only at points it generated (where `z` is known); evaluating it
anywhere else needs the inverse, whose existence is not studied (§3.2).

## Results

On the camel function (two Gaussians on the hypercube's diagonal), the ratio
`σ_w/⟨w⟩` after training (Tables 1–2)[^bendavid]:

| dims | VEGAS | FOAM | generative BDT | BDT, staged | generative DNN | DNN, staged |
|---|---|---|---|---|---|---|
| 4 | 2.82 (3·10⁵ evals) | 0.319 (3.9·10⁶) | 0.082 (3·10⁵) | 0.077 (3·10⁵) | 0.083 (2.9·10⁵) | 0.030 (2.9·10⁵) |
| 9 | 19 (1.5·10⁶) | — | 0.63 (3.2·10⁶) | 0.31 (3.2·10⁶) | 0.15 (2.9·10⁵) | 0.081 (2.9·10⁵) |

"Staged" integrates by sampling the regression model `e^h`, with the generative
model used only for a secondary unweighting. The 9-dimensional BDT figures take
3.2·10⁶ evaluations to VEGAS's 1.5·10⁶.

The networks scale better with dimension than the trees. The DNN is
implemented in TensorFlow/Keras, and the paper notes the log-determinant had no
GPU implementation at the time.

## Relevance to vibegraph

The test integrand is unmapped; here each channel's map has already flattened
the peaks it knows about, so the gain over a per-channel VEGAS grid is an open
question, as it is for [VEGAS+](vegas-plus.md). The tree sampler is open as
[boosted-tree-sampler-untried](../../backlog/feature/boosted-tree-sampler-untried.md);
the network approach is subsumed by
[normalizing-flow-sampler-untried](../../backlog/feature/normalizing-flow-sampler-untried.md),
whose flows have a density computable everywhere.

[^bendavid]: arXiv:1707.00028, §2–4, Tables 1 and 2.
