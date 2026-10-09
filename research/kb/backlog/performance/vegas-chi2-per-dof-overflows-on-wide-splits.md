---
type: Backlog Item
title: Per-iteration χ²/dof overflows to ~1e254 on wide channel splits
description: budget.rs floors a channel's variance at f64::MIN_POSITIVE, so the reported χ²/dof of hundred-channel rows overflows; σ is unaffected.
area: performance
state: open
priority: low
closes_when: The combination code reports a finite, meaningful χ²/dof on the 2→6 rows (an estimator fix, not a clamp), or the value is replaced by a statistic that does not divide by a floored variance.
blocked_by: []
opened: 2026-08-05
tags: [vegas, budget, chi2, reporting, two-to-six]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1076-L1098", title: "TODO.md entry T100"}
---
`ChannelHistory::combine_kept` (`vibegraph-lib/src/budget.rs`, around line 454)
divides each kept iteration's squared deviation by `v.max(f64::MIN_POSITIVE)`.
On splits with hundreds of channels, some channel iterations have (near-)zero
variance, and the per-iteration χ²/dof reaches ~1e254. The 2→6 `integrals`
re-run printed values over 10^250 on both rows. This is a reported statistic
only, and σ is unaffected. The manifest's cell notes say it is not a statistic
on these rows.

The pass-through was a deliberate decision: clamping would hide the cause. Treat
the overflow as expected on any row with hundreds of channels until the
estimator is fixed at its root. It may share a cause with the
few-accepted-point iterations that inflate `stop_scale` (see
[stop-scale-inflated-by-few-accepted-iterations](stop-scale-inflated-by-few-accepted-iterations.md)).

Detail: [note 32 §5.4](../../history/notes/32-perf-addendum-plan.md).
