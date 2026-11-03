---
type: Session Report
title: "T-V2 report: the CMS VBF/EWK σ has not converged at affordable budgets; a banked σ-level spine gate"
description: "At about 1.1 ms a point, a 10-hour campaign reaches DY MLL-50 at two nested rungs (20 and 12 seeds) and the two VBF cards at one rung (5 seeds). The typical DY seed still climbs between the rungs (paired +0.54 ± 0.27 pb), seed χ²/dof is 3.5–14, and a γγ → ℓℓ corner carries 75% of one sample's weight. The pass condition is not met. A banked 16-seed σ test on u c > e+ e- u c, calibrated on 40 seeds per arm, fails with the spine withheld."
status: draft
tags: [phase-space, spine, vbf, cms, budget-ladder, sprint-e, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-29}
---
The session brief is [T-V2](T-V2.md). The campaign plan below was written and
committed (`ba9df8cf`) before the long runs were launched. What actually ran
follows it.

**Verdict.** The item's pass condition is **not met**. On DY MLL-50 the 5-seed
median has not stopped moving between the two rungs. The seeds disagree with
their own errors (χ²/dof 3.5–4.4), and a per-event check finds a region of
small measure that the grids sample at w/w_max ≈ 1500. The VBF cards have
one rung each, so a ladder cannot be read on them. No residual against
MadGraph can be quoted as a convergence result. All that can be said is that
the readings are within a few percent of MadGraph, with the numbers below.

## Campaign plan (written before launch)

**Build.** `/home/user/wt/TV2` at `3b0d9363` (#81 `a09a7806` merged with #82
`da56ab77`), `cargo build --profile release-debug -p vibegraph`. Model: a
scratchpad copy of `research/refs/mg5amcnlo/models/loop_sm` (submodule at
`b7687064`) plus the gridpacks' `restrict_ckm_no_b_mass.dat` (identical in all
three). The three `param_card.dat` are byte-identical. PDF 325300 was already
in the `~/.vibegraph` cache.

**Command per run** (driver `T-V2-run.sh`, one result line per finished run,
skips recorded tags):

```
vibegraph --no-tui -v -y integrate <card>/proc_card.dat --run-card <card>/run_card.dat \
  --param-card <card>/param_card.dat --ufo-dir T-V2-ufo --out <dir> --force \
  --seed <S> -j 4 --fixed-budget --neval <N> --niter <K>
```

**Measured pilots** (seed 1, `--neval 10000 --niter 4`, 4 threads, nothing
else running):

| card | channels (massless + τ part) | wall | σ (pb) | MG σ (pb) |
|---|---|---|---|---|
| DYto2L-2Jets-EWK_Bin-MLL-50 | 68 + 74 | 527 s | 12.03 ± 0.56 | 13.12 ± 0.031 |
| VBFto2L_Bin-MLL-50 | 68 + 68 | 533 s | 8.50 ± 0.75 | 7.574 ± 0.030 |
| VBFto2L_Bin-MLL-4to50 | 68 + 68 | 461 s | 104.7 ± 7.0 | 117.6 ± 0.44 |

Where the time goes (DY): 9 s enumeration and compilation (766 subprocesses);
130 s for the two α-surveys (6 × 10⁴ points each, the survey's minimum); then
about 99 s per VEGAS iteration. One iteration is about 8–9 × 10⁴ points,
because 139 of the 142 channels sit at the 512-point floor whatever `--neval`
is below that. The cost is about 1.1 ms per point at 4 threads, the same as
the CERN host measured.

**What this allows.** About 1.1 ms a point puts 10 hours at about 3 × 10⁷
points, all runs together. The brief's design (20 + 20 seeds on DY, then 5
seeds × 2+ rungs × 3 cards) needs about 21 hours at the cheapest well-formed
rungs. So the plan keeps the brief's priority order and drops the second
VBF rung:

- **Rungs.** `R1` = `--neval 10000 --niter 3`: the 512-point floor, two
  warm-up iterations, one measured. `R4` = `--niter 12`: 4× R1's VEGAS points
  (12 iterations against 3), ten measured. `--neval` stays at the floor
  because raising it only re-weights the few channels above the floor, and
  it raises the survey to 4 × 10⁴ points a part.
- **Estimated wall per run** (from the pilot, one iteration less or eight
  more): DY R1 ≈ 430 s, DY R4 ≈ 1320 s, VBF MLL-50 R1 ≈ 415 s, VBF MLL-4to50
  R1 ≈ 335 s.

| phase | runs | estimate |
|---|---|---|
| 1. budget study, DY R1, seeds 1–20 | 20 | 2.4 h |
| 2. ladder R1, VBF MLL-50 and VBF MLL-4to50, seeds 1–5 | 10 | 1.0 h |
| 3. budget study, DY R4, seeds 1–20 | 20 | 7.3 h |
| total | 50 | 10.7 h |

- **Not planned:** R4 on the two VBF cards (10 runs, about 3.6 h), and any
  rung above R4. DY's ladder is phases 1 and 3 (its first five seeds a rung,
  and all twenty).
