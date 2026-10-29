---
type: Session Report
title: "T1 report: bbx's α-share tails are half-covered H → ZZ* peaks and Z → 4f peaks whose channels the α survey starved"
description: "On bbx_to_ccx_emmm_qcd0 (20 paired seeds, 600k and 1.2M) the largest single-point contributions sit on two narrow structures. 85% (1.2M) and 46% (600k) of their summed square is the H → ZZ* peak in the half no channel's map covers: every H channel draws the same Z pair first, on its pole, so the configuration with the other pair near shell lies in a Breit–Wigner tail. Seed 55's 1.18e-8 pb point in channel 613 is one of these. Another 11% / 31% is a Z → 4f peak (Z → l l γ*) whose covering channels the α survey drove to α = 1e-11 … 1e-5. A symmetrised H → ZZ* split lowers the mirror points' weights 20–380×, and an α floor of 1e-4 lowers the Z → 4f points 1.0–125×. Together they cut the summed square of the recorded tail contributions to 6% (1.2M) and 25% (600k). Proposed, not implemented, with a falsifier."
status: draft
tags: [two-to-six, multichannel, heavy-tail, budget-ladder, higgs, alpha-survey, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-29}
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-10-28/29, one ladder job at a time (short probe runs overlapped it); one container restart before 23:30Z on 10-28 killed seed 121 at 1.2M, which was rerun alone"
  commit: "ladder: binary from 537f623 (tree f5ec7351) + the uncommitted capture patch (ast-T1-instrumentation.patch, sha256 2487f68a…), binary sha256 f6180b60…; probe: 0b3f2c1 (tree be759587), binary sha256 af7e7204… built from the same source before rustfmt (formatting only)"
  command: "see Method; TSVs, traces, records and scripts under the session scratchpad, prefixed ast-T1-"
sources:
  - {id: brief, resource: "T1.md", title: "T1 brief"}
  - {id: s1, resource: "../../scaling-floor/sessions/S1-report.md", title: "scaling-floor S1 report"}
  - {id: p1, resource: "../../floor-bound-neyman/sessions/P1-report.md", title: "floor-bound-neyman P1 report"}
  - {id: b1, resource: "../../two-to-six-gate/sessions/B1-report.md", title: "two-to-six-gate B1 report"}
  - {id: probe, resource: "../../../../../vibegraph-lib/src/hadronic/tail_probe.rs", title: "probe_2to6_tail_points"}
  - {id: dc, resource: "../../../../../vibegraph-lib/src/phasespace/diagram_channel.rs", title: "sample_branch, draw_invariant, invariant_measure"}
---
## Verdict

- **Where the tails sit.** The largest per-point weights of bbx's α-share
  channels sit on two narrow structures:
  - **H → ZZ* → 4f, in the half no channel covers.** All 15 H → ZZ* channels
    (325, 461, 462, 563, 564 for `cc̄μμ`; 296, 395, 396, 561, 562 for `eeμμ`;
    346, 527, 528, 565, 566 for `cc̄ee`) draw one Z pair first, on its pole.
    That is `cc̄` in the first and third groups and `ee` in the second. The
    configuration with the *other* pair near shell is drawn from the first
    pair's Breit–Wigner tail. There the H channels' density is 10²–10⁵ below
    its value on the covered half, while |M|² is not. Points in that half get
    weights 10²–10⁴ above their iteration's mean, in the H channels themselves
    and in every spine channel (613, 614, 598, 604, 582) that lands there by
    accident.
  - **Z → 4f (`Z → l⁺l⁻ γ*`, `γ* → l⁺l⁻ / cc̄`).** A four-fermion invariant sits
    on the Z pole, with a low-mass γ*. The channels whose maps follow this
    structure (393, 294, 424, 501, 509, …) were given α = 1e-11 … 1e-5 by the
    30k × 6 α survey. The mixture's density there is set by channels that do
    not follow it.
- **Seed 55's point is the first kind.** Channel 613, iteration 3, point 33267:
  m(`cc̄μμ`) = 124.993 GeV (−2.1 Γ_H from the pole), m(μμ) = 85.2, m(`cc̄`) = 20.9,
  m(ee) = 91.7. Channel 613's own share of the mixture there is
  α₆₁₃g₆₁₃/g = 0.0053; channel 325 holds 0.76. |M|²·pref/g = 2.1e-11, which
  is 2 480× the channel's median point. The replay reproduces every one of
  channel 613's eight iterations bit for bit against the ladder's trace on this
  tree.
- **Variance ranking.** At 1.2M, 613 holds 52% of the quoted variance and 55% of
  the across-seed variance (seed 55). It is followed by 614 (13% / 8%), then
  604, 606, 598, 582, 574, 325 and 590 (2–5% each). At 600k the
  quoted variance spreads over 598, 574, 614, 582, 325 and 590 (7.5–11% each),
  and the across-seed shares lead with 586 (12%) and 612 (8%). The seed sd
  reproduces S1's: 2.714e-9 → 3.330e-9 (ratio 1.227), χ²/dof 0.80 / 1.41.
