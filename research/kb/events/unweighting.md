---
type: Algorithm
title: "Unweighting: accept/reject over per-channel grids and the truncated maximum"
description: "Draw a channel ∝ w_max_j, accept at w/w_max_j, keep overweights at weight > 1 (efficiency ≤ σ/Σ w_max_j); w_max from MadGraph unwgt.f's truncation ladder, because the scanned extremum never converges."
status: draft
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
tags: [events, unweighting, accept-reject, vegas, overweights]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n21-grids, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/21-resonance-sampling-and-events-plan.md#L382-L511", title: "Note 21 addendum, one grid per channel and per-channel w_max"}
  - {id: n23-e2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L281-L343", title: "Note 23 E2, accept/reject"}
  - {id: n23-grids, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L898-L943", title: "Note 23, per-channel VEGAS grids before E2"}
  - {id: n24-wmax, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1907-L1933", title: "Note 24 P4, w_max against budget"}
  - {id: n24-bias, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1934-L1953", title: "Note 24 P4, the sample's σ does not inherit the integrator's bias"}
  - {id: n31-i2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L110-L151", title: "Note 31 I2, the maxima never converge"}
  - {id: n32-plan, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L164-L353", title: "Note 32 wave 1, w_max from a percentile"}
  - {id: n32-out, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L478-L597", title: "Note 32 §5.1, the truncation rule's measured effect"}
  - {id: n41-m5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1617-L2005", title: "Note 41 M5 and F-A, MadEvent's unweighting and the weight tail"}
  - {id: mg-unwgt, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/3.7.1/Template/LO/SubProcesses/unwgt.f#L355-L446", title: "MadGraph 3.7.1 unwgt.f, trunc_max ladder and overweights"}
---

# Unweighting

`unweight::Unweighter` (`vibegraph-lib/src/unweight.rs`) turns a converged
multichannel integration into unweighted events. It sees only the
`ChannelIntegrand` seam — channel count, per-channel grid dimension and
`value_in_channel(j, u)` — so the same loop drives the fixed-beam, proton and
multi-multiplicity integrands. `generate` replays it against the banked grids
([events/generate](generate.md)); the weights then become a file through
[events/lhef-weight-strategy](lhef-weight-strategy.md).

## The weight

A trial's weight is the full integrand term of its channel —
`|M|² × Jacobian × flux × PDFs × α_j g_j/g`, with the VEGAS grid's own Jacobian
— not `|M|²/max|M|²`. How σ and that weight are assembled is
[pipeline/overview](../pipeline/overview.md).

## One grid per channel, so one maximum per channel

The integral is split by channel, `∫f = Σ_j ∫ f·α_j g_j/g`, and each term gets
its own VEGAS grid with the channel frozen
([phase-space/per-channel-vegas-grids](../phase-space/per-channel-vegas-grids.md)).
A single grid over the mixture would need one global `w_max` set by the worst
channel. With a maximum per channel the overall efficiency is `σ / Σ_j w_max_j`;
on the processes measured that gained 1.7–2.9× and lost slightly (0.91×) on
`ee_to_tatah`, where one channel carries all of `Σ w_max_j`. The predictor of
the gain is the largest channel's share of `Σ_j w_max_j`, not the channel
count[^n21-grids][^n23-grids].

## The accept/reject loop

1. Draw channel `j` with probability `∝ w_max_j`.
2. Draw `u` against channel `j`'s frozen grid (`VegasGrid::draw`, which returns
   the point and its Jacobian weight), plus any trailing uniforms the integrand
   consumes beyond the grid (`scale_draw_ndim`, the clustering-configuration
   draw), from a stream of the pass's own. They are carried in the accepted
   point, so a reconstruction at the same `u` reproduces the value it was
   accepted on.
3. Accept with probability `min(1, w/w_max_j)`.
4. Keep an event above its channel's maximum at weight `r = w/w_max_j > 1`.

**Why `∝ w_max_j` and not `∝ σ_j`.** A trial in channel `j` carries mean
weight `σ_j/w_max_j` in units of that channel's maximum, so drawing the channel
with probability `q_j` accumulates kept weight `∝ q_j σ_j/w_max_j`, which is
`∝ σ_j` exactly when `q_j ∝ w_max_j`. Any other rule needs a compensating
per-event weight and stops being an unweighted sample; drawing `∝ σ_j`
over-populates channels whose maximum is small relative to their integral.
Summed, `E[r] = σ / Σ_j w_max_j`: the largest acceptance any channel-selection
rule can reach, realised when nothing sits above the maxima[^n23-e2].

**Overweights are kept, not clipped.** Clipping would bias σ low invisibly;
keeping them leaves the estimator unbiased and puts the distortion on the
record. `UnweightStats` counts them three ways — `overweight_fraction`,
`overweight_weight_share` (the load-bearing one: a negligible rate can carry a
large share of σ) and `excess_share` (what capping at `w_max` would
discard) — and `generate` reports them per run.

## How `w_max_j` is set: the truncation ladder

Each `w_max_j` is read off a frozen scan of channel `j`'s grid by a `MaxRule`:

- **`Truncated { excess_share }`**, the default at `DEFAULT_EXCESS_SHARE = 0.01`:
  the lowest scanned weight that still leaves less than `excess_share` of the
  scan's summed weight above it. This is MadGraph's `trunc_max` ladder[^mg-unwgt]:

  ```fortran
  do while (xsum-dabs(swgt(i))*(nw-i) .lt. xtot*trunc_max
 $          .and. i .gt. 2)
     xsum = xsum + dabs(swgt(i))
     i = i-1
  enddo
  if (i .lt. nw) i=i+1
  th_maxwgt = dabs(swgt(i))
  ```

  The two largest weights are never candidates, so a scan of two points or
  fewer yields its extremum.
- **`Extremum`**: the largest weight the scan saw (`--max-truncation 0`).

**Why not the extremum.** On the 24-channel `p p > l+ l- j` grids,
`Σ_j w_max_j ∝ n^0.51` from 10³ to 2.6·10⁵ draws per channel with no plateau,
and the σ-share above the maxima falls only as `n^−0.46`: a Pareto weight tail of
index ≈ 2 (Hill estimator 2.08–2.40 on the σ-carrying channels). The extremum of
such a tail never settles, so a scan budget buys a point on an
acceptance-versus-overweight curve, not convergence; under truncation the
maximum is a quantile, which does converge, and the budget buys its
precision[^n31-i2]. Measured at matched budgets, the truncated rule raised the
five gating rows' efficiency from 22.2/20.6/23.3/10.6/4.21% to
54.1/52.1/52.9/38.9/9.98%, and `p p > l+ l- j` at 300k×8 needed 477 125 trials
for 20 000 events instead of 2 269 051 (4.36× cheaper per event)[^n32-plan][^n32-out].
Neither rule biases the estimator; the choice moves acceptance against sample
lumpiness.

**Scan budget** (`--scan-points N|share`, `ScanBudget`): `share` (default) gives
each channel the points the integration spent on it per iteration; `N` gives
every channel the same count. Allocation is the much weaker lever: on the llj
grids the two land on the same overweight-versus-efficiency curve within the
5-stream spread (±7%), and the scan's own seed-to-seed spread on `Σ w_max_j` is
±20% over five seeds (`unweight.rs:83`). The scan runs channels in parallel on
per-channel streams, which moves no number[^n31-i2][^n32-out].


**Where truncation still cannot help.** The ladder cannot step below the top
of a scan whose single largest weight already exceeds 1% of its sum, which under
a tail index near 2 is any scan below ~10⁴ points. Floor-bound channels
(0.8–3.9k points on the mixed MLM row) therefore get the extremum in effect, and
there the overweights carry 4.6–7.4% of their own σ. Raising every channel to
≥ 8000 scan points cost 8× the CPU, bought 0.3% of σ and halved the efficiency,
so no scan floor is used. The tail comes from the adapted VEGAS grids, not from
the maps or the maximum ([phase-space/vegas-grid-weight-tail](../phase-space/vegas-grid-weight-tail.md))[^n41-m5].

## The sample's own σ

The accept/reject pass is one pass over the frozen grids, so its estimator
(`σ̂ = Σ w_max · Σ r / trials`, `Unweighter::sigma_from_events`) does not go
through the integrator's iteration combination. When the integration budget
leaves the banked σ biased, the sample's σ̂ is not: on `p p > l+ l- j` at
100 000 evaluations, four of five seeds sat +1.1 to +2.3% above the banked σ
(the integrator then still ~1% low), and at 300 000 the two agreed (mean
−0.07%)[^n24-bias]. The file's declared σ is normalised to the integration
([events/multi-process-normalisation](multi-process-normalisation.md)); σ̂ is
kept in the header, and its pull against the integration is the generate gate
with teeth. How iterations combine is
[phase-space/vegas-iteration-combination](../phase-space/vegas-iteration-combination.md).

## MadEvent, for comparison

MadEvent unweights per channel too, keeps overweights at their own weight in
`store_events`, rescales each channel to its integral, and then, when
`combine_events` writes the delivered file, writes every kept event at one
weight and **truncates** overweights to it, logging the loss as `trunc_cross`
([events/multi-process-normalisation](multi-process-normalisation.md)). So a
MadEvent file has equal weights and a ≤ ~1% truncation bias in shapes; a
vibegraph `Buffer` file keeps unbiased overweights, which a consumer weighting
by `XWGTUP` must honour[^n41-m5].

## Caveats

- **Photon-pole processes carry the heaviest tails.** `ee_to_mumua` showed a
  weight share above `w_max` two orders larger than other rows and an event at
  8.4× its channel's maximum under the extremum rule; the honest fix is pole
  coverage, not a fudged `w_max`[^n23-e2].
- **A `neval` that gives a good σ does not give a good `w_max`**, and the two
  do not improve in step: under the extremum rule the llj share above `w_max`
  was still falling at 600 000 evaluations[^n24-wmax].
- **The largest `w/w_max` is an extremum estimate** and moves non-monotonically
  with budget; do not read it as convergence[^n24-wmax].
- A trial budget per event (`MAX_TRIALS_PER_EVENT = 5 000 000` in `generate`)
  stops a stuck sampler; it is sized far above the reciprocal of any measured
  efficiency.

The gate on all of this is `validate_unweighting`
([validation/event-output-gates](../validation/event-output-gates.md)): its σ
reference is the VEGAS σ **and** an independent weighted estimator over the same
grids with a different channel-selection rule, so the reference cannot share the
generator's mistake[^n23-e2].

[^n21-grids]: Note 21 addendum: why per-channel grids, and the per-channel `w_max` gain table.
[^n23-e2]: Note 23 E2: the `∝ w_max_j` correction, overweight treatment, the oracle, `ee_to_mumua`.
[^n23-grids]: Note 23, what accept/reject inherits from per-channel grids.
[^n24-wmax]: Note 24 P4, `w_max` against budget under the extremum rule.
[^n24-bias]: Note 24 P4, five-seed sweep of the sample's σ against the banked σ.
[^n31-i2]: Note 31 I2: the budget premise falsified; Pareto tail; allocation a weak lever.
[^n32-plan]: Note 32 wave 1: the rule to match, cited by line rather than paraphrased; σ cells must not move.
[^n32-out]: Note 32 §5.1, the truncation rule landed and its measured effect.
[^n41-m5]: Note 41 M5 and F-A: MadEvent's unweighting read from the pinned tree; the floor-bound channels.
[^mg-unwgt]: MadGraph 3.7.1 `unwgt.f:358-367` (ladder), `:390-446` (overweights kept, rescaled, written).
