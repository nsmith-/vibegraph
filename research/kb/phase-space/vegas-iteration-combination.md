---
type: Design Decision
title: VEGAS iterations combine by unweighted mean after a warm-up
description: "Lepage's 1/σ² iteration weighting correlates weights with estimates and biases σ low at small budgets; default is the arithmetic mean with warm-up 2, InverseVariance kept for goldens."
status: draft
tags: [vegas, bias, integration, statistics, convergence]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n24-p3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1671-L1717", title: "Note 24 P3, the five-seed sweep was not sufficient: the budget scan"}
  - {id: n24-p4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1934-L1953", title: "Note 24 P4, the sample's own σ does not inherit the integrator's bias"}
  - {id: n31-i1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L49-L109", title: "Note 31 I1, VEGAS first-iteration convergence bias (plan and measured result)"}
  - {id: n32-s1-plan, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L164-L353", title: "Note 32 Wave 1, I5 combine_seeds unweighted (plan)"}
  - {id: n32-s1-out, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L478-L597", title: "Note 32 §5.1, per-session outcomes (combine_seeds moved)"}
  - {id: n41-fa, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1617-L2005", title: "Note 41 M5 / F-A, MadEvent's iteration combination as read"}
  - {id: n34-s2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/34-draw-followup-plan.md#L145-L250", title: "Note 34 S2, MadGraph's last-three combination on the recarded llj row"}
  - {id: vegas-rs, resource: "vibegraph-lib/src/vegas.rs#L20-L135", title: "vegas.rs, 'Combining the iterations', DEFAULT_WARMUP_ITERS, IterationCombination"}
  - {id: budget-rs, resource: "vibegraph-lib/src/budget.rs", title: "ChannelHistory::combine, Target budget refusal of InverseVariance"}
  - {id: mg-dsample-comb, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/dsample.f#L296-L332", title: "MadEvent dsample.f, last-three-iteration combination"}
measured:
  - {commit: ea58ab9, landed_in: e99b05c, command: "5 seeds × 75k/150k/300k/600k points per iteration on the llj rows, before/after, same host"}
---

## Decision

`VegasGrid::adapt` combines its per-iteration estimates with
`IterationCombination::Unweighted`, the `#[default]`: the arithmetic mean of
the kept iterations, quoting `√(Σᵢ σᵢ²)/n`. The first `DEFAULT_WARMUP_ITERS
= 2` iterations still draw points and still refine the grid; only their
estimates are left out (`vegas.rs:114-135`)[^vegas-rs].
`IterationCombination::InverseVariance` is Lepage's `1/σ²` mean and reproduces
the old behaviour bit for bit, which is how pinned-seed goldens keep their
bytes. Both knobs are `#[serde(skip)]`, and the trained grid is untouched by
either (a bin-edge equality test pins it), so event samples drawn from a grid
do not depend on the rule[^n31-i1].

## Why not `1/σ²`

Each iteration's estimate is unbiased whatever grid it ran on. The `1/σ²`
combination is not, because each iteration's weight is estimated from the same
points that produced its integral. An iteration that undersamples the peak
returns a low integral *and* a low variance, and is weighted up. The bias is
`O(1/N)` in the per-iteration sample count, and at the per-channel budgets a
multichannel integration can afford it dominates the error[^vegas-rs].

The controlled measurement is in the module doc: a 5-dimensional product of
Gaussians with a known integral, 4 000 seeds, `niter = 10`:

| points/iter | rule, warm-up | mean rel | RMS rel | mean quoted |
|---|---|--:|--:|--:|
| 2 000 | `1/σ²`, 0 | **−1.21%** | 8.2% | 0.72% |
| 2 000 | `1/σ²`, 2 | **−1.40%** | 9.3% | 0.76% |
| 2 000 | unweighted, 2 | +0.14% | 10.7% | 1.70% |
| 10 000 | `1/σ²`, 0 | **−0.024%** | 0.187% | 0.182% |
| 10 000 | unweighted, 2 | −0.0004% | 0.193% | 0.191% |

Bold entries sit 8–9 standard errors from zero. Two readings set the default:

- **Discarding warm-up iterations does not remove the bias.** The
  estimate–weight correlation is in every iteration, and dropping the early
  ones makes it marginally worse. The combination rule is the lever.
- **The warm-up buys variance, not bias.** Under the unweighted mean the
  unconverged early iterations enter at full weight; at 10 000 points the RMS
  spread is 0.53% at warm-up 0 and 0.19% at 2, with a flat minimum at one or
  two and a slow rise past it. Two rather than one because the harder
  7-dimensional configurations put the minimum there.

The unweighted error is better calibrated but still an underestimate where the
grid is starved (1.70% quoted against a 10.7% spread at 2 000 points), so a
convergence stop must not read it alone: [stop rule](convergence-stop-rule.md)
widens it by an iteration-consistency factor, and a `Budget::Target` run
refuses an `InverseVariance` grid outright (`budget.rs`)[^budget-rs].

## How it was found on a real integrand

On `p p > l+ l- j` at `lpp = 1` (24 pooled channels, 7-dimensional grids) a
budget of 60 000 points per iteration passed a single-seed check, a five-seed
check and the seed-scatter check while sitting 1.0% below MadGraph:[^n24-p3]

| points/iter | 5-seed mean (pb) | rel vs MG | χ²/dof |
|---|---|---|---|
| 60 000 | 418.476 ± 0.438 | **−1.03%** | 1.55 |
| 150 000 | 421.658 ± 0.270 | −0.28% | 0.47 |
| 300 000 | 422.850 ± 0.189 | +0.002% | 1.90 |
| 600 000 | 423.524 ± 0.133 | +0.16% | 0.37 |

The five seeds at 60 000 were mutually consistent and collectively wrong. The
steps halve as the budget doubles, the `O(1/N)` signature. A seed sweep
detects a seed that missed a region; it cannot detect a bias every seed shares
([seed sweeps and budget ladders](../validation/seed-sweeps-and-budget-ladders.md)).
The integrand was not the cause: the same combination ran the fixed-beam path,
and llj showed it first because many channels split the budget thinly.

The event sample confirmed the diagnosis independently[^n24-p4]. The
accept/reject pass is one pass over frozen grids and never goes through the
iteration combination, so its mean `XWGTUP` converged to the true σ while the
banked integral was still low: +1.25% above the banked σ at 100 000 points per
iteration (four of five seeds on the same side), −0.07% at 300 000.

With the unweighted default, the llj ladder spans collapsed (`pp_to_llj`
2.09% → 0.46%, `llj_fixed` 0.80% → 0.12%, `llj_dyn` 0.76% → 0.06%) and the
gates' budgets were cut; rows with wide channel sets and no climb (`jj`, `bb`)
did not move[^n31-i1].

## The same rule at the other levels

- **Seeds.** `combine_seeds` (`validate_hadronic.rs:423`) takes the
  unweighted mean of seed results, `err = √(Σᵢ σᵢ²)/n`, with χ² about that
  mean. Seeds run equal budgets, so the move was second order: every hadronic
  row shifted by ≤ 0.02%, inside its tolerance[^n32-s1-out].
- **Iterations of different size.** A per-channel history under a varying
  allocation (`budget.rs`, `ChannelHistory::combine`) uses the rule when all
  kept iterations drew the same number of points, and otherwise weights by
  point count. A count is fixed before the iteration's points exist, so it
  cannot correlate with the estimate it weights; over 8 seeds at four
  process/target settings it differs from the plain mean by 0.26–0.40σ with
  no common sign[^budget-rs].

## MadEvent's rule, for comparison

MadEvent combines only the **last three** iterations, weighting each by
`xmean²/xsigma²` (`dsample.f:303-305`), forms χ² over those three with
`n − 1 = 2`, and widens the error by `√χ²` when χ² > 1
(`dsample.f:332`):[^mg-dsample-comb][^n41-fa]

```fortran
tmean = tmean+xmean(i)*xmean(i)**2/xsigma(i)**2
...
if (chi2 .gt. 1) tsigma=tsigma*sqrt(chi2)
```

That weight is also estimated from the iteration's own points. On the
recarded `pp_to_llj` row it pulls MadGraph's own result down 0.146% by
down-weighting the iterations that caught the weight tail; our converged value
is +0.09% from MadGraph's iterations recombined by point count[^n34-s2].
How MadEvent's integration, truncation and unweighting fit together is in
[unweighting](../events/unweighting.md).

Related: [VEGAS integrator](vegas-integrator.md).

[^vegas-rs]: `vibegraph-lib/src/vegas.rs`, module doc "Combining the iterations" and the `DEFAULT_WARMUP_ITERS` doc.
[^n31-i1]: Note 31 I1, the plan and its measured result.
[^budget-rs]: `vibegraph-lib/src/budget.rs`, module doc "Stopping" and `ChannelHistory::combine`.
[^n24-p3]: Note 24 P3, the budget scan on `p p > l+ l- j`.
[^n24-p4]: Note 24 P4, sample σ against banked σ.
[^n32-s1-out]: Note 32 §5.1, `combine_seeds` moved to the unweighted mean.
[^mg-dsample-comb]: MadEvent `dsample.f` at the pinned commit.
[^n41-fa]: Note 41 F-A, "MadEvent's behaviour".
[^n34-s2]: Note 34 S2 close-out, the recarded llj ladder.
