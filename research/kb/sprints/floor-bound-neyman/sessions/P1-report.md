---
type: Session Report
title: "P1 report: Neyman's ladder passes only through its biased iteration weights"
description: "On bbx_to_ccx_emmm_qcd0, 20 paired seeds at 600k and 1.2M, Neyman reads seed sd 2.71e-9 → 1.76e-9 (ratio 0.65) with χ²/dof 1.14 / 0.76, against ByAlpha's 2.69e-9 → 3.34e-9. But it sits 1.7e-9 and 2.4e-9 pb below ByAlpha (paired t −2.4, −2.8), and recombining its own iterations as a plain mean removes both the shift (t −0.3) and the gain (ratio 1.18, χ²/dof 1.51 / 1.67). The point-weighted combine, whose weights Neyman sets from earlier iterations, down-weights tail iterations: variance bought with bias. Neyman moves no budget above the floors, and 551–575 of 615 channels still draw identical points at both rungs."
status: draft
tags: [multichannel, allocation, vegas, two-to-six, heavy-tail, budget-ladder, neyman, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-24}
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-10-23/24, one ladder job at a time"
  commit: "probe at this session's commit (on a79753c, tree of a79753c = 3597c78 plus the probe change), run with the uncommitted VG_TRACE instrumentation below; ByAlpha control = B1's main-tree trace (e5a09a8, code-identical, see Control)"
  command: "see Method; TSVs, traces, logs and scripts under the session scratchpad, prefixed fbn-P1-"
sources:
  - {id: brief, resource: "P1.md", title: "P1 brief"}
  - {id: b1, resource: "../../two-to-six-gate/sessions/B1-report.md", title: "B1 report"}
  - {id: log, resource: "../../two-to-six-gate/log.md", title: "two-to-six-gate log, 20-seed MadEvent references"}
  - {id: budget, resource: "../../../../../vibegraph-lib/src/budget.rs", title: "allocate_iteration, neyman_allocation, ChannelHistory::combine_kept"}
---
## Verdict

- **On its face Neyman passes.** Over 20 paired seeds its per-seed sd falls ×0.648 from
  600k to 1.2M (leave-one-out 0.547–0.709), against 1/√2 = 0.707. Its χ²/dof is 1.14 and
  0.76. ByAlpha on the same tree reads ×1.243 (×1.048 without each rung's worst seed),
  with χ²/dof 0.79 and 1.44.
- **The gain is not in the draws. It is in how Neyman's iterations are combined, and
  that combination is biased low.** Under Neyman a channel's per-iteration point count
  varies, so `ChannelHistory::combine_kept` averages its iterations weighted by points.
  Those counts are set from the channel's earlier per-point sd, which a tail iteration
  inflates. The iterations after a tail point therefore draw more points and outweigh it.
  Measured on the same seeds:
  - Neyman − ByAlpha, as the production code combines them: −1.71e-9 pb (t −2.40) at
    600k, −2.41e-9 pb (t −2.77) at 1.2M. That is 0.10 % and 0.14 % of σ, and 4–6 times
    the 20-seed error of either arm's mean.
  - The same Neyman iterations, each channel recombined as the plain mean of its kept
    iterations: −0.26e-9 (t −0.33) and −0.30e-9 (t −0.23). The plain mean is unbiased,
    since each iteration is unbiased whatever count it was given. ByAlpha's own plain
    recombination moves its σ by 1.5e-11.
  - The shift sits in the channel-runs whose largest kept iteration drew fewer than
    their mean points: −1.60e-9 and −2.30e-9 a seed, against +0.15e-9 and +0.19e-9 for
    the rest.
  - Under the plain mean, Neyman's ladder reads sd 3.36e-9 → 3.96e-9 (ratio 1.18) and
    χ²/dof 1.51 / 1.67, no better than ByAlpha.

  The production weighting suppresses heavy-tail iterations, which lowers both the
  spread and the mean. A 1/√2 sd ratio from that is not the fix the item asks for.
- **Neyman does not move budget above the floors.** By construction
  (`allocate_iteration`) it re-splits only the points the α split already puts above
  the floors. Over iterations 1–7, measured:

  | arm | rung | floor-bound channels (mean, range) | points above the floors | points / iteration |
  |---|---|---|---|---|
  | ByAlpha | 600k | 589.3 (576–599) | 0.316 (0.295–0.331) | 1 736 706 |
  | Neyman | 600k | 588.4 (578–602) | 0.316 (0.295–0.331) | 1 736 677 |
  | ByAlpha | 1.2M | 583.6 (574–596) | 0.488 (0.470–0.503) | 2 323 099 |
  | Neyman | 1.2M | 578.3 (567–594) | 0.488 (0.470–0.503) | 2 323 058 |

  Iteration 0 is the α split under both rules: 579 (600k) and 569 (1.2M) channels at
  the floor, with 0.645 and 0.787 of the points above it. Between the rungs, 551–575
  channels per seed draw bit-identical histories under Neyman, against 563–580 under
  ByAlpha. The shared-draw structure is still there.
- **Channel 551 (falsifier).** Neyman feeds it only once a tail point has inflated its
  sd. Its kept iterations stay at the floor on 17 of 20 seeds at 600k and on 10 at
  1.2M; where it stays there, its proportional demand is 0.1–0.7 of the floor. On the six seeds it lifts at 1.2M, the
  points rise ×1.95 and the quoted variance falls ×0.17 (geometric means). Over the arm:
  - Neyman: mean kept points 12 981 → 18 580 and RMS quoted error 8.8e-11 → 5.8e-11.
    The variance falls ×0.44 for ×1.43 points, as the falsifier requires.
  - ByAlpha: 12 288 points at both rungs, and the RMS error rises 8.1e-11 → 1.0e-10.

  The across-seed sd of the term is set by the per-seed α, which spans five decades.
  It reads 2.5e-10 → 2.6e-10 (Neyman) and 2.3e-10 → 2.8e-10 (ByAlpha), so it cannot test
  the scaling.
- **Channel 613 is starved.** Neyman gives it about half of ByAlpha's points (14 025
  and 29 022 kept against 26 182 and 52 364). Seed 55's 1.18e-8 pb ByAlpha tail point
  is not drawn under Neyman, whose counts move the stream positions. That is a
  different draw, not a cured tail. Neyman's largest 613 single-point share over 40 runs
  is 9.7e-11.
