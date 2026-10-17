---
type: Paper
title: "i-flow: integration and sampling with normalizing flows"
description: "arXiv:2001.05486 (Gao, Isaacson, Krause; Mach. Learn.: Sci. Technol. 2020): a Python package integrating and sampling with normalizing flows; the reference flow integrator."
resource: "https://arxiv.org/abs/2001.05486"
status: draft
tags: [machine-learning, normalizing-flows, integration, paper, monte-carlo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: iflow, resource: "https://arxiv.org/abs/2001.05486", title: "C. Gao, J. Isaacson, C. Krause, i-flow: High-dimensional Integration and Sampling with Normalizing Flows, Mach. Learn.: Sci. Technol. (2020), doi:10.1088/2632-2153/abab62"}
---

C. Gao, J. Isaacson and C. Krause, "i-flow: High-dimensional Integration and
Sampling with Normalizing Flows" (2020). A normalizing flow is a trained
bijection between a simple distribution and the target, so it is both a
sampler and a density evaluable at any point. i-flow is a Python package that
uses one for numerical integration and for sampling from high-dimensional
distributions, and the authors report it outperforming other integration
algorithms on high-dimensional correlated integrals[^iflow].

## Relevance to vibegraph

A flow on the unit hypercube would take a channel grid's place, learning the
correlations a separable VEGAS grid cannot. It brings a machine-learning
dependency into a crate that has none, which is a design decision rather than a
detail: [normalizing-flow-sampler-untried](../../backlog/feature/normalizing-flow-sampler-untried.md).
The multichannel form is [MadNIS](madnis.md).

[^iflow]: arXiv:2001.05486, abstract.
