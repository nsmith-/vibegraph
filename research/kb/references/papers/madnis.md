---
type: Paper
title: "MadNIS: neural multi-channel importance sampling"
description: "arXiv:2212.06172 (Heimel et al.; SciPost Phys. 15, 141 (2023)): learned multichannel weights combined with a normalizing flow per channel, with buffered training; shown on Drell–Yan."
resource: "https://arxiv.org/abs/2212.06172"
status: draft
tags: [machine-learning, normalizing-flows, multichannel, integration, paper, madgraph]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: madnis, resource: "https://arxiv.org/abs/2212.06172", title: "T. Heimel, R. Winterhalder, A. Butter, J. Isaacson, C. Krause, F. Maltoni, O. Mattelaer, T. Plehn, MadNIS - Neural Multi-Channel Importance Sampling, SciPost Phys. 15, 141 (2023)"}
---

T. Heimel, R. Winterhalder, A. Butter, J. Isaacson, C. Krause, F. Maltoni,
O. Mattelaer and T. Plehn, "MadNIS — Neural Multi-Channel Importance
Sampling", SciPost Phys. 15, 141 (2023). It combines machine-learned
multichannel weights with a normalizing flow for importance sampling, in a
bi-directional setup on an invertible network with online and buffered
training for expensive integrands, and demonstrates it on Drell–Yan with an
additional narrow resonance[^madnis].

## Relevance to vibegraph

It is the same arrangement as vibegraph's — a channel decomposition with an
adaptive density per channel and adapted channel weights
([multichannel](../../phase-space/multichannel.md)) — with both adaptive parts
learned. The per-channel flow is the
[normalizing-flow sampler](../../backlog/feature/normalizing-flow-sampler-untried.md);
learning the channel weights jointly with the densities is the question
[alpha-grid-alternation-unstudied](../../backlog/feature/alpha-grid-alternation-unstudied.md)
asks of the classical pair. For the flow integrator itself see [i-flow](i-flow.md).

[^madnis]: arXiv:2212.06172, abstract.