- **Order.** Phase 2 goes before phase 3 so that every card has a reading
  early. The budget study is still complete before anything else is added.
- **Shared CPU.** The σ-level test is calibrated and gated on
  `/home/user/wt/T` while the campaign runs. That slows some runs; the
  recorded wall per run says by how much. It does not change any σ: a fixed
  budget at a fixed seed gives the same artifact at any thread count.

## What actually ran

| phase | planned | ran | wall |
|---|---|---|---|
| pilots (seed 1, niter 4, all three cards) | 3 | 3 | 14:22–14:57 (with two aborted sizing runs) |
| 1. DY R1, seeds 1–20 | 20 | 20 | 14:58–17:45, 386–935 s a run |
| 2. VBF MLL-50 and MLL-4to50 R1, seeds 1–5 | 10 | 10 | 17:45–18:43, 294–393 s a run |
| 3. DY R4 | seeds 1–20 | **seeds 1–12** | 18:43–00:20, 1106–2514 s a run |

- **Total wall time:** 14:22 to 00:20, 10.0 h. The campaign itself ran
  14:58 to 00:20, 9.4 h.
- **Deviation: phase 3 stopped at 12 seeds, not 20.** From about 19:50 to
  23:00 a sibling session's banked suite (`/home/user/wt/S`, load average 6–14)
  shared the 4 cores. That pushed R4 runs from 1262 s to 1700–2500 s, so 20
  seeds would have ended near 04:00. Seeds 13–20 are recorded in the results
  file as `SKIPPED`. The skips were set when the slowdown was seen, and
  11–12 were re-admitted once the host was free again.
- The calibration and gates on `/home/user/wt/T` also shared the cores, from
  about 15:00 to 16:00. Their effect shows in DY R1 seeds 1–5 (572–935 s,
  against 386–441 s later).
- **σ does not depend on any of this.** A fixed budget at a fixed seed gives
  the same artifact whatever the load or thread count. DY seed 1's
  iteration 3 reads 3.074097e-8 ± 2.395884e-9 in the pilot, in R1 and in R4
  alike.

**Moved parameters** the artifacts record (every run, all three cards; the
`params:` line of `integrate`):
`aEWM1 = 128.8253, Gf = 0.00001166379, ymt = 172.5, MT = 172.5, MZ = 91.15351, WT = 1.311, WZ = 2.493202, WW = 2.085, WH = 0.006382339`.
- MW is computed by the model and is not in the list. No warning was printed,
  so the card's 79.90685 agrees with the computed value at the card's
  precision.
- WH is recorded as moved: the card's 6.382339e-3 differs from the
  restriction default.
- The card writes MZ as 9.115351e+01. MadGraph read the same card, so it used
  the same seven-digit value, not the customize card's 91.153509740726733.

## Budget study (DY MLL-50; MG 13.12 ± 0.031 pb)

The rungs are **nested**. At a seed, R4's first three iterations are R1's,
bit for bit, so the rung difference per seed is a paired quantity.

| rung | seeds | mean (pb) | median | seed sd | sd/√n | mean quoted err | sd / quoted | seed χ²/dof | mean/MG | median/MG |
|---|---|---|---|---|---|---|---|---|---|---|
| R1 (10⁴ × 3) | 1–20 | 12.74 | 12.38 | 1.72 | 0.38 | 1.26 | 1.37 | 4.37 | 0.971 ± 0.029 | 0.944 |
| R4 (10⁴ × 12) | 1–12 | 12.90 | 12.88 | 0.64 | 0.18 | 0.55 | 1.17 | 3.45 | 0.983 ± 0.014 | 0.982 |

Per seed (pb), R1 → R4:
1: 11.97 → 12.58, 2: 14.94 → 14.06, 3: 11.80 → 11.93, 4: 14.00 → 13.24,
5: 12.13 → 12.09, 6: 11.47 → 12.63, 7: 11.35 → 12.58, 8: 12.94 → 13.13,
9: 10.68 → 13.23, 10: 11.55 → 12.35, 11: 12.98 → 13.51, 12: 12.49 → 13.42.
R1 seeds 13–20: 11.56, 12.45, 13.58, **18.10 ± 6.0**, 12.73, 10.93, 14.74,
12.32.

Paired over seeds 1–12, R4 − R1:

| quantity | R1 mean / median | R4 mean / median | paired difference |
|---|---|---|---|
| total | 12.36 / 12.05 | 12.90 / 12.88 | **+0.54 ± 0.27** (sd 0.93) |
| massless part | 8.46 / 8.43 | 8.60 / 8.40 | +0.14 ± 0.15 |
| τ part | 3.90 / 3.60 | 4.30 / 4.26 | **+0.40 ± 0.16** |

Reading it against the measured spread:
- **The quoted errors understate the spread at both rungs** (sd/quoted 1.37
  and 1.17, seed χ²/dof 4.4 and 3.5). The R1 seeds are heavy-tailed: seed 16
  reads 18.1 ± 6.0, and seeds 2, 4 and 19 read 14–15 with errors of 1.8–2.5.
