---
type: Backlog Item
title: Exact stratified-parallel axes for the VEGAS+α loop are unbuilt
description: Helicity strata, flavour-group × beam-ordering strata and a shorter adapt phase before the parallel frozen pass remain open as parallel axes.
area: performance
state: open
priority: low
closes_when: Each exact axis (helicity strata, flavour groups × beam orderings, frozen-pass share) is built and measured on wall time and σ spread, or recorded as not worth building.
blocked_by: []
opened: 2026-08-01
tags: [parallelism, stratification, vegas, helicity, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1119-L1135", title: "TODO.md entry T103"}
---
The user asked (2026-08-01) for embarrassingly parallel axes in the iterative
VEGAS+α loop, exact ones first, for SIMD and multi-thread promotion.
Channel-block stratification (note 31 §I4) and the batch-size-vs-iteration-count
measurement (note 32 S3) exist. Still open, all exact:
- **Helicity strata.** `Σ_hel |M_hel|²` is an exact orthogonal decomposition for
  unpolarized beams, so parity-folded helicity classes can carry their own
  budgets and grids. This would be the first real consumer of the single-helicity
  MadGraph benchmark
  ([mg-single-helicity-bench-no-consumer](mg-single-helicity-bench-no-consumer.md)).
- **Flavour groups × beam orderings.** These are already independent integrals,
  and both integrands are `Sync`.
- **Frozen-pass bulk.** `sample_frozen` is already embarrassingly parallel, so
  the lever is keeping the sequential adapt phase short.

Partition-based axes are second tier: per-diagram AMP2 shares (MadEvent's
G-directories) and per *distinct* map class. They have cluster-scale precedent
but carry the routing fragility, and they need the same coverage guardrail as
per-flow α tuning
([per-flow-alpha-gain-unmeasured](per-flow-alpha-gain-unmeasured.md)).

Detail: [note 31 §I4](../../31-perf-sprint-3-plan.md),
[note 32 §5.4](../../32-perf-addendum-plan.md).
