---
type: Algorithm
title: "Convergence-targeted integration: --target-rel and the stop rule"
description: "integrate stops when the sum of channel variances, each widened by a pooled consistency factor max(1, emp/quoted), meets --target-rel (0.1% default) after min_iters, capped at 500 iterations."
status: draft
tags: [phase-space, vegas, stop-rule, target-rel, budget]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n31-i4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L243-L328", title: "Note 31 I4 (convergence-targeted integration)"}
  - {id: n32-s51, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L478-L597", title: "Note 32 §5.1 (S3: --target-rel becomes the default)"}
  - {id: n32-s54, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L749-L793", title: "Note 32 §5.4 (χ²/dof overflow on wide splits)"}
  - {id: n32-s7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L865-L955", title: "Note 32 §7 (time to target; the stop consumed the overflow)"}
  - {id: n34-w1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/34-draw-followup-plan.md#L145-L250", title: "Note 34 S1 Part C and S2 (stop-scale calibration; the llj ladder misread)"}
  - {id: n34-s3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/34-draw-followup-plan.md#L251-L325", title: "Note 34 S3 (few-accepted-point iterations drive the inflation)"}
  - {id: n34-ttt, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/34-draw-followup-plan.md#L338-L415", title: "Note 34 §3 (llj converges; cap raised to 500)"}
  - {id: n41-fa, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1823-L2005", title: "Note 41 F-A (a single spike holds the χ² stop)"}
  - {id: n41-p12, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2299-L2557", title: "Note 41 P12 (the pooled consistency factor)"}
  - {id: n41-m6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2702-L2915", title: "Note 41 M6 (a target run that never stopped)"}
measured:
  - {commit: 098c9e2, host: "M3 Max, macOS", command: "vibegraph integrate <proc card> --run-card <run card> --target-rel 0.001 -j 1, seeds 20260719-21"}
  - {commit: ef660f3, host: "4-core container shared with another session", command: "vibegraph integrate pp_to_ll_0j2j_mlm --target-rel 2e-3 --neval 200000 --max-iters 24"}
---

# Convergence-targeted integration

## The mode

`vibegraph integrate` runs in **convergence mode by default**: it iterates
until σ's relative uncertainty reaches `--target-rel` (default `1e-3`,
`vibegraph-cli/src/integrate.rs:126`). `--fixed-budget` instead spends exactly
`--neval × --niter` and stops; it is the mode a banked run is reproducible
under, since a target run's length depends on the variance it measures. Every
caller that wants a fixed budget (CLI tests, `generate_samples.sh`,
`scripts/acceptance.sh`, the validation gates) says so explicitly.[^n32-s51]

| Flag | Default | Role |
|---|---|---|
| `--target-rel` | `1e-3` | the relative error to reach |
| `--neval` | 120 000 | points per iteration (subject to the floors) |
| `--min-iters` | 6 | iterations before a stop is considered |
| `--max-iters` | 500 | safety cap; reaching it reports the run gave up, with the achieved δ |
| `--max-points` | 4e8 | evaluation cap over all channels and iterations |
| `--allocate` | `neyman` (target) / `by-alpha` (fixed) | [phase-space/channel-budget-allocation](channel-budget-allocation.md) |

`Budget::Target` and `StopReason` (`TargetMet`, `MaxIters`, `MaxPoints`,
`Aborted`, `Budget`) live in `vibegraph-lib/src/budget.rs`. The run is a hard
split, one grid per channel, so the rule is written per channel.

## Preconditions

Each exists because a stop is a claim about an error bar, and error bars here
are known to be optimistic:

1. **The combination must be unweighted.** Lepage's `1/σ²` weighting is biased
   by the correlation between an iteration's estimate and its own variance,
   and the bias comes with a small error bar: exactly what makes a convergence
   test stop early on a wrong number. A `Target` budget panics on
   `IterationCombination::InverseVariance` rather than read its error. The
   warm-up iterations are discarded before combining
   ([phase-space/vegas-iteration-combination](vegas-iteration-combination.md)).
2. **A minimum iteration count**, `max(min_iters, warmup + 2)` (`budget.rs:948`),
   so the consistency factor has at least two kept iterations to measure.
3. **A consistency factor** on each channel's variance, below.

## The stop test: a pooled consistency factor per channel

The run stops when

```
scaled_rel = √( Σ_j  quoted_j · max(1, emp_j / quoted_j) ) / Σ_j I_j  ≤  target_rel
```

(`scaled_rel`, `budget.rs:1209`; the factor is `pooled_scale`, `budget.rs:493`).
For channel `j`'s `k` kept iterations of `n_i` points, `W = Σ n_i`:

```
quoted = Σ n_i² σ_i² / W²                          (the variance the mean claims)
emp    = Σ n_i² (I_i − Ī)² / W² · k/(k − 1)          (the scatter it actually shows)
```

So the widened variance is `max(quoted, emp)`: a channel whose iterations
disagree by more than they claimed is charged its measured scatter, and never
more. It is the PDG scale-factor treatment applied per channel, where the
inconsistency lives, so one inconsistent channel widens its own term and not
the terms of channels that agree with themselves. One iteration gets factor 1;
two get `(I₁ − I₂)²/(σ₁² + σ₂²)`; on equal iterations with equal errors the
factor equals χ²/dof (pinned by `the_stop_scale_is_the_scatter_of_the_mean_over_its_quoted_variance`).
Zero-variance iterations need no filter: the pooled factor never divides by one
iteration's variance.

### Why not χ²/dof of each iteration against its own error

The obvious alternative, `max(1, χ²/dof)` with χ² formed against each
iteration's own `σ_i`, fails on heavy tails. Per-point weights of the σ-carrying channels have a Pareto tail index
near 2 ([phase-space/vegas-grid-weight-tail](vegas-grid-weight-tail.md)), so an
iteration that draws one point far out in the tail quotes both a shifted
integral *and* a large `σ_i`. A χ² against each iteration's own error then reads
that iteration as consistent and every quiet iteration as many of their own
small σ from the mean it shifted, and the factor decays only as `1/(n − 1)`.
On a matched `p p > e+ e- + 0,1,2 jets` target run, one point in a two-jet
channel carrying 0.004% of σ put that channel's χ²/dof at 5675; the χ²-scaled
error never fell below 2.4e-3 in 17 iterations (a 2e-3 run was still going at
32), while the five-seed fixed sweep scattered *less* than its quoted errors
(χ²/dof 0.42).[^n41-fa][^n41-m6] The pooled factor compares the spike with the
scatter it really adds to the mean: no channel exceeds 1.8 after iteration 5,
and the run stops at iteration 8 on the error bar the seed spread supports.
`a_single_spike_holds_the_chi2_stop_and_not_the_pooled_one` keeps the old rule
inside the test as the comparison.[^n41-p12]

Measured after the change, at `--target-rel 2e-3`: `pp_to_ll_0j2j_mlm` stops at
8–10 iterations (the χ² rule, after channel merging, at 11–15) with 1.4× fewer
points, and on seed 20260928 the stopped artifact is byte-equal to the
fixed-budget one; `pp_to_llj_mlm` stops at 6–9, within one iteration of the old
rule, identical σ where they stop together. A fixed budget is unaffected byte
for byte; only the stop decision, `ConvergenceReport::scaled_rel`, the progress
projection and the log line ("consistency-scaled") changed.

## Wide splits: what the stop reads, and what it still cannot do

- **The reported χ²/dof overflows by design.** `combine_kept` floors each
  iteration variance at `f64::MIN_POSITIVE`, so on hundreds of channels the
  reported per-channel χ²/dof reaches ~1e250. It is passed through as a report,
  not a statistic. The stop does **not** consume it: when it did, a
  579-channel target was unsatisfiable and always burned its cap.[^n32-s7][^n32-s54]
  `a_wide_split_with_empty_iterations_still_converges` pins both halves: the
  reported χ²/dof exceeds 1e100 while the run meets its target
  ([backlog](../backlog/performance/vegas-chi2-per-dof-overflows-on-wide-splits.md)).
- **Few-accepted-point iterations still inflate the factor.** On the `2 → 6`
  rows a channel iteration with one to three accepted points has a tiny but
  positive variance; there are no all-points-cut iterations. Under the χ²
  factor, `scaled_rel/achieved_rel` spanned ×3.4–×758 after the accepted-point
  floor, so a 1% target fires on some seeds and 0.2% cannot, while the plain
  quoted error tracked the seed spread.[^n34-w1][^n34-s3] This has not been
  re-measured under the pooled factor, and `budget.rs` has no minimum-accepted
  qualification ([backlog](../backlog/performance/stop-scale-inflated-by-few-accepted-iterations.md)).

## `pp_to_llj` converges, at 140–156 iterations

`pp_to_llj` at the default 0.1% target needs 140/156/145 iterations
(16.8–18.7M evaluations, three seeds), which is why `--max-iters` is 500: the
cap is a safety bound a converging run never touches, and a stock run stopped at
the same iteration with a SHA-256-identical artifact after the cap was raised.
At its achieved accuracy the row is at CPU parity with MadGraph; the remaining
cost is its map quality (9× MadGraph's points) times its χ²/dof ≈ 1.38 through
the scaled stop.[^n34-ttt] The per-row time-to-target tables are
[performance/integration-vs-madgraph](../performance/integration-vs-madgraph.md).

The row's σ ladder once appeared to climb with budget and was read as a
convergence defect. A 40-seed-per-rung ensemble put the estimator's
expectation flat from 150k up (the drift hypothesis dies at 7.3σ); the recorded
ladder was one five-seed draw 2.3σ low at its bottom rung, and five-seed scatter
understated the row's per-seed spread by 2–5×. That is the case behind the
rung-to-rung rule in `AGENTS.md`'s physics-validation section; the budgets
the gates use are [validation/budget-alignment-rule](../validation/budget-alignment-rule.md)
and [validation/seed-sweeps-and-budget-ladders](../validation/seed-sweeps-and-budget-ladders.md).

## Calibration

When the mode was introduced, 2 processes × 2 targets × 2 allocations × 8 seeds
met their target 64/64, seed χ²/dof 0.44–1.14, sd/quoted ≤ 1.07 (llj
over-covers by ~25%).[^n31-i4] That calibration predates the pooled factor; the
mixed-row target runs above agree with their ten-seed fixed sweep (+0.5σ).

[^n32-s51]: Note 32 §5.1 S3: `--target-rel` became the default mode.
[^n32-s7]: Note 32 §7: the stop consumed the χ²/dof overflow on wide splits (fixed in `61e578f`).
[^n34-w1]: Note 34 S1 Part C: the factor is not calibratable as a constant on wide splits; S2: the llj ladder misread.
[^n34-s3]: Note 34 S3: few-accepted-point iterations, not empty ones, drive the inflation.
[^n41-fa]: Note 41 F-A: one spike holds the χ² stop.
[^n41-m6]: Note 41 M6: a 2e-3 target run that never stopped.
[^n41-p12]: Note 41 P12: the pooled factor and its measurements.
[^n34-ttt]: Note 34 §3: llj converges at 140–156 iterations; the cap raised to 500.
[^n31-i4]: Note 31 I4: hard split, Neyman under a target, the 64/64 calibration.
[^n32-s54]: Note 32 §5.4: the overflow is expected on hundreds of channels.