- **Recommendation.** Do not switch the fixed-budget default on this evidence (step 6
  does not apply). The next step is the item's second half, a floor that scales with
  `neval` (proposal below). The point-weighted combine under a data-driven allocation is
  a correctness finding in its own right (Found 1). It also touches the Neyman target
  mode the CLI defaults to.

## The ladder, per arm and rung

20 seeds (20260719, 11, 22 … 209), bbx only, multi sampler, 8 iterations (2 warm-up).
"err (comb.)" is the probe's combined quoted error, √Σerr²/n. "sd" is the per-seed
standard deviation. RMS quoted is the per-seed quoted error's RMS. All values are in pb.

| arm | rung | mean | err (comb.) | spread/√n | per-seed sd | RMS quoted | χ²/dof | sd ratio 1.2M/600k |
|---|---|---|---|---|---|---|--:|--:|
| ByAlpha | 600k | 1.689996e-6 | 6.940e-10 | 6.013e-10 | 2.689e-9 | 3.103e-9 | 0.79 | |
| ByAlpha | 1.2M | 1.691693e-6 | 8.196e-10 | 7.473e-10 | 3.342e-9 | 3.665e-9 | 1.44 | 1.243 |
| Neyman | 600k | 1.688286e-6 | 5.707e-10 | 6.065e-10 | 2.712e-9 | 2.552e-9 | 1.14 | |
| Neyman | 1.2M | 1.689281e-6 | 4.726e-10 | 3.931e-10 | 1.758e-9 | 2.114e-9 | 0.76 | 0.648 |
| Neyman, plain-mean replay | 600k | 1.689739e-6 | 6.714e-10 | 7.505e-10 | 3.356e-9 | | 1.51 | |
| Neyman, plain-mean replay | 1.2M | 1.691395e-6 | 8.934e-10 | 8.854e-10 | 3.960e-9 | | 1.67 | 1.180 |
| ByAlpha, plain-mean replay | 600k | 1.690011e-6 | 6.953e-10 | 6.070e-10 | 2.714e-9 | | 0.80 | |
| ByAlpha, plain-mean replay | 1.2M | 1.691689e-6 | 8.189e-10 | 7.446e-10 | 3.330e-9 | | 1.41 | 1.227 |

**Paired rungs** (per-seed 1.2M − 600k, over 20 seeds):

| arm | Δ mean | sd(Δ) | Δ/(sd/√n) | rung correlation | `√(s₁²/n+s₂²/n)` (not used) |
|---|---|---|--:|--:|---|
| ByAlpha | +1.697e-9 | 4.503e-9 | +1.69 | −0.10 | 9.59e-10 |
| Neyman | +9.956e-10 | 3.077e-9 | +1.45 | +0.10 | 7.23e-10 |
| Neyman, plain mean | +1.656e-9 | 5.452e-9 | +1.36 | | |

