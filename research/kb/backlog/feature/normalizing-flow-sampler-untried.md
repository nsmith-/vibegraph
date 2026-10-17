---
type: Backlog Item
title: A normalizing-flow channel sampler is untried, and needs an ML dependency decision
description: "Flows (i-flow, MadNIS) learn correlated per-channel densities a VEGAS grid cannot; trying one means choosing a Rust ML stack for training and inference, which is the user's call."
area: feature
state: needs-user
priority: low
closes_when: "The user has chosen the ML dependency (or offline training with in-tree inference), a flow sampler kind runs through --sampler, and it is measured against default VEGAS and the bin-width-bounded grid PR #48 adds (or whatever default that work leaves) on the comparison harness's rows (≥5 seeds; 20+ on the deciding pair), recorded as adopt or reject."
blocked_by: [sampler-hardwired-to-vegas-grid, artifact-has-no-sampler-kind, integrate-cannot-select-a-sampler, sampler-comparison-harness-missing]
opened: 2026-10-17
tags: [normalizing-flows, machine-learning, sampler, research, dependencies, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: iflow, resource: "../../references/papers/i-flow.md", title: "i-flow, arXiv:2001.05486"}
  - {id: madnis, resource: "../../references/papers/madnis.md", title: "MadNIS, arXiv:2212.06172"}
  - {id: nis, resource: "https://arxiv.org/abs/2001.05478", title: "Bothmann et al., Exploring phase space with Neural Importance Sampling, SciPost Phys. 8, 069 (2020)"}
---
The user named normalizing-flow samplers (2026-10-17). A flow is a trained
bijection on the hypercube whose density is known everywhere, so it can stand
in for a channel's grid and learn the correlations a separable grid cannot
([i-flow](../../references/papers/i-flow.md); neural importance sampling,
Bothmann et al., arXiv:2001.05478). [MadNIS](../../references/papers/madnis.md)
also learns the channel weights, which is
[alpha-grid-alternation-unstudied](alpha-grid-alternation-unstudied.md)'s
question in learned form.

**The user's decision: the dependency.** The crate has no ML stack. The options
to weigh, not decided here: a native Rust framework that trains and infers in
process; bindings to a C++ runtime (heavy build and distribution cost); or
training offline in Python and shipping weights the crate evaluates in tree.
What each costs in build time, binary size, platform support and determinism
belongs in the decision record.

What any choice must meet:

- **Determinism.** `-j` must not move an artifact's bytes: inference in a fixed
  batch order on CPU, training a function of the points in point order.
- **Lanes.** The seam is generic over `F: Real` with a batch draw
  ([sampler-hardwired-to-vegas-grid](sampler-hardwired-to-vegas-grid.md)); a
  flow's forward pass is the natural batch consumer.
- **Training cost** is reported apart from the evaluations after training, and
  counts against variance×CPU on the harness.
