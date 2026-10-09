---
type: Paper
title: "Automated event generation for loop-induced processes"
description: "arXiv:1507.00020 (Hirschi, Mattelaer; JHEP 10 (2015) 146): automated loop-induced event generation; §2.2 summarises MadEvent's single-diagram-enhancement channels, survey/refine and helicity importance sampling."
resource: "https://arxiv.org/abs/1507.00020"
status: draft
tags: [madgraph, multichannel, phase-space, paper, loop-induced]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n01-loop, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L525-L561", title: "Note 01, loop-induced processes summary"}
  - {id: paper, resource: "https://arxiv.org/abs/1507.00020v3", title: "The paper itself (§2.1, §2.2, Appendix A.2), read in its ar5iv rendering"}
---

"Automated event generation for loop-induced processes" (Hirschi and
Mattelaer, 2015) is the first fully automated framework for cross sections and
events of loop-induced processes such as `g g → H`, inside MadGraph5_aMC@NLO.
Its new ingredients are a polynomial decomposition of the loop integrand and
phase-space strategies for high multiplicity[^n01-loop].

## What it says about phase space

§2.2 summarises the MadEvent algorithm the paper builds on and extends[^paper]:

- **Diagram enhancement.** The integral is split into a sum of integrals, each
  with the singularity structure of one Feynman diagram topology; each such
  integration channel gets a phase-space parametrisation that undoes that
  structure. Loop diagrams are contracted to tree topologies to seed channels
  (`max_npoint_for_channel` sets which loop topologies seed one).
- **Survey and refine.** The survey computes each channel's cross section to
  about 5%; the refine uses the survey's relative cross sections and
  efficiencies to generate the unweighted events.
- **Helicity importance sampling.** The paper adds Monte Carlo sampling over
  helicity configurations, with each configuration's probing frequency adapted
  to its share, uncorrelated with the kinematic grid.

Appendix A.2 documents `job_strategy = 0/1/2`, a parallelisation option for
high-multiplicity tree-level samples (two channels per job; one per job for
the highest multiplicity; the loop-induced algorithm for the highest and mode 1
for the next). §2.1 decomposes the loop numerator as a polynomial in the loop
momentum, so OPP reduction reuses its coefficients.

Note 01's summary also quotes an optimal multichannel weight `1/Σᵢ (1/Jᵢ)`;
that formula is not in the paper's text.

## Relevance to vibegraph

vibegraph's integrator also builds one channel per diagram, parametrised by its
propagator chain, but combines the channels as one α-weighted mixture driven by
VEGAS ([diagram channels](../../phase-space/diagram-channels.md),
[multichannel](../../phase-space/multichannel.md)). MadEvent instead integrates
each configuration in its own job with its own grid and multiplies `|M|²` by an
enhancement factor: the `AMP2` share (times a propagator factor) at
`sde_strategy = 1`, propagator products alone at `sde_strategy = 2`; see
[MadEvent's phase-space maps](../codebases/madevent-phase-space-maps.md) and
[single-diagram enhancement](../../phase-space/madevent-single-diagram-enhancement.md).
Cite those, not this paper, for what MadEvent's code does. The helicity
importance sampling is what `nhel = 1` would build
([nhel1-run-cards-refused](../../backlog/feature/nhel1-run-cards-refused.md)).

[^n01-loop]: Note 01, loop-induced processes summary.
[^paper]: arXiv:1507.00020 §2.2 and Appendix A.2 (ar5iv rendering of v3); journal reference from INSPIRE.