**Paired arms** (per-seed Neyman − ByAlpha at one rung; the α survey depends on the seed only, and `cross.py` asserts the two arms' α agree to 1e-12):

| rung | production combine | Neyman plain − ByAlpha | Neyman plain − ByAlpha plain |
|---|---|---|---|
| 600k | −1.711e-9, sd 3.181e-9, t −2.40 | −2.577e-10, sd 3.469e-9, t −0.33 | −2.723e-10, t −0.35 |
| 1.2M | −2.412e-9, sd 3.897e-9, t −2.77 | −2.981e-10, sd 5.875e-9, t −0.23 | −2.941e-10, t −0.22 |

The production shift's largest per-channel shares at 1.2M are in channels 613 (−5.96e-10),
614 (−5.55e-10), 590 (+3.02e-10), 296, 604, 385, 606 and 574.

```
$ python3 -I fbn-P1-scripts/shift.py t26-B1-main-trace.txt fbn-P1-neyman-trace.txt
=== 600000, 20 seeds
σN - σA (production combine):        mean -1.711e-09 sd 3.181e-09 t -2.40
σN(plain iteration mean) - σA:        mean -2.577e-10 sd 3.469e-09 t -0.33
σN(plain) - σA(plain):                mean -2.723e-10 sd 3.465e-09 t -0.35
=== 1200000, 20 seeds
σN - σA (production combine):        mean -2.412e-09 sd 3.897e-09 t -2.77
σN(plain iteration mean) - σA:        mean -2.981e-10 sd 5.875e-09 t -0.23
σN(plain) - σA(plain):                mean -2.941e-10 sd 5.872e-09 t -0.22
$ python3 -I - (weighted − plain, split by whether the channel-run's largest kept iteration drew fewer than its mean points)
600000: Σ over seeds (weighted - plain) -1.453e-09 per seed; ... fewer than their mean points: 3040, contributing -1.598e-09 per seed; the rest 2484: +1.450e-10
1200000: Σ over seeds (weighted - plain) -2.113e-09 per seed; ... fewer than their mean points: 3136, contributing -2.299e-09 per seed; the rest 2476: +1.857e-10
$ python3 -I fbn-P1-scripts/plainarm.py fbn-P1-neyman-trace.txt neyman
neyman plain 600000: mean 1.689739e-06 quoted 6.714e-10 spread/√n 7.505e-10 sd 3.356e-09 chi2/dof 1.51 | pull vs reported +2.17 pull vs equal-weight +1.17
neyman plain 1200000: mean 1.691395e-06 quoted 8.934e-10 spread/√n 8.854e-10 sd 3.960e-09 chi2/dof 1.67 | pull vs reported +3.12 pull vs equal-weight +2.07
neyman plain paired Δ mean +1.656e-09 sd 5.452e-09 t +1.36; sd ratio 1.180
$ python3 -I fbn-P1-scripts/extra.py t26-B1-main-trace.txt fbn-P1-neyman-trace.txt   (part 3)
  alpha : ratio 1.243, leave-one-out 0.917-1.438; worst seeds 600k 33, 1.2M 55; without them sd 2.374e-09 -> 2.488e-09, ratio 1.048
  neyman: ratio 0.648, leave-one-out 0.547-0.709; worst seeds 600k 55, 1.2M 165; without them sd 2.512e-09 -> 1.440e-09, ratio 0.573
```

Neyman's per-seed quoted errors are smaller too (RMS 2.55e-9 and 2.11e-9, against 3.10e-9
and 3.67e-9). The point-weighted variance gives a down-weighted tail iteration a small
share of the quoted error as well as of the mean.

## Step 2: the premise check

Run as the first three seeds of the 20, so they count in the ladder. Per-iteration
points (floor) for channels 551 and 613, from the trace:

```
600k  20260719 551: 512(512) 2048(2048) ×7         613: 2995(512) 3034(1788) 2718(1405) 2253(1238) 2100(1146) 1774(1085) 1773(1052) 1733(1023)
600k  11       551: 512(512) 2048(2048) ×7         613: 4213(512) 2710(1829) 2148(1430) 2113(1277) 2073(1182) 2024(1124) 2015(1084) 2014(1050)
600k  22       551: 512(512) 2048(2048) ×7         613: 2157(512) 1823(1823) 1412(1412) 1409(1263) 1416(1176) 1396(1096) 1369(1055) 1333(1024)
1.2M  20260719 551: 717(512) 2089(2048) 17724 15057 11714 10813 10085 9540 (2048)
1.2M  11       551: 848(512) 2048 2048 2048 2048 2180 3143 2966 (2048)
1.2M  22       551: 512(512) 2048(2048) ×7
1.2M  20260719 613: 5990(512) 4990 4444 4001 3280 3225 3589 3638
```

Channel 551 left its floor at 1.2M on two of three seeds, so the brief's stop condition
("at their floors at both rungs") did not hold and the 20-seed run went ahead.
`neyshare.py` replays `neyman_allocation` from the trace. It reproduces every one of the
280 Neyman allocations (40 runs × iterations 1–7) with zero mismatches, and prints the
channel's proportional demand. On seed 20260719 at 600k, 551 asked for 0.48–0.68 of its
floor. At 1.2M it asked for 1.02 of it, then 8.65 once iteration 1 (warm-up, 2.55e-9 pb
± 2.14e-9 against the 0.6–0.9e-9 of its kept iterations) had raised its sd. On seed 22
at 1.2M the demand is 0.18–0.29. **Neyman lifts a floor-bound channel only after one of
its own points has made it look noisy.**

