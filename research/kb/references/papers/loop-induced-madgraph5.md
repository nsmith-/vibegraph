---
type: Paper
title: Loop-induced processes and phase-space optimisation in MG5
description: "arXiv:1507.00020 (Hirschi, Mattelaer): automated loop-induced event generation; its appendix describes MadGraph's per-diagram multichannel phase-space strategy."
resource: "https://arxiv.org/abs/1507.00020"
status: draft
tags: [madgraph, multichannel, phase-space, paper, loop-induced]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n01-loop, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/01-paper-summaries.md#L525-L561", title: "Note 01, loop-induced processes summary"}
---

"Automated event generation for loop-induced processes" (Hirschi and
Mattelaer, 2015) is the first fully automated framework for cross sections and
events of loop-induced processes such as `g g → H`, inside MadGraph5_aMC@NLO.
Its new ingredients are a polynomial decomposition of the loop integrand and
phase-space strategies for high multiplicity[^n01-loop].

## What the notes take from it

The appendix describes MadGraph's phase-space integration, which applies to
tree-level processes too:

- **Channels from diagrams.** Each Feynman diagram defines a phase-space
  channel whose parametrisation maps its propagator peaks to flat coordinates.
- **Multichannel weights.** Samples are drawn from the mixture of channels;
  the paper's combined weight is `1/Σᵢ (1/Jᵢ)` over the channel Jacobians,
  which minimises the variance.
- **Job strategies** (`job_strategy = 0/1/2`) decide how survey and refine
  steps are distributed over channels.
- **Polynomial decomposition** (Eq. 2.6): the numerator `N(t)` is expanded in
  powers of the loop variable `t`, so OPP reduction does not re-evaluate the
  full integrand.

## Relevance to vibegraph

vibegraph's integrator follows this design: one channel per diagram,
parametrised by its propagator chain, combined as a mixture and driven by VEGAS
([diagram channels](../../phase-space/diagram-channels.md),
[multichannel](../../phase-space/multichannel.md)).

The appendix's mixture weight is a description of the method, not of
MadEvent's code. MadEvent integrates one configuration per job directory, each
with its own grid, and divides `|M|²` among configurations by `AMP2` ratios
(`sde_strategy = 1`) or propagator products (`sde_strategy = 2`); see
[MadEvent's phase-space maps](../codebases/madevent-phase-space-maps.md) and
[single-diagram enhancement](../../phase-space/madevent-single-diagram-enhancement.md).
Cite those, not this paper, for what MadEvent does.

[^n01-loop]: Note 01, loop-induced processes summary.