- **Tail indices.** Each channel's tail is a mixture. The bulk of the
  spine channels has a Hill index above 2 (2.3–2.7 at the top 0.1%). Their
  extreme tails, the top 50 of 120 pooled iterations at 1.2M, read 1.18–1.94
  (598 1.18, 614 1.23, 588 1.32, 613 1.54, 604 1.73, 606 1.85, 582 1.89, 574
  1.94). The H channels are under 2 at every threshold: 325 1.64–2.01, 346 and
  296 1.1–1.9. The floor-bound 551, 385, 425 and 291 read 0.9–1.5 at 2%. **At or
  under 2, on their extreme tails: every channel named above.** Their per-point
  variance is governed by rare points, and √n scaling is not expected until
  those points are sampled routinely.
- **Why the quoted variance does not fall.** The bulk does scale. Each channel's
  median per-iteration point variance (V·n) is unchanged from 600k to 1.2M
  (×0.84–1.33), and the term's quoted variance falls as 1/2 or better where no
  rare hit lands (582 ×0.48, 574 ×0.46, 590 ×0.44, 325 ×0.38, geometric means). In
  613 the geometric mean is ×1.00, but its log-ratio sd is 2.41: one hit on
  the uncovered H half moves it by ×20 517 on seed 55. These hits are the rare
  draws of a channel whose map does not include the peak, about one per 10⁶
  points in 613. A doubled budget doubles the chance of drawing one. The
  √n reading does not hold for such a term: most seeds see no hit and the
  seeds that do carry it alone.
- **Candidate fix, checked on the dumped points.**
  1. *Symmetrise the H → ZZ* split.* Draw either pair first with probability ½
     (equivalently, add mirror channels). At seed 55's point the H channels'
     density rises ×241, the mixture ×197, and the weight falls from 1.52e-12 to
     7.8e-15 GeV⁻². The contribution to the seed's σ falls from 1.18e-8 to
     6.0e-11 pb. Over all recorded H-mirror top points (20 seeds, both rungs)
     the summed square falls to 0.3% (1.2M) and 2.7% (600k). Channel 325's own
     per-point second moment, reweighted to the symmetric map from its own
     draws, falls to 0.30–1.03 of today's per iteration. The covered half pays
     at most a factor 2 in density.
  2. *Floor α at 1e-4* in the survey's result (renormalised). It lowers the
     Z → 4f points ×1.0–125. It costs ~4% on typical weights and ~9% on a
     typical iteration's second moment (613 and 590 replayed). A floor of 1e-3
     costs ×2.2 and is not proposed.
  - Both together: the recorded tail contributions' summed square falls to
    6.4% (1.2M) and 25% (600k).
- **The excess is not fully explained.** Replacing the recorded tail points by their
  post-fix values gives a seed sd of 2.03e-9 (600k) and 1.85e-9 (1.2M), ratio
  0.91, not 0.71. Of the rest, a part is the 563–580 channels drawn identically
  at both rungs (S1, B1), anticorrelated with the others by the per-seed α. That
  part is not a tail and no sampler change to the tails reaches it. The other
  part is the tails this record cannot see: only the top 8 of 33 watched
  channels per iteration were recorded. The fix is a necessary condition for the
  1/√2 the item asks, not a sufficient one, and the falsifier below says so.
- **Falsifier for the fix.** On a 20-paired-seed bbx ladder (600k, 1.2M, this
  allocation), with the symmetrised H → ZZ* split and the α floor:
  - (a) the probe's record of channel 613, 614, 598 and 604 shows no H-mirror
    point with a single-point share above 1e-3 of its iteration. Today seeds 55,
    22, 176 and 11 have one at 0.92, ≥0.5 and ≥0.5.
  - (b) the geometric-mean ratio of 613's and 614's quoted variance is ≤ 0.6,
    with the log-ratio sd under 1.0 (today ×1.00 sd 2.41 and ×0.76 sd 1.43).
  - (c) the per-seed sd ratio is ≤ 0.85 and χ²/dof at 1.2M is ≤ 1.2.
  - (d) σ moves by less than its 20-seed error at each rung.

  (a) and (b) failing means the mechanism is wrong. (a) and (b) holding while (c)
  fails means the remaining excess is the identical-draw / α-survey part,
  and the next step is the heavy-tail error model below, not another sampler
  change.
- **Error model, if the fix is not taken.** Read the 2 → 6 integrals cells
  with the spread a Hill index of 1.2–1.9 implies. The sd of the mean scales as
  n^(1/α − 1), so a rung ratio of 2^(1/α − 1) = 0.79–0.97 is expected, not 0.71.
  Report the seed median and an interquartile spread beside the mean and sd,
  and drop the χ²/dof against quoted errors as a pass criterion (B1 Found 4).

## Step 1: the ladder and the variance ranking