## Step 4: the falsifier

`extra.py`, part 2, with kept points and the channel term's quoted variance, per seed,
600k → 1.2M:

```
ch551 alpha : 12/20 seeds bit-identical between rungs; 0 seeds with >1.2x points at 1.2M; other non-identical seeds var ratio geo-mean 1.75 (n 8)
ch551 neyman: 7/20 seeds bit-identical between rungs; 6 seeds with >1.2x points at 1.2M [33, 55, 66, 121, 132, 20260719]: geo-mean points ratio 1.95, geo-mean quoted-var ratio 0.17; other non-identical seeds var ratio geo-mean 1.06 (n 7)
ch613 alpha : 0/20 bit-identical; 20 seeds with >1.2x points: geo-mean points ratio 2.00, geo-mean quoted-var ratio 1.00
ch613 neyman: 0/20 bit-identical; 19 seeds with >1.2x points: geo-mean points ratio 1.98, geo-mean quoted-var ratio 0.51
```

Arm-level reading (`ana.py`, `cross.py`), at 600k → 1.2M:

| channel | arm | mean kept points | RMS quoted error of the term | across-seed sd of the term | largest single-point share |
|---|---|---|---|---|---|
| 551 | ByAlpha | 12 288 → 12 288 | 8.08e-11 → 1.02e-10 | 2.27e-10 → 2.81e-10 | 1.45e-10 → 2.06e-10 |
| 551 | Neyman | 12 981 → 18 580 | 8.82e-11 → 5.84e-11 | 2.46e-10 → 2.61e-10 | 2.72e-10 → 2.06e-10 |
| 613 | ByAlpha | 26 182 → 52 364 | 1.06e-10 → 2.64e-9 | 4.08e-9 → 4.79e-9 | 6.8e-11 → 1.18e-8 (seed 55) |
| 613 | Neyman | 14 025 → 29 022 | 1.35e-10 → 9.73e-11 | 4.04e-9 → 4.07e-9 | 9.7e-11 → 8.8e-11 |

- Where Neyman gives channel 551 more points, its within-seed variance falls at least
  in proportion. The falsifier is not refuted.
- The across-seed variance of a channel term does not test it on this process. The α
  vector is surveyed per seed and moves σ between channels, which B1 found too: 551's
  term mean 3.4e-10 has an α-driven across-seed sd 2.5–3 times its RMS quoted error, at
  both rungs and under both rules.
- Under ByAlpha, 613's quoted variance does not fall with doubled points (geometric mean
  ×1.00). Its median per-iteration point variance is unchanged (1.60e-16 → 1.51e-16), so
  the sum of iteration variances is carried by a few spiky iterations at both rungs.
  This is not followed further here.

## Step 5: cost and σ against MadEvent

Both arms spend the same points: Neyman is handed exactly the α split's total, floors
included, and re-splits it (points/iteration in the table above). Per-seed wall time:

| arm | rung | wall/seed mean (min–max) | where |
|---|---|---|---|
| ByAlpha | 600k | 719 s (612–790) | B1 session, 2026-10-22/23 |
| ByAlpha | 1.2M | 1 297 s (962–1 777) | B1 session |
| ByAlpha | 600k | 1 012, 956 s (seeds 20260719, 11) | this session, same binary as Neyman |
| ByAlpha | 1.2M | 1 204, 1 312 s (seeds 20260719, 11) | this session |
| Neyman | 600k | 866 s (757–989) | this session |
| Neyman | 1.2M | 1 173 s (1 036–1 421) | this session |

Same-session pairs: at 600k, seed 20260719 took 1 012 s under ByAlpha and 938 s under
Neyman, and seed 11 took 956 and 974 s. At 1.2M they took 1 204 and 1 312 s under ByAlpha, and 1 294 and 1 328 s under
Neyman. The two rules cost the same to
within this host's run-to-run noise, which is ±15 % between sessions.

Variance × time at 1.2M, per seed (sd² × mean wall):
- ByAlpha: (3.342e-9)² × 1 297 = 1.45e-14.
- Neyman as combined: (1.758e-9)² × 1 173 = 3.63e-15, a factor 0.25.
- Neyman, plain mean: (3.960e-9)² × 1 173 = 1.84e-14, a factor 1.27.

The 0.25 is the biased estimator's figure. The unbiased one is no better than ByAlpha.

**Pulls against the 20-seed MadEvent references** (reported 1.686455e-6 ± 1.31e-9;
equal-weight 1.687758e-6 ± 1.52e-9; two-to-six-gate log, 2026-10-23), with the measured
spread/√n as the arm's error:

