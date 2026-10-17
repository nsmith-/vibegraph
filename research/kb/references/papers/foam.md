---
type: Paper
title: "FOAM: a cellular self-adapting Monte Carlo generator"
description: "physics/0609068 (Jadach, Sawicki 2006; FOAM 2.06) and physics/0203033 (Jadach 2003): binary cell splitting driven by max/mean weight or variance; a candidate per-channel sampler, untried here."
resource: "https://arxiv.org/abs/physics/0609068"
status: draft
tags: [foam, cellular-sampling, integration, paper, monte-carlo, unweighting]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: foam206, resource: "https://arxiv.org/abs/physics/0609068", title: "S. Jadach, P. Sawicki, Upgrade of the Cellular General Purpose Monte Carlo Tool FOAM to version 2.06 (IFJPAN-V-05-05)"}
  - {id: foam, resource: "https://arxiv.org/abs/physics/0203033", title: "S. Jadach, Foam: A General-Purpose Cellular Monte Carlo Event Generator, Comput. Phys. Commun. 152 (2003) 55"}
---

S. Jadach and P. Sawicki, "Upgrade of the Cellular General Purpose Monte Carlo
Tool FOAM to version 2.06" (2006), which updates S. Jadach, "Foam: A
General-Purpose Cellular Monte Carlo Event Generator", Comput. Phys. Commun.
152 (2003) 55.

## The algorithm

FOAM combines importance and stratified sampling[^foam206]. It divides the
integration domain into a "foam" of cells, which may be hyperrectangles,
simplices or Cartesian products of both, by repeated binary splits during an
**exploration phase**. Each split chooses the cell, and the position and
direction of the dividing plane, so as to minimise the ratio of maximum to mean
weight (for generating events) or the weight variance over the mean (for
integrals)[^foam][^foam206]. Each cell is explored by Monte Carlo before it is
split. Generation then picks a cell and samples uniformly inside it, giving
weight-one events or weighted ones.

The 2003 paper states that adaptive integration and simulation work on a
personal computer for a relatively small number of dimensions, at most 16, a
limit it expects to rise with CPU power[^foam]. Version 2.06 removes the
simplex memory cost that grew as `n · N_cells` (a problem past about 10⁶ cells
and `n > 10`); the hyperrectangular foam already stored a constant amount per
cell, and the ROOT-integrated mFOAM uses hyperrectangles only[^foam206].

## Relevance to vibegraph

Unlike VEGAS's separable grid ([VEGAS](vegas.md)), a cell tree can follow
correlations between coordinates, and its event-generation objective is the
max/mean weight ratio that sets unweighting efficiency here
([grid weight tail](../../phase-space/vegas-grid-weight-tail.md)). It would sit
in place of a channel's grid, on the residual each channel's map leaves. A
hadronic `2 → n` channel has `2 + (3n − 4)` coordinates, so the `2 → 6` rows
reach 16. Untried here:
[foam-sampler-untried](../../backlog/feature/foam-sampler-untried.md).

[^foam206]: physics/0609068, abstract and §1–2.
[^foam]: physics/0203033, abstract.
