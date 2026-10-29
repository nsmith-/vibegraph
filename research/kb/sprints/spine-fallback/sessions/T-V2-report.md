---
type: Session Report
title: "T-V2 report: budget study and CMS VBF/EWK ladder (in progress)"
description: "Campaign plan with measured per-run wall times, written before the long runs were launched."
status: draft
tags: [phase-space, spine, vbf, cms, budget-ladder, sprint-e, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-29}
---
The session brief is [T-V2](T-V2.md). This checkpoint holds the campaign plan
only; the results follow when the campaign ends.

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
```