| arm | rung | vs reported | vs equal-weight |
|---|---|--:|--:|
| ByAlpha | 600k | +2.46 | +1.37 |
| ByAlpha | 1.2M | +3.47 | +2.32 |
| Neyman | 600k | +1.27 | +0.32 |
| Neyman | 1.2M | +2.07 | +0.97 |
| Neyman, plain mean | 600k | +2.17 | +1.17 |
| Neyman, plain mean | 1.2M | +3.12 | +2.07 |

Neyman's closer agreement is the −0.1 % bias above, not a better estimate. The unbiased
reading of both arms sits +2.2 to +3.5 σ above the reported reference, and the pull grows
with budget (Found 4). Per D1's third amendment the cells stay `info`, since the
references' own channels scatter.

## Step 6: where the default would change

Neyman does not pass on an unbiased reading, so nothing should switch. For the record,
the places that fix `ByAlpha` for a fixed budget:
- `FixedBeamIntegrand::adapt_grids` / `adapt_grids_with` (`hadronic.rs`) and the
  `proton.rs` equivalent. Every `integrate_with` caller in `validate_sigma.rs`
  (the `validate_*` σ gates and probes) goes through these.
- `IntegrateArgs::allocation` in `vibegraph-cli/src/integrate.rs`: `--fixed-budget`
  defaults to `ByAlpha` and the target mode to `Neyman`.
- The `multiplicity.rs` tests.

The target-mode default already runs the combine Found 1 describes.

## Proposal: a floor that scales with `neval` (not implemented)

At 1.2M, 49 % of each iteration's points sit above the floors under either rule. The
other 51 % (about 1.19M of 2.32M) are floor points. Their channels draw identical points
at both rungs, which holds for 551–580 of 615 channels per seed. Neyman cannot reach
them: it is handed only the points above the floors, and pins every channel whose
demand is under its floor. A floor proportional to `neval` would make every channel's
draws differ between rungs and grow with the budget. The cost is a coverage promise that
then depends on the request, and a check that the lowest rung still meets
`MIN_CHANNEL_NEVAL` accepted points. It has to be evaluated with an outcome-independent
combination (Found 1), or it will read the same false gain wherever its counts vary
between iterations.

## Control: why ByAlpha was not rerun for 20 seeds

The brief allows reusing B1's main-tree ByAlpha run "if the tree is identical for every
file the probe compiles". B1's main tree was `e5a09a8` (= `784582d` + `c0e5907`), and
this branch's base is `48d9c1b`:

```
$ git diff --stat e5a09a8 48d9c1b -- . ':!research'
(no output)
```

So every tracked file outside `research/`, `Cargo.lock` included, is identical.
The probe change switches `integrate_with` → `adapt_grids_budget(Budget::Fixed,
BlockAllocation::ByAlpha, …)`, which is the call `adapt_grids` makes. I checked it as
order-preserving by rerunning ByAlpha seeds 20260719 and 11 at 600k with this session's
binary:

```
fbn-P1-alphacheck.tsv:   alpha 600000 20260719 1.692984697e-6 3.231842e-9 1011.5 13058820 584
                         alpha 600000 11       1.692529615e-6 2.922915e-9  956.2 12967695 586
t26-B1-main.tsv (B1):          600000 20260719 1.692984697e-6 3.231842e-9  779.4
                               600000 11       1.692529615e-6 2.922915e-9  736.2
$ cmp <(B1 trace lines, those two seeds, sorted) <(this session's trace, first 11 fields, sorted)
TRACE-IDENTICAL     (9 840 lines: every channel × iteration I, V, n, accepted, α, max weight)
```

The same check at 1.2M (`fbn-P1-alphacheck12`) reproduces B1's lines as well:
20260719 gives 1.696160317e-6 ± 2.973708e-9 and 11 gives 1.694481593e-6 ± 3.799538e-9,
and the 9 840 trace lines are again byte-identical.
`ana.py` also rebuilds each iteration's floor from the cumulative acceptance
(`floor_for_acceptance` in Python) and asserts it against the floor the instrumentation
printed. It holds on every line of this session's traces, which is what licenses the
rebuilt floors on B1's trace. The control's statistics in the ladder table are the replay
of B1's main trace, and they match B1's printed summary (mean 1.689996e-6, sd 2.69e-9,
χ²/dof 0.79; 1.2M 3.34e-9, 1.44).

**Transformation class.** For ByAlpha the probe change is order-preserving, checked bit
for bit above. Neyman is a different sampler: its gate is statistical, on 20 seeds per
rung, read as paired differences.

## Method

Probe change (committed): `VG_2TO6_ALLOC` (`alpha` / `neyman` / both), `VG_2TO6_WATCH`
(channel indices). The TSV gains columns `alloc points share_floor_bound watch`, and a
file with other columns is refused. Allocation appears in every progress and summary
line. The adjacent-rung line is now the paired per-seed difference, with its sd and the
sd ratio. `share_floor_bound` is the public report's `floor_bound_channels`, the last
iteration's α share at or under the floor. The per-iteration counts in this report come
from the trace, because `ConvergenceReport` exposes no per-iteration split (Found 3).