`bbx_to_ccx_emmm_qcd0`, multi sampler, ByAlpha, 8 iterations (2 warm-up),
seeds 20260719, 11, 22 … 209, both rungs on this tree. The capture patch adds,
beside S1's 14-field trace line, the top 8 points (with coordinates and grid
Jacobian) and the top 256 weights of 33 watched channels per iteration, and the
top point of every other channel. It changes no draw: seed 20260719 at 1.2M and
seed 55 at 600k reproduce S1's TSV to every printed digit. Seed 55 at 1.2M reads
1.701461883e-6 ± 1.196190e-8 here against S1's 1.701418058e-6 ± 1.196173e-8, the
cross-binary ulp drift S1 recorded (S1 Found 3).

```
$ python3 -I ast-T1-scripts/rank.py ast-T1-ladder-trace.txt 25      (ast-T1-rank.txt)
== rung 600000: 20 seeds | mean 1.690011e-06 | seed sd 2.714e-09 | RMS quoted 3.109e-09 | chi2/dof 0.80
== rung 1200000: 20 seeds | mean 1.691689e-06 | seed sd 3.330e-09 | RMS quoted 3.662e-09 | chi2/dof 1.41
```

Quoted-variance share: a channel's Σ over seeds of its term's quoted variance
(plain mean of kept iterations, ΣV/k²) over the total. Across-seed share: each
channel's excursion `e = term − median of its kept iterations`, then
cov(e, σ_seed)/var(σ_seed). The per-channel across-seed variance of the terms
themselves is not usable (the per-seed α moves σ between channels, B1 Step 3).

| rung | channel | quoted-var share | across-seed share | seed sd without its excursion | largest excursion (pb, seed) |
|---|---|--:|--:|---|---|
| 1.2M | 613 | 0.520 | +0.545 | 2.425e-9 | 1.18e-8 (55) |
| 1.2M | 614 | 0.135 | +0.085 | 3.235e-9 | 4.29e-9 (22) |
| 1.2M | 604 | 0.050 | +0.014 | 3.365e-9 | 3.26e-9 (176) |
| 1.2M | 606 | 0.042 | +0.001 | 3.374e-9 | 2.33e-9 (11) |
| 1.2M | 598 | 0.039 | −0.018 | 3.417e-9 | 1.10e-9 (77) |
| 1.2M | 582 | 0.033 | −0.004 | 3.350e-9 | 5.80e-10 (33) |
| 1.2M | 574 | 0.026 | +0.038 | 3.214e-9 | 1.05e-9 (20260719) |
| 1.2M | 325 | 0.022 | −0.018 | 3.408e-9 | −8.0e-10 (143) |
| 1.2M | 590 | 0.021 | +0.041 | 3.203e-9 | −6.0e-10 (187) |
| 600k | 598 | 0.110 | +0.060 | 2.595e-9 | 1.94e-9 (66) |
| 600k | 574 | 0.101 | +0.048 | 2.654e-9 | 2.28e-9 (187) |
| 600k | 614 | 0.100 | −0.034 | 2.900e-9 | 2.07e-9 (154) |
| 600k | 582 | 0.096 | −0.053 | 2.908e-9 | 1.37e-9 (88) |
| 600k | 325 | 0.095 | −0.026 | 2.859e-9 | 1.59e-9 (198) |
| 600k | 590 | 0.075 | +0.003 | 2.762e-9 | 1.23e-9 (176) |
| 600k | 346 | 0.057 | +0.006 | 2.751e-9 | 1.52e-9 (121) |
| 600k | 586 | 0.048 | +0.118 | 2.462e-9 | 2.97e-9 (33) |
| 600k | 606 | 0.047 | −0.016 | 2.763e-9 | −5.9e-10 (143) |
| 600k | 596 | 0.040 | +0.046 | 2.635e-9 | 1.66e-9 (154) |
| 600k | 612 | 0.031 | +0.078 | 2.530e-9 | 1.51e-9 (165) |
| 600k | 588 | 0.027 | −0.018 | 2.792e-9 | 1.65e-9 (209) |
| 600k | 296 | 0.024 | −0.009 | 2.757e-9 | 8.4e-10 (209) |

With every channel's excursion removed the seed sd reads 2.323e-9 → 2.156e-9.
The excursions' summed across-seed share is 0.44 at 600k and 0.75 at 1.2M.

```
$ python3 -I ast-T1-scripts/robust.py ast-T1-ladder-trace.txt
20 seeds; identical channels per seed 563-580
as run: sd 2.714e-09 -> 3.330e-09 ratio 1.227, corr -0.10; paired diff sd 4.502e-09
   identical-draw part: sd 1.729e-09 (both rungs, same values); rest: sd 3.457e-09 -> 3.500e-09 ratio 1.012; corr(identical, rest) -0.63 / -0.34
robust (median of iterations): sd 2.323e-09 -> 2.156e-09 ratio 0.928, corr +0.23
   identical-draw part: sd 1.366e-09; rest: sd 3.138e-09 -> 2.791e-09 ratio 0.889; corr(identical, rest) -0.74 / -0.66
```