- **The failure signature is present.** Ten of twelve seeds rise from R1 to
  R4. The two that fall (2 and 4) are R1's high outliers. That is the typical
  seed climbing while the tail is trimmed, the pattern of a region found more
  often at the larger budget. The paired climb is 2.0σ on the total and 2.5σ
  on the τ part.
- **A 5-seed ladder could not resolve convergence at these rungs.** R4's
  measured per-seed sd is 0.64 pb, so five seeds resolve the median to about
  0.3–0.35 pb (2.5%). The median moved by 0.83 pb between R1's seeds 1–12 and
  R4's. R4's 5-seed median (12.58) and its 12-seed median (12.88) differ by
  about one 5-seed resolution.
- **R4 against MadGraph:** 0.983 ± 0.014 on the total (pull −1.2 on the
  spread). The parts move in opposite directions: massless 8.60 ± 0.15
  against MadGraph's P0_qq_llqq 9.02 ± 0.07 (0.953), and τ 4.30 ± 0.10
  against P0_qq_taptamqq 4.05 ± 0.05 (1.062). These errors are spread-based,
  from 12 seeds at χ²/dof 3.5, and the τ part is still climbing. Neither part
  is a converged residual.

## Ladder on the two VBF cards (R1 only, 5 seeds)

| card | MG σ (pb) | mean (pb) | median | seed sd | sd/√5 | seed χ²/dof | mean/MG | median/MG |
|---|---|---|---|---|---|---|---|---|
| DY MLL-50 (R1, seeds 1–5) | 13.12 ± 0.031 | 12.97 | 12.13 | 1.41 | 0.63 | 1.94 | 0.988 ± 0.048 | 0.924 |
| DY MLL-50 (R4, seeds 1–5) | 13.12 ± 0.031 | 12.78 | 12.58 | 0.88 | 0.39 | 6.16 | 0.974 ± 0.030 | 0.959 |
| VBF MLL-50 | 7.574 ± 0.030 | 7.875 | 7.906 | 0.925 | 0.414 | 2.02 | 1.040 ± 0.055 | 1.044 |
| VBF MLL-4to50 | 117.6 ± 0.44 | 131.6 | 110.9 | 46.6 | 20.9 | 14.21 | 1.119 ± 0.177 | 0.943 |

Per seed (pb):
- VBF MLL-50: 8.25 ± 1.0, 6.53 ± 0.51, 9.06 ± 1.3, 7.91 ± 1.7, 7.63 ± 0.94.
- VBF MLL-4to50: 93.1 ± 8.9, 95.1 ± 6.4, 158.8 ± 19, 110.9 ± 12,
  **200.3 ± 101**.

