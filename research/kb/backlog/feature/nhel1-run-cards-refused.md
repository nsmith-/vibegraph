---
type: Backlog Item
title: Helicity Monte Carlo (nhel = 1) is not implemented
description: "nhel = 1 (one helicity configuration per point, drawn from an adapted distribution) is refused; it is in scope, and it gates wpwm_to_wpwmz_cw's σ and events."
area: feature
state: open
priority: medium
closes_when: "nhel = 1 cards integrate and generate with MadEvent's helicity sampling, gated by seeded σ against a MadEvent nhel = 1 reference, and wpwm_to_wpwmz_cw's integrals and samples cells are measured."
blocked_by: []
opened: 2026-09-07
tags: [run-card, helicity-sampling, coverage, in-scope]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L464-L470", title: "TODO.md entry T032"}
---
In scope (user, 2026-10-09). `vibegraph-lib/src/runcard.rs` lists `nhel` with
default 0 and refuses any other value as `UnsupportedField`, which is right
until the feature exists: it changes both the estimator and the per-event
weight.

**What MadEvent does** (`matrix_madevent_group_v4.inc`, MadGraph 3.7.1). An
initialisation phase sums every helicity configuration to find the non-zero
ones (`GOODHEL`) and seeds a discrete `Helicity` grid (`DiscreteSampler`).
After that, each phase-space point evaluates **one** configuration,
`HEL_PICKED`, drawn by `genps` from that grid, which adapts during the survey
like a VEGAS dimension. |M|² is multiplied by `hel_jacobian` (the inverse of
the pick probability), so the estimator stays unbiased. The event's helicities
are the picked configuration. It is cheaper per point and noisier per point
than the explicit sum; whether it pays depends on the process
([note 41](../../history/notes/41-completeness-trace-msq-feasibility.md) names helicity
sampling as the cheaper lever for the trace-form study).

**Where it matters now.** For the 222-diagram `wpwm_to_wpwmz_cw`, MadGraph
itself chose `nhel = 1`, so the reference's banked σ and its 10000 events are
unreachable from the card that produced them, and the row's `integrals` and
`samples` cells are `uncovered`. Even once reachable, both cells stay
informational until the row's amplitudes agree; see
[wpwmz-cw-ow-five-vector-residual](../validation/wpwmz-cw-ow-five-vector-residual.md).