## Step 2: tail indices

Hill estimates use the top weights (W lines). Each weight is divided by its
iteration's mean accepted weight, then pooled over the kept iterations of all 20
seeds (120 iterations). Thresholds are the top 0.1%, 0.5% and 2% of the pooled
accepted count, and fixed k = 50 and 200 (the extreme tail). The error is
α/√k. Per-iteration median Hill at 2% is given for comparison.

```
$ python3 -I ast-T1-scripts/hillpool.py ast-T1-ladder-trace.txt 1200000 …   (ast-T1-hill-1200000.txt)
   613 20   612064 | 2.29±0.09 (612) | 2.39±0.04 (3060) | 1.92±0.02 (12241) | k=50: 1.54±0.22 | k=200: 2.13±0.15 | max 4483 | 2.28
   614 20 11368842 | 2.40±0.02 (11368) | n/a | n/a | k=50: 1.23±0.17 | k=200: 1.77±0.13 | max 10186 | n/a
   604 20   623582 | 1.75±0.07 (623) | 2.00±0.04 (3117) | 1.94±0.02 (12471) | k=50: 1.73±0.25 | k=200: 1.70±0.12 | max 4488 | 2.02
   606 20  8282273 | 2.60±0.03 (8282) | n/a | n/a | k=50: 1.85±0.26 | k=200: 1.90±0.13 | max 5310 | n/a
   598 20  9923773 | 2.48±0.02 (9923) | n/a | n/a | k=50: 1.18±0.17 | k=200: 1.68±0.12 | max 2011 | n/a
   582 20 12674547 | 2.34±0.02 (12674) | n/a | n/a | k=50: 1.89±0.27 | k=200: 1.84±0.13 | max 966 | n/a
   574 20  9176852 | 2.57±0.03 (9176) | n/a | n/a | k=50: 1.94±0.27 | k=200: 2.26±0.16 | max 2414 | n/a
   590 20  9172286 | 2.62±0.03 (9172) | n/a | n/a | k=50: 2.30±0.33 | k=200: 2.38±0.17 | max 828 | n/a
   325 20  1210945 | 1.86±0.05 (1210) | 1.71±0.02 (6054) | 1.06±0.01 (24218) | k=50: 1.83±0.26 | k=200: 2.01±0.14 | max 1077 | 1.72
   346 20  3369247 | 1.78±0.03 (3369) | 1.09±0.01 (16846) | n/a | k=50: 1.54±0.22 | k=200: 1.80±0.13 | max 3331 | 1.80
   296 20  4388727 | 1.69±0.03 (4388) | 1.27±0.01 (21943) | n/a | k=50: 2.53±0.36 | k=200: 1.87±0.13 | max 2378 | 1.78
   588 20   753736 | 1.84±0.07 (753) | 1.93±0.03 (3768) | 1.76±0.01 (15074) | k=50: 1.32±0.19 | k=200: 1.61±0.11 | max 1825 | 1.94
   551 20    51787 | 2.26±0.32 (51) | 1.36±0.08 (258) | 1.08±0.03 (1035) | k=50: 2.26±0.32 | k=200: 1.48±0.10 | max 352 | 1.00
$ python3 -I ast-T1-scripts/hillpool.py ast-T1-ladder-trace.txt 600000 …    (ast-T1-hill-600000.txt)
   613 20   308159 | 2.35±0.13 (308) | 2.36±0.06 (1540) | 2.13±0.03 (6163) | k=50: 2.63±0.37 | k=200: 2.37±0.17 | max 276 | 2.28
   614 20  5696628 | 2.57±0.03 (5696) | 1.61±0.01 (28483) | n/a | k=50: 1.60±0.23 | k=200: 1.96±0.14 | max 2088 | n/a
   598 20  5004540 | 2.57±0.04 (5004) | 1.85±0.01 (25022) | n/a | k=50: 1.40±0.20 | k=200: 2.05±0.14 | max 2324 | 2.45
   325 20   601901 | 1.64±0.07 (601) | 1.73±0.03 (3009) | 1.38±0.01 (12038) | k=50: 1.78±0.25 | k=200: 1.67±0.12 | max 2190 | 1.67
```

"n/a" is a threshold above the 256 weights the trace keeps per iteration. The
threshold choice moves the index by up to 1, and that sensitivity is the
finding: the extreme tail is not the bulk's power law continued, it is a second,
rare population. 613 at 600k has no such point in 20 seeds (max normalised
weight 276, k = 50 index 2.63). At 1.2M it has one (4 483, seed 55), and its
k = 50 index drops to 1.54.

## Step 3: the record, and seed 55's point