Per part (mean of 5 seeds against MadGraph's per-P σ):

| card | massless / P0_qq_llqq | τ / P0_qq_taptamqq |
|---|---|---|
| VBF MLL-50 | 4.98 / 5.39 (0.923) | 2.90 / 2.17 (1.334) |
| VBF MLL-4to50 | 100.7 / 88.5 (1.138) | 30.9 / 29.8 (1.038) |

- **VBF MLL-4to50 is the worst-behaved card.** Seed χ²/dof is 14.2, and one
  seed reads 200 ± 100. The pilot's last iteration read iteration χ²/dof 55.7.
  94 of its 136 channels accept less than one point in four. The DY and
  VBF MLL-50 runs print no such warning.
- **The VBF MLL-50 τ part reads 1.33 of MadGraph (+3.3σ on 5 seeds).** The τ
  part is also +6% on DY R4. One hypothesis was checked and ruled out: the
  `$$ h` card's diagram counts are 48/35/48 for τ, equal to the electron
  counts, so the h → ττ diagrams are removed as MadGraph's `$$` removes them.
  The DY `$ h` card has 50/37/50. Nothing else was checked; see Found.
- The VBF medians span 0.94–1.04 of MadGraph. With one rung, they say
  nothing about convergence.

## What the brief's pass condition needs, and what would close it

- **Size of the residual.** At R4 on DY it is −1.7% ± 1.4% on the total, but
  −4.7% on the massless part and +6.2% on the τ part, with seed χ²/dof 3.5 and
  a paired climb still under way. On the VBF cards, at R1, it is +4% ± 5.5%
  and +12% ± 18%.
- **Consistency across cards.** None can be established at one rung. The
  only shared feature is the heavy tail: seed χ²/dof is above 2 on every
  card at every rung.
- **Next per-event checks**, in order:
  1. The γγ → ℓℓ corner below. Locate the overweight event's point in its
     channel. Compare that channel's density with |M|² there. Then rerun R1 on
     DY with the pole floor moved (a `DEFAULT_POLE_SCALE` scan, as T-F1 did on
     `u c`). If the climb is this corner being under-mapped, the R1 median
     should move with the floor.
  2. Pin the per-beam μF against MadGraph's own record on a banked
     `p p > l+ l- j j` run with the dynamical scale (T-V1's Found). This
     corner sits at quark pT 2–6 GeV, right above the floor.
  3. The τ part: a per-part ladder on a τ-only card (`p p > ta+ ta- j j`),
     where the 5-seed spread is smaller.
- **Budget.** A converged ladder on these cards needs a much larger budget:
  R16 (48 iterations, about 80 min a run) or higher, at 20 seeds on two rungs.
  That is 25–50 h on this host, or a host with more cores.

## The committed σ-level test

Commit `9e7ab447` on `sprinte-spine-fallback`.

- **Where.** `vibegraph-lib/tests/spine_sigma.rs`, a new test target with
  `required-features = ["extended-validation"]`, so it is in the banked layer
  that `pixi run validate` (`validation/validate.sh`) runs under
  `--profile release-debug`. It needs no reference data, only the interned
  SM: banked for cost, like `rambo_flat_mc`. `validation/manifest.toml` has a
  `[[standalone]] key = "spine-sigma"` row for it, with its blind spot.
- **What.** `u c > e+ e- u c QCD=0`, fixed beams at √ŝ = 1 TeV,
  `ptj = ptl = 0`, `mmjj > 300`, `mmll > 50` (T-F1's card). All 10 channels
  are spines. `FixedBeamIntegrand` with `use_multichannel(…, 30000, 6, seed)`
  and `adapt_grids(200000, 8, seed)`, as T-F1's production probe. Sixteen
  seeds (11, 22, …, 176) combined by `common::seeds::combine` (unweighted
  mean).
- **Assertions.** Relative spread (sd/√16 over the mean) ≤ 3.0e-3, then seed
  χ²/dof in [0.15, 4.0].
- **Blind spot.** A bias every seed shares. There is no independent σ for
  this process (flat RAMBO does not converge on it, T-F1), so a map that is
  wrong the same way on every seed passes. The walk/density test in
  `diagram_channel.rs` covers that.

### Calibration

The ignored probe `probe_the_seed_scatter_of_the_no_scale_spine` runs 40
seeds (1001–1040), disjoint from the gated ones. It was run on the tree as
committed and under M1. Sixteen-of-40 quantiles come from 20 000 random
draws without replacement (`T-V2-calib.py`).

| arm | 40-seed mean (pb) | per-seed rel. sd | sd / quoted | 16 of 40: rel. spread, 0.1% / 50% / 99.9% | 16 of 40: χ²/dof, 0.1% / 50% / 99.9% |
|---|---|---|---|---|---|
| spine (as committed) | 1.79781e-2 | 3.29e-3 | 0.91 | 4.5e-4 / 8.2e-4 / 1.11e-3 | 0.25 / 0.85 / 1.57 |
| spine withheld (M1) | 1.80991e-2 | 1.27e-1 | 1.46 | 7.5e-3 / 2.9e-2 / 4.4e-2 | 0.38 / 2.36 / 7.17 |

- The **spread bound separates the arms**, with a factor of 2.7 to the spine
  arm's 99.9% quantile and 2.5 to M1's 0.1% quantile.
- **χ²/dof alone does not.** The no-spine estimator quotes errors nearly as
  wide as its scatter on most seeds, so its median χ²/dof is 2.4. The band is
  kept for what it does catch: seeds that are not independent (low edge), and
  the confidently-wrong signature (high edge). It failed M1's gated seeds too
  (5.74).
- The spine arm's resampled quantiles understate a tail, because they are
  drawn from 40 values. The first eight gated seeds scatter by 5.4e-3 per
  seed, against 3.3e-3 over the 40. An eight-seed version of this test read a
  relative spread of 1.91e-3, the top of its resampled range. So the gate
  uses sixteen seeds, and the bound keeps a factor of 2.9 over the gated
  value.

### Gate run, and mutation M1

Command (on `9e7ab447`'s tree, campaign running alongside):
`cargo test -j 2 -p vibegraph-lib --profile release-debug --config 'profile.release-debug.debug=0' --features extended-validation --test spine_sigma -- --nocapture`

```
u c > e+ e- u c QCD=0 at √ŝ = 1000 GeV, 200000 × 8 a seed: 10 peripheral channels of 10
  16 seeds: mean 1.799259e-2 pb, spread 1.88e-5 (rel 1.05e-3), seed χ²/dof 1.06, 81 s
test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 81.42s
real 2m8.677s   user 3m37.886s
```

**Runtime:** 81 s of test time, with the 4-core host shared with a
4-thread campaign run, and 3.6 min of CPU. The earlier eight-seed run took 41 s
under the same sharing, and the 40-seed probe took 194 s (about 5 s a
seed), also shared. No uncontended timing was taken.

**M1**, the spine withheld: restored
`&& (n_out == 2 || fiducial_scale > F::zero())` in `from_diagram_with`. The
same command:

```
u c > e+ e- u c QCD=0 at √ŝ = 1000 GeV, 200000 × 8 a seed: 0 peripheral channels of 10
  16 seeds: mean 1.829004e-2 pb, spread 6.74e-4 (rel 3.68e-2), seed χ²/dof 5.74, 139 s
thread '…' panicked at vibegraph-lib/tests/spine_sigma.rs:210:5:
the seeds' relative spread 3.68e-2 exceeds 3.0e-3
test result: FAILED. 0 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 138.75s
```

- The M1 seeds show the confidently-wrong signature: seed 110 reads
  1.553e-2 ± 5.6e-4 pb, 14% low with a 3.6% error.
- Seeds 11–55 reproduce T-F1's M1 row exactly (2.066886e-2, 1.714126e-2, …).
- The source was restored from a backup, and `cmp` reported it identical.

## The μF floor edge, per event (DY MLL-50)

`vibegraph generate T-V2-runs/dy-r1-s1/grid.bin.zst --seed 7 --nevents 600 -j 1`
on DY's R1 seed-1 grid. This run took 24 min, at an unweighting efficiency of
1.85e-3. A first try at 2000 events was stopped after 15 min. Histogram:
`T-V2-muf.py`.

The LHE record holds only `SCALUP = max(μF₁, μF₂)`. The veto acts on each
beam's μF, and no per-beam μF is written. So two readings are shown: SCALUP,
and the softest outgoing quark's pT, which stands in for the clustering scale
of the beam that quark is attached to.

| bin (GeV) | events, SCALUP | events, softest-quark pT |
|---|---|---|
| 0–1 | 0 | 1 |
| 1–1.5 | 0 | 3 |
| 1.5–1.8 | 0 | 1 |
| 1.8–1.9 | 0 | 0 |
| 1.9–2 | 0 | 0 |
| 2–2.1 | 0 | 11 |
| 2.1–2.2 | 0 | 11 |
| 2.2–2.5 | 2 | 21 |
| 2.5–3 | 5 | 39 |
| 3–4 | 9 | 58 |
| 4–6 | 27 | 63 |
| 6–10 | 60 | 69 |
| 10–∞ | 497 | 323 |

- **The edge is at 2 GeV.** There are no events in 1.8–2.0 GeV, then 11
  events in each of the next two 0.1 GeV bins. The minimum SCALUP is
  2.219 GeV.
- **Five events lie below 2 GeV in the proxy, and none of them crosses the
  floor.** All five have a same-flavour `q q̄` final pair with `mjj` of 4–89
  GeV and SCALUP of 92–124 GeV. These are `q q̄ → ℓℓ (Z/γ* → q q̄)` events.
  The soft quark clusters with its partner, not with a beam, so both beam
  scales are at the boson scale and the proxy does not apply. On the CMS
  cards the proxy stands in for a beam scale only on the t-channel
  topologies.
- This is the same edge T-V1 found on `u c > e+ e- u c` alone: 2.0006 GeV
  minimum there.

The same sample is the clearest evidence about convergence; see "Per-event
reading of the tail" below.

## Per-event reading of the tail (DY MLL-50, R1 seed 1)

The 600-event sample above is the finer oracle here. The integration of that
grid read σ = 11.97 ± 0.93 pb. The sample reads 44.3 ± 30.7 pb (`generate`'s
own check: +270%, pull +1.05). Its overweight rate is 1.0e-4, and the
overweight events carry 75% of the sample's σ. The largest event has
w/w_max = 1547.

The heaviest events (event weight share of the sample):

| share | subprocess | quark pT (GeV) | lepton pT (GeV) | mjj (GeV) | mℓℓ (GeV) | SCALUP |
|---|---|---|---|---|---|---|
| 56.3% | `u u > e+ e- u u` | 3.22, 5.09 | 3.92, 5.71 | 975 | 59.5 | 5.1 |
| 7.1% | `u u > ta+ ta- u u` | 2.05, 6.42 | – | 1995 | 61.4 | 6.4 |
| 0.3% | `u u~ > ta+ ta- u u~` | 23.6, 2.45 | – | 677 | 74.6 | 22.9 |

The top event, in detail:
- Both quarks are at |y| > 5, with pT just above the 2 GeV floor.
- The two quark-line transfers are t = −10.7 and −28.9 GeV².
- The leptons are soft (pT 3.9 and 5.7) and far apart in rapidity
  (y = 1.38 and −3.67), with mℓℓ just above the 50 GeV cut.

That is photon–photon fusion, `γγ → ℓ⁺ℓ⁻`. Each quark radiates a
quasi-real photon, and the lepton pair is made by t-channel lepton exchange
between the photons. The ladder has three soft rungs: two photon virtualities
at 10–30 GeV², and a lepton exchange at pT² ≈ 15–30 GeV². All three sit at
or near the spine's pole floor of 10 GeV² (`DEFAULT_POLE_SCALE` × the pole
fraction).

What this shows and what it does not:
- **It shows** that at R1 the grid samples this corner at w/w_max ≈ 1500.
  The integration found almost none of it; the unweighting found it by
  chance. That is a region of small measure carrying a large share of σ: the
  failure AGENTS.md warns a σ sweep is blind to.
- **It does not show** that the corner is mis-mapped, as opposed to
  under-trained. Two seeds of one grid cannot tell those apart. The weight a
  1/|t| integrand gets under a map with density ∝ 1/(|t| + 10 GeV²) only
  reaches a factor of about 2–3 at these virtualities. So the floor alone does
  not explain a factor of 1500: three rungs, plus a VEGAS grid that never saw
  the corner, would be needed.

## Gates (on `/home/user/wt/T`)

These ran on the tree of `9e7ab447`. Later commits touch only
`research/kb/`. Logs: `T-V2-gate-lint.log`, `T-V2-test-ws.log`,
`T-V2-gate-spine16-final.log` in the scratchpad.

```
$ cargo fmt --all --check                                                       FMT_EXIT=0
$ cargo clippy -j 2 --workspace --all-targets -- -D warnings                    CLIPPY_EXIT=0
$ cargo clippy -j 2 --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
                                                                                CLIPPY_EXT_EXIT=0
$ cargo clippy -j 2 -p vibegraph-lib --all-targets --features extended-validation -- -D warnings
                                                                                CLIPPY_EXT_LIB_EXIT=0
$ cargo test -j 2 --workspace --no-fail-fast -- --test-threads 2                TEST_EXIT=0
  45 "test result: ok" lines, 0 FAILED; 1710 passed, 0 failed, 22 ignored
$ cargo test -p vibegraph-lib --profile release-debug --config 'profile.release-debug.debug=0' \
    --features extended-validation --test spine_sigma -- --nocapture          (host idle, 00:21)
  16 seeds: mean 1.799259e-2 pb, spread 1.88e-5 (rel 1.05e-3), seed χ²/dof 1.06, 37 s
  test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 37.07s
$ python3 scripts/kb.py lint
  1002 concepts, 333 backlog items; 0 errors, 4 warnings
```

- **Clippy.** The fourth clippy line was added because `vibegraph/extended-validation`
  is the CLI crate's feature. It is the library's own feature that compiles
  `spine_sigma`.
- **kb lint.** It first failed with 1 error: the sessions `index.md` was stale
  after the plan commit `ba9df8cf`. `python3 scripts/kb.py index` fixed it, in
  `9e7ab447`. The 4 warnings are pre-existing `state: blocked` backlog items.
- **Runtime.** The idle-host run is 37 s, bit-identical to the shared-host run
  (81 s).
- **Not run:** the rest of the banked layer (`pixi` is not installed, and
  `validation/validate.sh` was not run). The change adds a test target and a
  manifest row; it touches no amplitude, map or integrator code.

## Corrections to the brief

- **"Run with `VIBEGRAPH_FETCH_CONSENT=1` once, to fetch it into the cache."**
  It was not needed: 325300 was already in `~/.vibegraph/pdf` from T-V1. The
  CLI's consent flag is `-y`, and every run passed it.
- **"Measure one run per card, then plan a campaign that fits in about 10
  hours."** The brief's design does not fit. At 1.1 ms a point and the
  142-channel 512-point floor (8–9 × 10⁴ points an iteration whatever
  `--neval` is), the cheapest rungs cost 7 and 21 minutes a run. 20 + 20 DY
  seeds plus 5 seeds × 2 rungs × 2 VBF cards come to about 21 h. The plan
  kept the priority order and dropped the second VBF rung.
- **The ladder's "rungs × 4 apart" rule in iterations nests the rungs.** At a
  fixed seed, R4 extends R1, so rung-to-rung comparisons are paired. The
  table reads them that way. Independent rungs would need disjoint seed sets.
- **"until the 5-seed median stops moving within the measured spread."** At
  R4 the measured spread puts the 5-seed median's resolution at about 2.5%.
  So "stopped moving" cannot be read below that, whatever the budget, without
  more seeds.

## Found

- **A γγ → ℓℓ corner dominates the DY tail at R1.**
  - *Evidence:* the 600-event sample from DY R1 seed 1. Overweight events
    carry 75% of the sample's σ, and the top event has w/w_max = 1547. It is
    `u u > e+ e- u u` with both quarks at |y| > 5 and pT 3.2 / 5.1 GeV,
    t = −10.7 and −28.9 GeV², soft leptons, and mℓℓ = 59.5. The sample's σ is
    +270% on its integration's.
  - *To close:* locate the point in its channel and compare density with |M|²
    there. Then scan `DEFAULT_POLE_SCALE` on DY R1, as T-F1 did on `u c`, and
    see whether the R1 → R4 climb follows the floor. Three soft rungs sit at
    or near the 10 GeV² pole floor here.
- **The τ part reads high where the total does not.**
  - *Evidence:* DY R4 τ part 4.30 ± 0.10 against MadGraph's 4.05 ± 0.05, and
    still climbing (+0.40 ± 0.16 pb from R1, paired). VBF MLL-50 R1 τ part is
    2.90 ± 0.22 against 2.17 ± 0.03. The massless parts read low (0.95, 0.92).
    The `$$ h` diagram removal matches MadGraph's (48/35/48 τ diagrams, as for
    electrons).
  - *To close:* a τ-only card (`p p > ta+ ta- j j`, the same run card) at two
    rungs × 20 seeds, against MadGraph's P0_qq_taptamqq. Check the τ part's
    channel set: 74 channels on DY under `$ h` and 68 on VBF under `$$ h`,
    against 68 for the massless part on both. The extra six on DY were not
    identified.
- **The LHE record has no per-beam μF.**
  - *Evidence:* SCALUP is max(μF₁, μF₂). The 2 GeV veto acts on each beam, so
    the floor edge can only be shown through a proxy (softest-quark pT), and
    the proxy is wrong for s-channel `q q̄` pairs (5 of 600 events).
  - *To close:* write MadGraph's per-beam record (the `<mgrwt>`
    `<pdfrwt beam=…>` scale), or a debug dump of `EventScales::mu_f`, so the
    floor can be pinned per event against MadGraph.
- **Per-point cost and the per-channel floor set the ladder's price.**
  - *Evidence:* 1.1 ms a point at 4 threads on these cards (766
    subprocesses, about 3200 (group, diagram) channels). 139 of 142 channels
    sit at the 512-point floor, so `--neval` below about 8 × 10⁴ does nothing.
    The two α-surveys alone cost 130 s a run.
  - *To close:* profile a DY point (`scripts/profile.sh`) to see whether the
    matrix elements or the channel densities dominate. A converged CMS ladder
    needs a 25–50 h campaign, or a bigger host.
- **`generate` is serial and slow on these cards.**
  - *Evidence:* 600 events took 24 min at efficiency 1.85e-3 (324k trials,
    1 thread).
  - *To close:* parallelise the accept/reject pass, or document the cost
    next to `-j` in `generate`.
- **A sibling session's load is not visible to a campaign's plan.**
  - *Evidence:* phase 3's runs took 1.7–1.9× longer while
    `/home/user/wt/S`'s banked suite ran, and that cost 8 of the 20 planned R4
    seeds.
  - *To close (process):* serialise heavy campaigns on a shared container,
    or give each session a core budget.

## Raw result lines (driver output, one per run)

```
pilot-dy-r0 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=1 neval=10000 niter=4 rc=0 wall=527s | σ = 12.029886 ± 0.557276 pb
pilot-vbf50-r0 card=VBFto2L_Bin-MLL-50 seed=1 neval=10000 niter=4 rc=0 wall=533s | σ = 8.503414 ± 0.747117 pb
pilot-vbf4-r0 card=VBFto2L_Bin-MLL-4to50 seed=1 neval=10000 niter=4 rc=0 wall=461s | σ = 104.699526 ± 6.997600 pb
dy-r1-s1 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=1 neval=10000 niter=3 rc=0 wall=693s | σ = 11.969900 ± 0.932908 pb
dy-r1-s2 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=2 neval=10000 niter=3 rc=0 wall=735s | σ = 14.936697 ± 2.374074 pb
dy-r1-s3 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=3 neval=10000 niter=3 rc=0 wall=935s | σ = 11.797280 ± 0.574758 pb
dy-r1-s4 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=4 neval=10000 niter=3 rc=0 wall=922s | σ = 13.995478 ± 1.759216 pb
dy-r1-s5 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=5 neval=10000 niter=3 rc=0 wall=572s | σ = 12.126630 ± 0.689593 pb
dy-r1-s6 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=6 neval=10000 niter=3 rc=0 wall=440s | σ = 11.466032 ± 0.473115 pb
dy-r1-s7 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=7 neval=10000 niter=3 rc=0 wall=412s | σ = 11.352622 ± 0.403627 pb
dy-r1-s8 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=8 neval=10000 niter=3 rc=0 wall=441s | σ = 12.935757 ± 1.052571 pb
dy-r1-s9 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=9 neval=10000 niter=3 rc=0 wall=420s | σ = 10.679520 ± 0.359401 pb
dy-r1-s10 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=10 neval=10000 niter=3 rc=0 wall=421s | σ = 11.551483 ± 0.430045 pb
dy-r1-s11 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=11 neval=10000 niter=3 rc=0 wall=414s | σ = 12.984483 ± 0.942344 pb
dy-r1-s12 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=12 neval=10000 niter=3 rc=0 wall=402s | σ = 12.490177 ± 0.932102 pb
dy-r1-s13 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=13 neval=10000 niter=3 rc=0 wall=424s | σ = 11.561231 ± 0.567131 pb
dy-r1-s14 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=14 neval=10000 niter=3 rc=0 wall=415s | σ = 12.445249 ± 1.365465 pb
dy-r1-s15 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=15 neval=10000 niter=3 rc=0 wall=394s | σ = 13.579473 ± 1.818190 pb
dy-r1-s16 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=16 neval=10000 niter=3 rc=0 wall=399s | σ = 18.101254 ± 5.997293 pb
dy-r1-s17 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=17 neval=10000 niter=3 rc=0 wall=387s | σ = 12.734328 ± 0.634621 pb
dy-r1-s18 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=18 neval=10000 niter=3 rc=0 wall=389s | σ = 10.929752 ± 0.524480 pb
dy-r1-s19 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=19 neval=10000 niter=3 rc=0 wall=386s | σ = 14.742465 ± 2.518524 pb
dy-r1-s20 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=20 neval=10000 niter=3 rc=0 wall=402s | σ = 12.321067 ± 0.778491 pb
vbf50-r1-s1 card=VBFto2L_Bin-MLL-50 seed=1 neval=10000 niter=3 rc=0 wall=388s | σ = 8.245534 ± 1.033098 pb
vbf50-r1-s2 card=VBFto2L_Bin-MLL-50 seed=2 neval=10000 niter=3 rc=0 wall=386s | σ = 6.526713 ± 0.505958 pb
vbf50-r1-s3 card=VBFto2L_Bin-MLL-50 seed=3 neval=10000 niter=3 rc=0 wall=393s | σ = 9.061639 ± 1.329933 pb
vbf50-r1-s4 card=VBFto2L_Bin-MLL-50 seed=4 neval=10000 niter=3 rc=0 wall=384s | σ = 7.906333 ± 1.679582 pb
vbf50-r1-s5 card=VBFto2L_Bin-MLL-50 seed=5 neval=10000 niter=3 rc=0 wall=385s | σ = 7.632453 ± 0.940979 pb
vbf4-r1-s1 card=VBFto2L_Bin-MLL-4to50 seed=1 neval=10000 niter=3 rc=0 wall=310s | σ = 93.066868 ± 8.924896 pb
vbf4-r1-s2 card=VBFto2L_Bin-MLL-4to50 seed=2 neval=10000 niter=3 rc=0 wall=306s | σ = 95.122941 ± 6.375758 pb
vbf4-r1-s3 card=VBFto2L_Bin-MLL-4to50 seed=3 neval=10000 niter=3 rc=0 wall=334s | σ = 158.780579 ± 18.758837 pb
vbf4-r1-s4 card=VBFto2L_Bin-MLL-4to50 seed=4 neval=10000 niter=3 rc=0 wall=319s | σ = 110.927985 ± 12.393580 pb
vbf4-r1-s5 card=VBFto2L_Bin-MLL-4to50 seed=5 neval=10000 niter=3 rc=0 wall=294s | σ = 200.281889 ± 100.525222 pb
dy-r4-s1 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=1 neval=10000 niter=12 rc=0 wall=1262s | σ = 12.577028 ± 0.478981 pb
dy-r4-s2 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=2 neval=10000 niter=12 rc=0 wall=1443s | σ = 14.055480 ± 1.024113 pb
dy-r4-s17 SKIPPED: not run, the campaign is capped near 10 h
dy-r4-s18 SKIPPED: not run, the campaign is capped near 10 h
dy-r4-s19 SKIPPED: not run, the campaign is capped near 10 h
dy-r4-s20 SKIPPED: not run, the campaign is capped near 10 h
dy-r4-s3 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=3 neval=10000 niter=12 rc=0 wall=1703s | σ = 11.929993 ± 0.232501 pb
dy-r4-s4 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=4 neval=10000 niter=12 rc=0 wall=2367s | σ = 13.242881 ± 0.536947 pb
dy-r4-s13 SKIPPED: not run, the campaign is capped near 10 h (host shared with a sibling's banked suite from about 19:50)
dy-r4-s14 SKIPPED: not run, the campaign is capped near 10 h (host shared with a sibling's banked suite from about 19:50)
dy-r4-s15 SKIPPED: not run, the campaign is capped near 10 h (host shared with a sibling's banked suite from about 19:50)
dy-r4-s16 SKIPPED: not run, the campaign is capped near 10 h (host shared with a sibling's banked suite from about 19:50)
dy-r4-s5 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=5 neval=10000 niter=12 rc=0 wall=2294s | σ = 12.089536 ± 0.231996 pb
dy-r4-s6 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=6 neval=10000 niter=12 rc=0 wall=2514s | σ = 12.625261 ± 0.436599 pb
dy-r4-s7 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=7 neval=10000 niter=12 rc=0 wall=2315s | σ = 12.575855 ± 0.426379 pb
dy-r4-s8 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=8 neval=10000 niter=12 rc=0 wall=1815s | σ = 13.129231 ± 0.628075 pb
dy-r4-s9 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=9 neval=10000 niter=12 rc=0 wall=1135s | σ = 13.234986 ± 0.743730 pb
dy-r4-s10 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=10 neval=10000 niter=12 rc=0 wall=1106s | σ = 12.347596 ± 0.273219 pb
dy-r4-s11 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=11 neval=10000 niter=12 rc=0 wall=1134s | σ = 13.508101 ± 0.926678 pb
dy-r4-s12 card=DYto2L-2Jets-EWK_Bin-MLL-50 seed=12 neval=10000 niter=12 rc=0 wall=1136s | σ = 13.424739 ± 0.611322 pb
```
