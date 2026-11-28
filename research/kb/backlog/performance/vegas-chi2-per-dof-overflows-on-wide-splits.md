---
type: Backlog Item
title: Per-iteration χ²/dof overflows to ~1e254 on wide channel splits
description: budget.rs floors a channel's variance at f64::MIN_POSITIVE, so the reported χ²/dof of hundred-channel rows overflows; σ is unaffected.
area: performance
state: open
priority: medium
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
estimator is fixed at its root. The few-accepted-point iterations once suspected of
sharing the cause do not occur under the pooled stop
([two-to-six-gate P1](../../sprints/two-to-six-gate/sessions/P1-report.md)); the statistic itself is the problem, see
[chi2-per-own-sigma-misreads-heavy-tails](../validation/chi2-per-own-sigma-misreads-heavy-tails.md).

Detail: [note 32 §5.4](../../history/notes/32-perf-addendum-plan.md).

## Seen on the CMS 4-jet cards (2026-11-28)

Card 252 (GJets-4Jets, 1068 grids) at `--fixed-budget --neval 120000 --niter 6`
on 2431b587 logs and records an iteration χ²/dof of 5.66e6, 1.04e7 and 4.57e6,
and `vg-validator` saw channels with σ = 0 and δσ ≈ 5.8e-146 in the record.
The run's per-iteration totals cannot confirm the value independently: the
`iteration N` log lines are running combinations of the kept iterations (the
last equals the final σ), and inverting them is ill-posed (one seed's running
error rises between iterations, which implies a negative weight). No log line
or record field carries each kept iteration's own total, so a reader cannot
compute the run-level χ² at all.

The reported value is `running_estimate`'s mean of the channels' own χ²/dof
(`vibegraph-lib/src/budget.rs`), so a single channel with a near-zero quoted
variance sets it. The same number goes into the `iteration` log line, the
progress records and the result record, so any reader of `chi2_per_dof` on a
many-channel CMS card sees noise, not a diagnostic. Raised to medium for that
reason: the campaign's seed and budget readings have no usable iteration χ²
on these cards, and the per-iteration totals that would replace it are not
recorded.