`probe_2to6_tail_points` (`vibegraph-lib/src/hadronic/tail_probe.rs`, committed
as 0b3f2c1, `#[ignore]`d, in-crate under `#[cfg(test)]`) builds the row as the σ
gate does and installs the trace's α vector. It then either replays chosen
channels serially or decomposes the points the run recorded. Under ByAlpha a
channel's schedule depends only on its own history, so the serial replay is
the production draw. Every replayed iteration of channels 613, 325, 598, 462, 614
and 590 for seed 55 at 1.2M (48 lines) equals the ladder trace's
`(I, V, n, accepted)` bit for bit. All 84 480 recorded points re-evaluate to
their recorded weight within 1e-9 (`ast-T1-classify.log`, 0 mismatches in 40
seed-runs).

```
$ VG_TAIL_TRACE=ast-T1-ladder-trace.txt VG_TAIL_SEEDS=55 VG_TAIL_NEVAL=1200000 VG_TAIL_CHANNELS=613 \
    ast-T1-lib-probe --ignored --exact hadronic::tail_probe::probe_2to6_tail_points      (ast-T1-probe-s55.txt)
I 55 1200000 613 2 1 1.6092441091483237e-17 1.024570626733911e-37 8390 4675 …
I 55 1200000 613 3 1 1.9769925530466999e-16 3.3034691207510493e-32 8390 4876 …   (trace: identical)
I 55 1200000 613 4 1 1.569548227244983e-17 1.967998042342806e-36 8390 4363 …
$ python3 -I ast-T1-scripts/points.py ast-T1-probe-s55.txt 0 55,1200000,613,3
seed 55 neval 1200000 ch 613 it 3 rank 0 idx 33267: fval 5.938e-04 pb-units | wgt 1.022e+01 | pref*m2/g 2.134e-11 | m2 1.353e-09 | g 2.357e-17 gown 1.787e-17 alpha 6.992e-03 own_share 5.299e-03
   top mixture shares: [(325, 0.76109), (598, 0.14876), (462, 0.033079), (564, 0.019681), (588, 0.0098106)]
   own channel 613: Z(s) m=85.222 [(q2-M2)/MG=-4.73]; b(t) t=-5.642e+04; b(t) t=-2.804e+04; Z(s) m=91.682 [+0.41]; a(s) m=20.857
   ch 325 (share 7.61e-01): Z(s) m=500.000; Z(s) m=91.682 [+0.41]; H(s) m=124.993 [(q2-M2)/MG=-2.09]; Z(s) m=20.857 [-35.40]; Z(s) m=85.222 [-4.73]
      pT c:45.70 c~:22.20 e+:58.96 e-:164.58 mu+:103.79 mu-:58.18 | eta all within |0.8| | minDR c mu-:0.441 cc~:0.664
      m(cc~) 20.86 m(ee) 91.68 m(mumu) 85.22
```

Reading the invariants:
- **Poles near the point.** The H line is 2.1 widths (Γ_H = 6.4 MeV) from its
  pole. Z(ee) sits at +0.4 Γ_Z and Z(μμ) at −4.7 Γ_Z. Channel 613's own two b
  t-channel lines are far from their pole (t = −2.8e4 and −5.6e4 GeV²).
- **Cuts.** No cut boundary is near: the lowest pT is 22 GeV against 20, ΔR
  0.44 against 0.4, |η| ≤ 0.8.
- **Factors against 613's median point in that iteration** (`dlines.py`, log₁₀):
  - fval +4.92;
  - grid Jacobian +1.62 (the grid holds few points there, a factor 40);
  - |M|² +4.70;
  - αⱼ/g −1.38 (other channels' density is high there);
  - |M|²/g₆₁₃ +4.43.

  The point is a matrix-element peak that 613's map does not follow.
  The VEGAS grid adds a factor 40 on top.

Channel 613's map is a b-exchange spine emitting γ*(`cc̄`), Z(ee) and Z(μμ). It
flattens the b propagators and the three s-channel lines. It has no H line. The
channel built for this peak is 325, a timelike tree with `ŝ` → Z H, H → Z(`cc̄`) Z(μμ).

## Step 4: the mechanism

**The H → ZZ* split is covered on one side only.** `sample_branch` draws the left
daughter's invariant first, with a Breit–Wigner over `[lo, (√s − μ_r)²]`. It
then draws the right one over `[lo, (√s − √s_l)²]`
(`phasespace/diagram_channel.rs`). For H → ZZ* (M_H < 2M_Z) the left Z lands
on its pole, and the right one must stay below 125 − 91 = 34 GeV. The mirror
configuration, right pair near shell and left pair far below, reads the left
Breit–Wigner deep in its tail.

The replays measure which pair each H channel draws first. The pair with the
median near M_Z is the left one (`hsplit.py`, three iterations each, seed 55):

```
325, 461, 462, 563, 564 (H -> cc~ mu+mu-):  cc~ median 90.5-90.8     -> mu+mu- on shell is uncovered
296, 395, 396, 561, 562 (H -> e+e- mu+mu-): e+e- median 90.6-90.8    -> mu+mu- on shell is uncovered
346, 527, 528, 565, 566 (H -> cc~ e+e-):    cc~ median 90.7-90.8     -> e+e- on shell is uncovered
```

The channel's own draws confirm it (`c325.py`, 325's kept points, seed 55):
- 325's median point has m(`cc̄`) = 89.1 and m(μμ) = 28.0, with g₃₂₅ = 2.2e-12;
- its 15 largest |M|²·pref/g points all have m(`cc̄`) = 18–34 GeV, the
  uncovered side, with g₃₂₅ = 4e-19 to 5e-15;
