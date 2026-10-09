---
type: Paper
title: "VEGAS+: adaptive stratified sampling"
description: "arXiv:2009.05112 (Lepage 2020): adds adaptive stratification on top of the VEGAS importance grid; vibegraph implements only the classic grid, and the gain is unmeasured here."
resource: "https://arxiv.org/abs/2009.05112"
status: draft
tags: [vegas, stratification, integration, paper, monte-carlo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n01-vegasplus, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/01-paper-summaries.md#L155-L182", title: "Note 01, VEGAS entry and VEGAS+ additions"}
  - {id: n01-lips, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/01-paper-summaries.md#L183-L188", title: "Note 01, phase-space integration"}
---

G. P. Lepage, "Adaptive multidimensional integration: VEGAS enhanced"
(2020). It keeps the [VEGAS](vegas.md) importance-sampling grid and adds
adaptive stratified sampling on top[^n01-vegasplus]:

- the unit hypercube is divided into `N_s` stratification cells per dimension;
- each iteration allocates more integrand evaluations to cells with larger
  variance, and adapts that allocation as it goes.

This targets what a separable grid cannot follow: integrands with several
peaks, or peaks along diagonals. The paper reports 2–19× improvement over
classic VEGAS on such problems.

## Relevance to vibegraph

vibegraph's integrator is classic VEGAS with no stratification
([VEGAS integrator](../../phase-space/vegas-integrator.md)). Two things already
cover part of what VEGAS+ addresses: the multichannel decomposition gives each
propagator structure its own map and grid, which handles most diagonal
structure, and the evaluation budget is already allocated across channels by
variance ([channel budget allocation](../../phase-space/channel-budget-allocation.md)).
Whether within-channel stratification still pays is unmeasured; it is open as
[vegas-plus-stratification-unmeasured](../../backlog/feature/vegas-plus-stratification-unmeasured.md),
to be decided statistically (seed sweep and `χ²/dof` at matched points), since
stratification changes the sampling order and cannot be compared bit for bit
against banked results.

[^n01-vegasplus]: Note 01, "VEGAS+ additions".