Instrumentation (not committed; `$SP/fbn-P1-instrumentation.patch`): B1's patch
(`max_weight` on `BlockIteration`, and a `VG_TRACE` line per channel per iteration in
`integrate_channels`), extended with this iteration's floor, the α share and the rule.
Neither changes a draw. The two ByAlpha seeds' traces are byte-identical to B1's, and
every Neyman seed's replay matches its TSV σ to ≤ 2.9e-10 relative.

```
SP=/tmp/claude-0/-home-user-vibegraph/c98eb2f4-0e99-502e-b316-ad96baf4894f/scratchpad
CARGO_INCREMENTAL=0 pixi run cargo test --config 'profile.release-debug.debug=0' -p vibegraph-lib \
  --profile release-debug --features extended-validation --test validate_sigma --no-run
# fbn-P1-run.sh <tag> <alloc> <rungs> <seeds>, each launched with nohup setsid:
#   VG_TRACE=$SP/fbn-P1-<tag>-trace.txt VG_2TO6_ROWS=bbx VG_2TO6_SAMPLERS=multi VG_2TO6_ALLOC=<alloc>
#   VG_2TO6_WATCH=551,613 VG_2TO6_RUNGS=<rungs> VG_2TO6_SEEDS=<seeds> VG_2TO6_OUT=$SP/fbn-P1-<tag>.tsv
#   target/release-debug/deps/validate_sigma-d2e9a0a9766c990d --ignored --nocapture --exact probe_2to6_budget_ladder
fbn-P1-run.sh alphacheck   alpha  600000          20260719,11
fbn-P1-run.sh neyman       neyman 600000,1200000  20260719,11,22
fbn-P1-run.sh neyman       neyman 600000,1200000  33,…,209     # killed by the container restart at ~21:28Z after 600k seed 99
fbn-P1-run.sh neyman       neyman 600000          110,…,209    # resumed; 600k/110's 4 305 partial trace lines dropped first
fbn-P1-run.sh neyman       neyman 1200000         33,…,209
fbn-P1-run.sh alphacheck12 alpha  1200000         20260719,11
python3 -I $SP/fbn-P1-scripts/ana.py    neyman=$SP/fbn-P1-neyman-trace.txt:$SP/fbn-P1-neyman.tsv
python3 -I $SP/fbn-P1-scripts/ana.py    alphaB1=$SP/t26-B1-main-trace.txt:$SP/t26-B1-main.tsv
python3 -I $SP/fbn-P1-scripts/cross.py  $SP/t26-B1-main-trace.txt $SP/fbn-P1-neyman-trace.txt
python3 -I $SP/fbn-P1-scripts/shift.py  $SP/t26-B1-main-trace.txt $SP/fbn-P1-neyman-trace.txt
python3 -I $SP/fbn-P1-scripts/extra.py  $SP/t26-B1-main-trace.txt $SP/fbn-P1-neyman-trace.txt
python3 -I $SP/fbn-P1-scripts/plainarm.py <trace> <label>
python3 -I $SP/fbn-P1-scripts/neyshare.py $SP/fbn-P1-neyman-trace.txt <neval> <seed>
```

Each seed-run appears exactly once in the Neyman TSV (40 lines) and trace (40 × 615 × 8
lines). Neyman run: 2026-10-23T17:33Z to 2026-10-24T05:08Z, with the restart gap.

## Per-seed values

σ ± quoted error in pb. The ByAlpha values are B1's main-tree TSV, and the Neyman values
are `fbn-P1-neyman.tsv`. The last three columns are Neyman only: kept points over
iterations 2–7 for the watched channels, and the wall time.