- |M|² there is only 6× below the median, while g₃₂₅ is 10³–10⁶ below.

The density ratio of the symmetrised split, computed from `invariant_measure`'s
Breit–Wigner (`sym.py`):

```
m_cc  20.857 m_mumu  85.222: mirror/orig 481.7  sym/orig 241.4      (seed 55's point)
m_cc  89.100 m_mumu  28.000: mirror/orig 0.001684  sym/orig 0.5008  (325's median point)
```

**Why a spine channel carries it.** At the peak the mixture density is set by
325, 462 and 564 (α 7.2e-3, 1.1e-3 and 4.4e-4). In the uncovered half those
densities are low, so f/g = 2.1e-11. Channel 325 sees such values in 1% of its
points (its p99 is 1.9e-11). Channel 613 draws the region about once in 10⁶
points (one hit in 20 seeds × 2 rungs). Each such hit carries
α₆₁₃ · f/g · w_grid. That is a rare-event term: its variance exists, but at
n·p ≪ 1 most seeds see nothing and the seed that sees one carries all of it.

**The second class: Z → 4f with starved channels.** Examples:
- seed 11, channel 606, iteration 7: m(μμ) = 91.27, m(ee) = 7.9, m(`cc̄ee`) on the
  Z pole (+0.03 Γ). The densest channels there (501, 315, 518, 509) have
  g ~ 1e-13 against 606's 3e-16, at α = 7e-9 … 7e-8.
- seed 20260719, channel 425: the densest channels are 424 and 423, at
  α = 3e-11.
- seed 20260719, channel 574: Z → ee γ*(μμ), m(eeμμ) = 90.91; the densest
  channels are 377, 369 and 378, at α = 1e-6 … 1e-5.

The α survey samples from the mixture. A channel whose structure the survey
never visits gets W ≈ 0, and the Kleiss–Pittau update drives its α down
geometrically over the six survey iterations (B1 saw channel 551's α span five
decades across seeds). The mechanism is a small multichannel denominator: every
channel with a large α has a low density where |M|² peaks.

**Classification of all recorded top points** (`classify.py`, `aggregate.py`):
- *H-mirror*: on the H peak within 15 Γ_H, with the H channels' second-drawn
  pair nearer M_Z.
- *Z4*: a four-fermion invariant within 15 Γ_Z of M_Z.

c is a point's contribution to its seed's σ, fval/(n·k).

```
$ python3 -I ast-T1-scripts/aggregate.py ast-T1-class-1200000.txt ast-T1-ladder-trace.txt 1200000
20 seeds at 1200000; points 31680
  class: points, sum c (pb), share of sum c^2, sum c^2 ratio after: symmetric H split | alpha floor 1e-4 | both
  Hmirror :   3217 1.259e-07 0.8547 | 0.0029 | 0.9955 | 0.0027
  Hpeak   :   1622 2.045e-08 0.0048 | 2.4422 | 0.9605 | 2.3395
  Z4      :   8354 8.122e-08 0.1058 | 0.9999 | 0.2482 | 0.2480
  other   :  18487 2.270e-07 0.0346 | 0.8142 | 0.8885 | 0.7100
  all: 0.1483 | 0.9125 | 0.0644
$ python3 -I ast-T1-scripts/aggregate.py ast-T1-class-600000.txt ast-T1-ladder-trace.txt 600000
  Hmirror :   2899 1.405e-07 0.4603 | 0.0270 | 0.9767 | 0.0194
  Hpeak   :   1856 3.819e-08 0.0214 | 3.0684 | 0.9539 | 2.8970
  Z4      :   7805 8.687e-08 0.3076 | 1.0000 | 0.1565 | 0.1565
  other   :  19120 3.615e-07 0.2107 | 0.9195 | 0.7026 | 0.6212
  all: 0.5793 | 0.6662 | 0.2498
```

The largest single-point contributions at 1.2M are 1.18e-8 (seed 55, 613,
H-mirror), 4.68e-9 (22, 614, H-mirror), 3.57e-9 (176, 604, H-mirror) and
2.50e-9 (11, 606, Z4). At 600k they are 2.96e-9 (33, 586, Z4), 2.91e-9 (66, 598,
H-mirror) and 2.49e-9 (187, 574, other). The per-seed tables are in
`ast-T1-agg-*.txt`. On 19 of 20 seeds at 1.2M, and on 15 of 20 at 600k, H-mirror
or Z4 carries more than half of the seed's recorded Σc². The "Hpeak" rise under
the symmetric split is points on the covered half, whose density halves.

- **Not a VEGAS grid that failed to adapt.** The grid adds a factor 1–40 at the
  tail points (w_grid 10.2 at seed 55's point). The structure behind them is
  ×10²–10⁵.
- **Not a cut edge.**
- **Not the floor.** 613, 614, 598, 604 and 582 are α-share channels.
- **Not the drift against MadEvent** (`bbx-drifts-from-madevent-with-budget`).
  The H-mirror points' summed contribution is 7.0e-9 pb a seed at 600k and
  6.3e-9 at 1.2M, so they do not grow with budget.

## Step 5: the fix, checked arithmetically

**Per point.** The two counterfactuals:
- *Symmetrised split.* Each H → ZZ* channel's density becomes
  g'_k = R_k g_k, with R_k the symmetrised/original ratio of its two Breit–Wigner
  draws at the point; the mixture becomes g' = g + Σ_k α_k g_k (R_k − 1).
- *α floor.* α'_k = max(α_k, floor), renormalised, applied to every term
  including the channel's own.

The point's weight under each is fval·g/g' (with α'_j/α_j for the floor). Lower
bounds of the invariant draws are taken as 0: they are cut-implied floors of a
few GeV², which moves the arctangent range by under 1e-4 of π.

| seed, channel, it | class | contribution now (pb) | symmetric split | α floor 1e-4 | both |
|---|---|---|---|---|---|
| 55, 613, 3 | H-mirror | 1.180e-8 | 6.0e-11 | 1.18e-8 | 6.0e-11 |
| 22, 614, 6 | H-mirror | 4.682e-9 | 1.6e-11 | 4.68e-9 | 1.6e-11 |
| 176, 604, 4 | H-mirror | 3.574e-9 | 2.7e-11 | 3.55e-9 | 2.7e-11 |
| 11, 606, 7 | Z4 | 2.497e-9 | 2.50e-9 | 5.8e-10 | 5.8e-10 |
| 20260719, 425, 3 | Z4 | 1.524e-9 | 1.52e-9 | 1.2e-11 | 1.2e-11 |
| 20260719, 574, 6 | Z4 | 1.130e-9 | 1.13e-9 | 3.1e-10 | 3.1e-10 |
| 66, 598, 4 (600k) | H-mirror | 2.912e-9 | 1.7e-11 | 2.91e-9 | 1.7e-11 |
| 33, 586, 7 (600k) | Z4 | 2.961e-9 | 2.96e-9 | 4.6e-11 | 4.6e-11 |

**On a whole channel, from its own draws.** These reweight every accepted point
of a channel that the replay dumped. They use the same draws and assume the grid
density is unchanged.

Channel 325 under the symmetric split (`sym325.py`, seed 55, 1.2M), per-point
second moment, new over old: 0.35, 0.56, 0.72, 0.30, 0.94, 0.36, 1.03, 1.02
(iterations 0–7). The term moves −3% … +18% as weight shifts between channels.

The α floors on 613 and 590 (`hpeak.py`, iterations other than 613's hit):

| channel | floor | term | second moment |
|---|---|---|---|
| 613 | 1e-4 | ×1.04 | ×1.09 |
| 613 | 1e-3 | ×1.49 | ×2.2 |
| 590 | 1e-4 | ×1.05 | ×1.09 |
| 590 | 1e-3 | ×1.55 | ×2.3 |

Two other partitions of unity were checked in the same pass:
- `αⱼgⱼ²/Σαₖgₖ²` cuts the hit ×1e4 in second moment but costs 613's typical
  iterations ×3.4.
- MadEvent's AMP2 partition costs 590 and 614 ×4–15.

Neither is proposed.

**On the ladder** (`project.py`). The recorded points are replaced by their
post-fix contributions and the mean the fix moves into the H channels is not
added back, so read the spreads, not the means:

```
600000:  as run sd 2.714e-09 | symmetric H split 3.171e-09 | alpha floor 1e-4 1.846e-09 | both 2.028e-09
1200000: as run sd 3.330e-09 | symmetric H split 2.254e-09 | alpha floor 1e-4 3.347e-09 | both 1.851e-09
```

With both, the rung ratio reads 0.91. That is an upper bound on what a seed
spread with its visible tails removed can show here, not a prediction. The
falsifier in the verdict is the test.

## Method

- Capture patch: `ast-T1-instrumentation.patch`, not committed. It is S1's trace
  line plus `TopPoint` capture in `adapt_blocks_iteration`. The env vars are
  `VG_TOPX=8`, `VG_TOPM=256` and `VG_TOPW=<33 channels>`. Its output is
  `ast-T1-ladder-trace.txt` and `.points`.
- Build: `ast-T1-build.sh` (validate_sigma, release-debug, debug=0). Drivers are
  `ast-T1-run.sh` / `ast-T1-ladder.sh`, one seed per process, launched with
  `nohup setsid`. They skip a seed already in the TSV and drop a partial trace
  of the seed they rerun. After the restart only seed 121 at 1.2M was rerun;
  every seed-run appears once in the trace (4 920 lines each).
- Probe: `ast-T1-buildprobe.sh` builds the lib test binary. It is run as
  `ast-T1-probe.sh <tag> <seeds> <neval> <channels> [VG_TAIL_*]`, and over the
  ladder's recorded points as `ast-T1-classify-run.sh <neval> <seeds>`.
- Scripts in `ast-T1-scripts/`:
  - `rank.py`, `robust.py`, `geo.py`: the ladder;
  - `hillpool.py`, `hill.py`, `dlines.py`: tail indices and factors;
  - `points.py`, `c325.py`, `hsplit.py`: invariants and the split order;
  - `sym.py`, `sym325.py`, `hpeak.py`: the symmetric-split arithmetic;
  - `classify.py`, `aggregate.py`, `project.py`: the per-point classes and
    counterfactuals.
- Falsifier input today (`geo.py`), geometric mean of the quoted-variance
  ratio 600k → 1.2M (log-ratio sd in brackets), with points ×2.00 in every
  channel except 551:

  | channel | quoted variance | falls on |
  |---|---|---|
  | 613 | ×1.001 (2.41) | 17/20 |
  | 614 | ×0.757 (1.43) | |
  | 604 | ×0.737 (1.83) | |
  | 606 | ×0.648 | |
  | 598 | ×0.635 | |
  | 582 | ×0.478 | |
  | 574 | ×0.464 | |
  | 590 | ×0.439 | 20/20 |
  | 325 | ×0.379 | |
  | 346 | ×0.449 | |
  | 296 | ×0.616 | |
  | 551 (floor-bound, points ×1.00) | ×1.251 | |

  The median per-iteration point variance moves ×0.84–1.33.

## Gates

On 0b3f2c1 (the probe), detached, with `CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` (`ast-T1-gates.log`):

```
$ pixi run cargo fmt --all --check                                                    --> exit 0
$ pixi run cargo clippy --workspace --all-targets -- -D warnings                      --> exit 0
$ pixi run cargo clippy --workspace --all-targets --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings   --> exit 0
$ pixi run cargo test --workspace                                                     --> exit 0
  48 test-result lines: 1704 passed, 0 failed, 20 ignored
```

No production code is committed. The capture patch stayed in the scratchpad and
the tree was reverted before the probe was written.

## Brief corrections

- **"Rerun here; traces do not replay across trees."** True across binaries,
  not across source. This tree's library source equals S1's base, a2b31a5,
  apart from S1's removed rule. Even so, seed 55 at 1.2M differs from S1's run
  at 2.6e-5 relative, while seeds 20260719 (1.2M) and 55 (600k) match S1 to every
  digit. The rerun was needed, and the record's replays are exact only against a
  trace from the same build, which is why the probe installs the trace's α
  vector rather than re-surveying.
- **"613, 614, 590 are α-share channels with their own tails."** They carry
  the tails, but the tails are not theirs. 613's and 614's largest points sit
  on a peak that belongs to the H channels, at an own share of 0.5–40%.
- **"Test-only record (`#[cfg(test)]` or `#[ignore]`d)."** The per-point
  decomposition is committed that way. The capture of a running integration's
  top points needs a hook in `adapt_blocks_iteration`, which is production code,
  so it stayed uncommitted, as in B1, P1 and S1. The probe's replay mode
  recovers the same points from the trace's α vector without it, bit for bit.

## Found

1. **H → ZZ* splits are mapped on one side only.**
   - Evidence: every one of the 15 H → ZZ* channels draws the same pair on
     shell (Step 4), and points in the other half carry 46–85% of the recorded
     tail Σc².
   - To close: a symmetrised two-daughter Breit–Wigner draw, or mirror channels,
     for any split whose two daughters carry the same pole and whose parent is
     below twice it. Measure it with the falsifier above. The same draw order
     likely applies to any `X → V V*` below threshold in other rows.
2. **The α survey starves channels whose peak it never samples.**
   - Evidence: Z → 4f tail points are dominated in density by channels at
     α = 3e-11 … 1e-5 (Step 4). A 1e-4 floor costs ~9% in second moment on
     typical channels.
   - To close: a floor, or a minimum per-channel survey draw, in
     `adapt_alphas`, measured on the gated rows as well as bbx (it changes
     every row's α).
3. **`uux` has the same three ZH → ZZ* channels** (288, 305, 322, with the
   same pair order) at α = 0.018–0.028, 2–4× bbx's. Why its ladder falls
   anyway is open. The probe runs on it unchanged
   (`VG_TAIL_DIR=uux_to_ccx_emmm_qcd0`).
4. **Ulp drift across builds of the same source.** S1 Found 3 is
   stronger than stated: one library source built twice gives different α
   vectors for some seeds. A trace replays only against its own binary.
5. **The floor-bound tail on `784582d` (`two-to-six-tail-point-unlocated`).**
   This record covers the per-point half of that item on main: the
   probe and the classification work for any channel, and 551's pooled index
   here is 1.0–1.5 at 2%. The `784582d` attribution (#48 or #51) was not run.