| seed | α 600k σ ± err | N 600k σ ± err | α 1.2M σ ± err | N 1.2M σ ± err | N 551 kept pts 600k / 1.2M | N 613 kept pts 600k / 1.2M | N wall s 600k / 1.2M |
|---|---|---|---|---|---|---|---|
| 20260719 | 1.692985e-06 ± 3.23e-09 | 1.689622e-06 ± 2.51e-09 | 1.696160e-06 ± 2.97e-09 | 1.688976e-06 ± 1.70e-09 | 12288 / 74933 | 12351 / 22177 | 938 / 1294 |
| 11 | 1.692530e-06 ± 2.92e-09 | 1.692840e-06 ± 3.20e-09 | 1.694482e-06 ± 3.80e-09 | 1.688336e-06 ± 1.55e-09 | 12288 / 14433 | 12387 / 23777 | 974 / 1328 |
| 22 | 1.687680e-06 ± 2.26e-09 | 1.687805e-06 ± 2.18e-09 | 1.693235e-06 ± 4.95e-09 | 1.691115e-06 ± 3.21e-09 | 12288 / 12288 | 8335 / 14073 | 989 / 1421 |
| 33 | 1.695843e-06 ± 3.98e-09 | 1.691943e-06 ± 2.98e-09 | 1.690116e-06 ± 2.07e-09 | 1.689219e-06 ± 1.58e-09 | 12634 / 24358 | 12760 / 26350 | 881 / 1132 |
| 44 | 1.691714e-06 ± 3.36e-09 | 1.688324e-06 ± 2.65e-09 | 1.689361e-06 ± 1.79e-09 | 1.690325e-06 ± 2.24e-09 | 12288 / 12288 | 7704 / 13274 | 902 / 1162 |
| 55 | 1.687840e-06 ± 2.39e-09 | 1.683297e-06 ± 2.34e-09 | 1.701477e-06 ± 1.20e-08 | 1.688042e-06 ± 1.64e-09 | 12288 / 16529 | 11118 / 22611 | 915 / 1179 |
| 66 | 1.693903e-06 ± 4.08e-09 | 1.686439e-06 ± 2.31e-09 | 1.688422e-06 ± 1.98e-09 | 1.686149e-06 ± 1.49e-09 | 12288 / 15602 | 18089 / 33228 | 926 / 1121 |
| 77 | 1.687743e-06 ± 2.29e-09 | 1.687339e-06 ± 2.19e-09 | 1.693021e-06 ± 2.43e-09 | 1.688364e-06 ± 1.55e-09 | 12288 / 12288 | 11120 / 19471 | 956 / 1091 |
| 88 | 1.685365e-06 ± 2.87e-09 | 1.688994e-06 ± 2.91e-09 | 1.688976e-06 ± 1.76e-09 | 1.690437e-06 ± 2.48e-09 | 12288 / 12288 | 8542 / 14152 | 836 / 1325 |
| 99 | 1.688616e-06 ± 2.48e-09 | 1.689011e-06 ± 2.25e-09 | 1.692527e-06 ± 2.08e-09 | 1.690184e-06 ± 2.26e-09 | 12288 / 12288 | 12325 / 21547 | 978 / 1288 |
| 110 | 1.686504e-06 ± 2.20e-09 | 1.684418e-06 ± 2.06e-09 | 1.690776e-06 ± 1.69e-09 | 1.687974e-06 ± 1.60e-09 | 12288 / 12288 | 13156 / 27293 | 844 / 1102 |
| 121 | 1.687289e-06 ± 3.01e-09 | 1.686536e-06 ± 2.16e-09 | 1.688685e-06 ± 1.77e-09 | 1.688795e-06 ± 1.60e-09 | 25768 / 41757 | 17430 / 29889 | 761 / 1104 |
| 132 | 1.689649e-06 ± 2.36e-09 | 1.688558e-06 ± 2.36e-09 | 1.691047e-06 ± 1.87e-09 | 1.689655e-06 ± 1.72e-09 | 12316 / 20991 | 11339 / 21603 | 828 / 1080 |
| 143 | 1.690404e-06 ± 3.12e-09 | 1.690991e-06 ± 3.11e-09 | 1.694753e-06 ± 1.91e-09 | 1.692296e-06 ± 1.73e-09 | 12288 / 13017 | 12830 / 23838 | 771 / 1036 |
| 154 | 1.691603e-06 ± 3.50e-09 | 1.686458e-06 ± 2.61e-09 | 1.689614e-06 ± 1.98e-09 | 1.687296e-06 ± 1.59e-09 | 12288 / 12288 | 9720 / 19101 | 825 / 1194 |
| 165 | 1.691323e-06 ± 3.68e-09 | 1.684504e-06 ± 2.08e-09 | 1.686988e-06 ± 1.90e-09 | 1.693786e-06 ± 4.77e-09 | 12288 / 12288 | 16574 / 18611 | 760 / 1231 |
| 176 | 1.690794e-06 ± 3.04e-09 | 1.691217e-06 ± 2.81e-09 | 1.693102e-06 ± 3.96e-09 | 1.689596e-06 ± 1.81e-09 | 12288 / 13409 | 14943 / 22877 | 757 / 1121 |
| 187 | 1.690985e-06 ± 3.84e-09 | 1.687221e-06 ± 2.46e-09 | 1.690276e-06 ± 2.00e-09 | 1.688824e-06 ± 1.74e-09 | 12288 / 12288 | 45734 / 102581 | 824 / 1090 |
| 198 | 1.687732e-06 ± 3.10e-09 | 1.692749e-06 ± 3.09e-09 | 1.691925e-06 ± 3.03e-09 | 1.688652e-06 ± 1.61e-09 | 12288 / 13682 | 12292 / 80163 | 897 / 1088 |
| 209 | 1.689423e-06 ± 3.26e-09 | 1.687445e-06 ± 2.28e-09 | 1.688913e-06 ± 1.88e-09 | 1.687604e-06 ± 1.56e-09 | 12288 / 12288 | 11742 / 23822 | 768 / 1066 |

## Validation

Committed: the probe change and this report (with the regenerated sessions index). No
production code is committed. The instrumentation was reverted before the gates, and
`git diff --stat` shows only `vibegraph-lib/tests/validate_sigma.rs` and the two kb
files. The gates ran on that tree with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`:

```
$ pixi run cargo fmt --all --check                                         -> exit 0
$ pixi run cargo clippy --workspace --all-targets -- -D warnings           -> Finished, exit 0
$ pixi run cargo clippy --workspace --all-targets \
    --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings  -> exit 0
$ pixi run cargo test --workspace                                          -> exit 0
  47 test-result lines: 1646 passed, 0 failed, 19 ignored
$ pixi run --skip-deps kb-lint
  923 concepts, 299 backlog items; 0 errors, 4 warnings
```

All four warnings are pre-existing `state: blocked` backlog items, none of them touched
here. `pixi run --skip-deps validate` was not run: no amplitude, colour, coupling or
evaluator code changed. The probe is `#[ignore]` and only calls the public
`adapt_grids_budget`.

## Brief corrections

- **"`probe_2to6_budget_ladder` passes `BlockAllocation::ByAlpha` to
  `adapt_grids_budget`"**: it called `integrate_with` → `FixedBeamIntegrand::adapt_grids`,
  which fixes `ByAlpha` inside `integrate_channels`. The probe now calls
  `adapt_grids_budget` itself.
- **"Run under both allocations on this tree"**: the ByAlpha arm is B1's main-tree run,
  reused under the brief's own exception and checked bit for bit (Control). The
  coordinator's note after the restart listed the 20-seed ByAlpha control as missing. It
  is not missing; it is substituted, with the evidence above.
- **"Record per seed … the number of floor-bound channels per iteration"**: the public
  `ConvergenceReport` has only the last iteration's α-share count. The per-iteration
  numbers here come from the uncommitted trace, and the committed TSV carries what the
  public API offers.
- **The sd-ratio success criterion is not sufficient.** A rule that sets iteration
  counts from earlier outcomes can meet "sd ×1/√2, χ²/dof near 1" through a biased
  combination. The criterion needs a paired-mean check against an outcome-independent
  arm beside it.

## Found

1. **The point-weighted iteration combine is biased low when the counts come from
   earlier outcomes.**
   - Evidence: on bbx under Neyman, −1.71e-9 pb (600k) and −2.41e-9 pb (1.2M) against
     ByAlpha, 20 paired seeds, t −2.40 and −2.77. That is 0.10–0.14 % of σ, against
     `ChannelHistory::combine`'s documented "0.26–0.40 σ with no common sign". The plain
     mean of the same iterations removes it (t −0.33, −0.23).
   - The shift sits in channel-runs whose largest kept iteration drew fewer than their
     mean points.
   - It also shrinks the seed spread and the quoted error, so a ladder reads it as a gain.
   - The CLI's target mode defaults to Neyman and runs this combine, and the
     `BlockAllocation::Neyman` documentation's target-mode figures were measured with it.
   - To close it: an outcome-independent combination for unequal counts. That could be
     the plain mean, inverse-variance weights fixed in advance, or weights from a count
     schedule declared before the run. Then re-measure Neyman's target-mode figures
     (p p > l+ l- j, 8 seeds) and this ladder under it.
2. **Floor points dominate the 2 → 6 budget.** At 1.2M, 51 % of each iteration's 2.32M
   points sit in the floors (49 % above), under either rule, and 551–580 of 615 channels
   draw identical histories at both rungs. To close it: the scaling-floor proposal
   above, evaluated with Found 1's combine.
3. **`ConvergenceReport` cannot show the per-iteration allocation.** The probe can record
   only the last iteration's α-share floor count, and nothing of what Neyman did. To
   close it: expose per-iteration totals of points, floor-bound channels and points
   above the floors (or the per-channel points and floors) on `ConvergenceReport`. Then
   the probe's TSV can carry the counts this report took from a throwaway trace.
4. **bbx's unbiased σ moves away from MadEvent with budget.** Measured against the
   20-seed reported reference with the measured spread: ByAlpha +2.46 → +3.47 σ, and
   Neyman's plain mean +2.17 → +3.12 σ. Against equal-weight: +1.37 → +2.32 and +1.17 →
   +2.07. The paired rung difference is +1.7e-9 under both (t +1.69, +1.36). Each step is
   sub-3σ, but the trend is common to both rules. To close it: a third rung (2.4M) on
   20 seeds read as a paired difference, or the channel-level comparison against the
   MadEvent per-channel record. The references' own scatter (D1's third amendment)
   limits what a pull can say.
