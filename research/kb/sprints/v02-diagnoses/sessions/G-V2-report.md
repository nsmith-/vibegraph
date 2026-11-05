---
type: Session Report
title: "G-V2 report: the 3 % slice deficit is gone on main, and a point-level tail index is the indicator that sees it"
description: "On main the both-W |cos θ| > 0.99 slice integrates correctly at the default budget (30 seeds, -0.07 % and +0.07 %, chi2/dof about 1, against a 1e6 reference and a frozen-grid estimator). The deficit reproduces on the V2-era tree (-2.4 %, 10 seeds); its cause was a heavy weight tail from timelike-only channels that the unbounded grid adaptation starved, with the convergence stop stopping on the iterations that missed the tail. The bin-width bound and the peripheral spine removed it. An iteration-scatter indicator is blind to exactly the low runs; a per-channel Hill tail index below 2 flags 16 of 17 at-risk runs and 0 of 330 correct ones. No production code: a runtime-warning proposal (decision G5)."
status: draft
tags: [vegas, convergence, cuts, multichannel, heavy-tail, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-04}]
---
Diagnosis and a proposal. No source change is committed. Every number below is
from a run made in this session. The scratch is under the session scratchpad,
`G-V2/`: the driver `run.sh`, the result lines `G-V2-results.txt`, the
statistics `stats.py` (unweighted mean, error `max(√Σσᵢ²/n, sd/√n)`, χ²/dof
about the own mean with quoted errors), the per-channel parsers `chan.py`,
`ind.py`, `dumpind.py` and `tailind.py`, and the two probe patches
(`probe-full.patch`, `patch_tail.py`).

**Setup.** Process `u u~ > w+ b w- b~ / t t~`, the banked card
(`onshell_uu_run_card.dat` with `lpp1 = lpp2 = 0`, `ebeam = 250`,
`use_syst = False`, its dynamical scale), `vibegraph integrate`, `-j 2`,
`release-debug`. FW is both W's at |cos θ| > 0.99 in the lab. It is an
uncommitted probe in `Cuts::pass` (`VG_PROBE_FWD=fw`, legs 2 and 4). Further
uncommitted probes:

- `VG_PROBE_FREEZE` skips `refine_grid`, so every grid stays uniform. This is
  the non-adaptive estimator: the α-survey weights are fixed before the run,
  the channel maps are the only importance sampling, and every iteration is
  i.i.d.
- `VG_PROBE_DUMP` writes each `integrate_channels` call's per-channel,
  per-iteration `(integral, variance, neval, accepted)`, plus two point-level
  statistics: `q`, the largest point's share of Σw², and the Hill tail index of
  the top 5 % of accepted |w| (10 ≤ k ≤ 200).
- `report_channels` also prints the accepted count.

All probes are inert when their variables are unset. A fixed seed reproduces
σ to the printed digit with and without them (seed 20261101: 0.001497 ±
0.000008 pb on all three builds), and the banked gates' indicator values agree
between two differently probed builds.

Three trees were built:

| label | commit | what it has |
|---|---|---|
| main | 75131e1 (code of main 86b8e5c) | everything below |
| prespine | e487ee8, the parent of #81 | the bin-width bound K = 6 (#48), the soft-all and τ defaults (#51), the plain-mean combination (#75), the α floor (#84); no peripheral spine without a pT cut, so the phase-space line reads "0 peripheral, 13 all-timelike" |
| v2era | 2b198c8 (#28, the merge of the V2 sprint) | none of #48, #51, #75, #81, #84; "0 peripheral, 13 all-timelike", split-angle isotropic |

The FW slice's reference is main at `--neval 1e6 --target-rel 3e-3`,
0.0014912 ± 0.0000009 pb over 5 seeds. The integrand (matrix element, cuts,
scale) is the same in all three trees. The trees' own 1e6 references agree
with it: v2era 0.0014873 ± 0.0000032 (3 seeds), prespine 0.0014880 ±
0.0000023 (3 seeds). So do the non-adaptive estimates.

## Verdict

1. **The brief's premise is false on main.** At the default budget
   (`--neval 120000`, `--target-rel 1e-2`) the FW slice reads **−0.07 %
   (−0.6 σ) over 20 seeds, χ²/dof 1.12**, and +0.07 % over 10 more. Every run
   stops at the 6-iteration minimum. A fixed 12-iteration budget reads −0.02 %,
   and the frozen-grid (non-adaptive) estimator reads +0.07 %. There is no
   deficit to detect on this slice.
2. **The deficit reproduces on the V2-era tree.** It reads −2.43 % (−6.3 σ,
   10 seeds, χ²/dof 2.06), and −2.18 % on 10 more. V2 measured −2.8 %. The
   1e6 reference and the frozen-grid estimate of the same tree are correct, so
   this is the sampler, not the integrand.
3. **Mechanism.** The cause is a heavy-tailed weight distribution in the
   σ-carrying channels, not the floor and not the allocation. No channel's map
   follows the forward W emission: before #81 all 13 channels were timelike.
   The unbounded VEGAS grid adaptation then starves the region further.
   - The bin-width bound is the main fix: prespine reads −0.34 % with it and
     −2.11 % with `--grid-max-bin-width none` (4 of 5 runs give up at 500
     iterations).
   - The soft-all map is a second, smaller one: prespine reads −0.93 % with
     `--map-split-angle isotropic`.
   - The peripheral spine (#81) removes the tail outright. Main's runs stop at
     6 iterations with scatter at χ²/dof 1.
   - The convergence stop turns the tail into a one-sided bias. On prespine
     the stopping iteration and σ correlate at **Pearson 0.80**: runs that
     stopped at 23 and 42 iterations read −3.4 % and −2.2 %, and runs past 100
     iterations read +0.2 % to +1.0 %.
4. **Detection.** The iteration-scatter indicator I measured first is
   structurally blind to the failing runs. It is the largest per-point
   variance of any σ-carrying channel over its median, across the kept
   iterations. It fires on runs that drew a tail point, which the existing
   stop scale already keeps running, and stays quiet on the runs that stopped
   early on a low value. A **point-level Hill tail index below 2** (infinite
   variance) sees the regime in every run:
   - **16 of 17** runs of the two at-risk trees are flagged, including every
     early-stopped low run.
   - **0 of 330** correct integrations are flagged: 10 FW runs on main, 245
     `validate_sigma` integrations and 75 `validate_hadronic` integrations.
   - The margin is thin on main's FW slice. Its lowest index is 2.08, against
     2.07 for the one positive missed.

   It needs a new per-point statistic in the hot reduction, and it has one
   positive process. I therefore stop before production code and propose it as
   the decision-G5 runtime warning (§5).

## 1. Reproduction (brief step 1)

FW slice, banked card. "vs ref" is against main's 1e6 reference, 0.0014912 ±
0.0000009 pb.

| tree | arm | seeds | σ (pb) | error | seed sd | χ²/dof | vs ref | pull | iterations (min–median–max) | gave up |
|---|---|---|---|---|---|---|---|---|---|---|
| main | reference, `--neval 1e6 --target-rel 3e-3` | 5 | 0.0014912 | 0.9e-6 | 1.9e-6 | 0.93 | — | — | 6–6–6 | 0 |
| main | default, target 1e-2 | 20 | 0.0014902 | 1.3e-6 | 5.7e-6 | 1.12 | −0.07 % | −0.6 | 6–6–6 | 0 |
| main | default, target 1e-2 (second sweep, tail probe) | 10 | 0.0014922 | 1.7e-6 | 5.4e-6 | 1.02 | +0.07 % | +0.5 | 6–6–6 | 0 |
| main | `--fixed-budget --niter 12` | 10 | 0.0014909 | 1.2e-6 | 3.8e-6 | 1.02 | −0.02 % | −0.2 | 12 | 0 |
| main | **frozen grids**, fixed 1e6 × 8, by-α | 5 | 0.0014922 | 2.3e-6 | 5.1e-6 | 1.03 | +0.07 % | +0.4 | 8 | 0 |
| v2era | reference, 1e6, target 3e-3 | 3 | 0.0014873 | 3.2e-6 | 5.5e-6 | 2.59 | −0.26 % | −1.2 | 7–7–9 | 0 |
| v2era | **default, target 1e-2** | 10 | 0.0014549 | 5.7e-6 | 1.8e-5 | 2.06 | **−2.43 %** | **−6.3** | 7–13–246 | 0 |
| v2era | default, second sweep | 10 | 0.0014587 | 1.1e-5 | 3.3e-5 | 4.95 | −2.18 % | −3.1 | 8–14–500 | 2 |
| v2era | frozen grids, fixed 1e6 × 8, by-α | 5 | 0.0015282 | 2.3e-5 | 4.9e-5 | 0.88 | +2.48 % | +1.6 | 8 | 0 |
| prespine | reference, 1e6, target 3e-3 | 3 | 0.0014880 | 2.3e-6 | 1.7e-6 | 0.19 | −0.21 % | −1.3 | 31–33–53 | 0 |
| prespine | default, target 1e-2 | 10 | 0.0014862 | 7.1e-6 | 2.3e-5 | 2.49 | −0.34 % | −0.7 | 23–77–182 | 0 |
| prespine | default, second sweep | 7 | 0.0014804 | 7.1e-6 | 1.9e-5 | 1.95 | −0.72 % | −1.5 | 18–74–432 | 0 |
| prespine | frozen grids, fixed 1e6 × 8, by-α | 5 | 0.0014788 | 2.6e-5 | 5.9e-5 | 1.33 | −0.83 % | −0.5 | 8 | 0 |

Seeds: default 20261101–20261110 (main also to 20261120), second sweep
20261501–20261510, fixed 20261301–, frozen 20261401–, reference 20261201–.

The frozen-grid estimators of the two old trees are only good to 1.6–1.8 %
(their uniform grids over timelike maps are inefficient). They are unbiased by
construction, and their scatter agrees with their quoted errors (χ²/dof 0.88
and 1.33), so they bound a sampler-side bias rather than pin it. Main's
frozen estimator is good to 0.15 % and agrees with every adaptive arm.

## 2. Mechanism (brief step 2)

| hypothesis | test that comes out differently if false | result |
|---|---|---|
| **512-point floor** starves the slice | Where the σ sits in the per-channel table: points, accepted points, share | **falsified.** The floor-bound channels (0–5, 9, 11) carry < 0.2 % of the slice together. The deficit is in channels 6, 7, 8, 10 and 12 (each 11–33 % of the slice by the reference), allocated 18 000–35 000 points an iteration (35–70× the floor), with 8 700–18 400 accepted. The acceptance cap ("13 of 13 channels accept less than one point in 4", printed after iteration 1) is a cold-start artifact: the later acceptances are 0.35–0.76. |
| **Neyman allocation** under-estimates a channel's variance early | v2era with `--allocate by-alpha` | **mostly falsified.** −1.94 % (−5.2 σ, 10 seeds) against −2.43 %. At most a quarter of the deficit. The count-weighted mean that #75 removed rode on it. |
| **Iteration weighting / the stop** (1/σ² combination, or stopping on confidently low iterations) | A fixed budget with the same per-iteration size, which no stopping rule can bias | **partly confirmed.** The combination has been the plain mean since #75, and v2era's fixed 13-iteration budget reads −1.18 % (−1.9 σ, χ²/dof 2.8) against the target's −2.43 %. On prespine the fixed 60-iteration budget reads −0.48 %, the target run −0.34 %. Stopping time correlates with σ at Pearson 0.80 (Spearman 0.70) over 10 prespine seeds, and at −0.12 on v2era, where nearly every run stops at 7–13 iterations. |
| **Grid adaptation** collapses bins on the region | Remove the bin-width bound from the tree that has it (prespine `--grid-max-bin-width none`) | **confirmed, the main cause.** −2.11 % (−2.4 σ, 5 seeds); 4 of 5 runs give up at the 500-iteration cap and the fifth stops at 267. With the bound the same tree reads −0.34 %. |
| the map (isotropic against soft-all) | prespine `--map-split-angle isotropic` | **contributes.** −0.93 % (−2.4 σ, 5 seeds, χ²/dof 0.65). The low χ²/dof is a confidently low sweep, not a clean one. |
| no channel follows the forward W (the spine) | main against prespine | **confirmed.** Main has 7 peripheral channels, all runs stop at 6 iterations, and the per-point variance of the dominant channel is stable to a factor 1.2–2.5 across iterations. On prespine it varies by 10²–10⁴. |

**The per-channel table** (v2era, kept iterations averaged per seed, then
over seeds; default 10 seeds, reference 3 seeds). The α survey is re-run per
seed, so σ moves between channels. The single channels are noisy, and only
the σ-carrying set is read.

| channel | default (pb) | reference (pb) | default/reference − 1 | share of the slice (reference) | points / iteration, default | accepted / iteration, default | points / iteration, reference |
|---|---|---|---|---|---|---|---|
| 6 | 1.883e-4 ± 1.3e-5 | 1.702e-4 ± 2.8e-5 | +10.6 % | 0.114 | 18 451 | 9 076 | 110 191 |
| 7 | 1.928e-4 ± 1.3e-5 | 1.703e-4 ± 2.7e-5 | +13.2 % | 0.115 | 17 970 | 8 702 | 139 155 |
| 8 | 4.442e-4 ± 2.0e-5 | 4.879e-4 ± 4.2e-5 | −8.9 % | 0.328 | 28 117 | 14 774 | 292 568 |
| 10 | 4.399e-4 ± 2.1e-5 | 4.870e-4 ± 4.2e-5 | −9.7 % | 0.327 | 35 501 | 18 392 | 339 698 |
| 12 | 1.897e-4 ± 1.2e-5 | 1.700e-4 ± 2.8e-5 | +11.6 % | 0.114 | 18 953 | 8 899 | 113 335 |
| 0–5, 9, 11 | < 1.6e-6 together | < 2.5e-6 together | — | < 0.002 | 1 640–1 850 | 490–660 | ~2 048 |
| sum | 0.0014576 | 0.0014878 | −2.0 % | | | | |

**Reading.** At the default budget the slice's σ sits in five channels, none
of which maps it. Their per-iteration estimates carry spikes of 4–13× their
own mean at random iterations (seed 20261103: channel 12 at 13.2×, channel 8
at 5.5×). The per-point variance of the dominant channel swings by up to 10⁵
between iterations. That is the signature of a weight distribution whose tail
index is below 2: the variance is not finite at this sampling, the quoted
error describes the bulk, and most seeds draw too few tail points. Every
estimate is still unbiased, and the deficit is the gap between the mean and
the typical value of a sum over such a distribution.

The target stop amplifies it in one direction. A run that has not drawn a
tail point shows calm iterations, meets its target and stops low. A run that
has drawn one is held by `stop_scale` and keeps sampling. The bin-width bound
caps each coordinate's Jacobian at 6 and so bounds how far the grid can starve
the tail, and the peripheral spine puts a map on it. Together they took the
slice from −2.4 % to −0.07 %.

## 3. Detection (brief step 3)

Two indicators, measured on the same runs. "Correct runs" are the main FW
sweep and every σ integration of the two banked gates.

| indicator | definition | at-risk runs flagged (v2era + prespine, second sweep) | correct runs flagged | worst correct value | what it cannot see |
|---|---|---|---|---|---|
| R > 100 (iteration scatter) | max/median over kept iterations of `variance · neval`, over channels with ≥ 2 % of σ; existing per-iteration data, no new statistic | 11 / 17 | 0 / 355 (main FW 35, banked 320) | 72.7 (main FW seed 20261504), 21.8 (banked, `ud_to_epemud_qcd0`) | **The runs that matter.** The six it misses include every early-stopped low run: prespine 20261503 (−2.4 %, R = 79.6) and 20261505 (−2.4 %, R = 22.0), v2era 20261504 (−3.3 %), 20261506 (−3.2 %) and 20261510 (−2.0 %). On the first sweep, R ≥ 100 picked out the runs the stop scale already kept going. |
| **Hill index < 2** (point level) | per channel with ≥ 2 % of σ, the median over kept iterations of the Hill estimator of the top 5 % of accepted \|w\| (10 ≤ k ≤ 200); flag if any is below 2 | **16 / 17** | **0 / 330** (main FW 10, banked 320) | 2.08 (main FW), 2.24 (banked, `ee_to_mumu_tata_qcd0` and `bb_fixed_scale`) | A tail heavier than the bulk but above index 2. The one miss, v2era 20261501, reads 2.07 and is −1.8 %. It also cannot tell whether a flagged run is wrong: v2era 20261502 and 20261508 ran 500 iterations and are correct, but flagged. It flags the regime, not the bias. |

Hill index per run (second sweep):

- main: 2.19, 2.36, 2.34, 2.18, 2.08, 2.22, 2.10, 2.23, 2.33, 2.19.
- v2era: 1.62 (−7.1 %), 1.33 (−3.3 %), 1.74 (−1.4 %), 1.59 (−3.2 %), 1.85
  (−1.0 %), 1.83 (+0.8 %), 1.94 (−3.2 %), 1.43 (−2.0 %), 2.07 (−1.8 %), 1.85
  (+0.3 %).
- prespine: 1.67 (0.0 %), 1.60 (−2.4 %), 1.61 (+0.4 %), 1.64 (−1.2 %), 1.69
  (+0.3 %), 1.72 (+0.3 %), 1.55 (−2.4 %).

Banked rows: 245 `validate_sigma` integrations, minimum 2.24, 5th percentile
2.58, median 7.8; 75 `validate_hadronic` integrations, minimum 2.25.

**Why this is not reliable enough to gate the stop:**

- The positive class is one process in two historical trees. On main nothing
  fails, so the indicator cannot be shown to fire on main, which the brief
  asked for.
- Main's own FW slice sits at 2.08–2.36 while being correct. A stop that held
  runs below some threshold near 2 would sit next to correct, converged rows.
  The budget's own documentation records a Pareto index "near 2" for the
  σ-carrying channels of production rows.
- The estimator's k is a choice made here (top 5 %, at most 200) and not
  calibrated.
- It needs a sort, or a top-k selection, of every channel's accepted weights
  every iteration in the hot reduction of `adapt_blocks_iteration`.

## 4. Gate results

The committed change is this Markdown file. The probe trees were used to run
the banked gates with the dumps on. The indicator values of the two
differently probed builds agree to the printed digit (max R 21.8 on both), and
the fixed-seed σ agrees across all builds.

- `cargo fmt --all --check` (committed tree): `fmt exit 0`.
- `cargo clippy --jobs 2 --workspace --all-targets -- -D warnings`:
  `Finished`, exit 0.
- `cargo clippy --jobs 2 --workspace --all-targets --features vibegraph/extended-validation -- -D warnings`:
  `Finished`, exit 0.
- `cargo test --jobs 2 --workspace -- --test-threads 2` (committed tree,
  `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 RAYON_NUM_THREADS=2`):
  50 suites, 1804 passed, 0 failed, 25 ignored; `test exit 0`.
- `cargo test --jobs 2 -p vibegraph-lib --profile release-debug --features extended-validation --test validate_sigma -- --nocapture --test-threads 1`
  (probe tree, `VG_PROBE_DUMP` set, `RAYON_NUM_THREADS=2`):
  `test result: ok. 8 passed; 0 failed; 33 ignored; 0 measured; 0 filtered out; finished in 1292.98s`.
  The second probe build gave the same counts, `finished in 1021.12s`.
- Same for `--test validate_hadronic`:
  `test result: ok. 20 passed; 0 failed; 26 ignored; 0 measured; 0 filtered out; finished in 835.09s`
  (second build `762.94s`).
- `python3 scripts/kb.py lint`: `1053 concepts, 340 backlog items; 0 errors,
  4 warnings` before this file. After it, `python3 scripts/kb.py index`
  printed `wrote 1 index files` and lint printed `1054 concepts, 340 backlog
  items; 0 errors, 4 warnings`. The warnings are on backlog items this
  session did not touch.
- Sampling order: unchanged. No source change is committed.

## 5. Proposal for decision G5: a runtime warning

- **Indicator.** For every channel carrying at least 2 % of the combined σ,
  the median over the kept iterations of the Hill tail index of the top 5 %
  (10 ≤ k ≤ 200) of that iteration's accepted |w|. It needs one extra
  per-channel statistic per iteration: a top-k selection over the accepted
  weights in the serial reduction of `adapt_blocks_iteration`, which already
  visits every point in order.
- **Threshold.** Warn when any such channel's index is below **2.0**: the
  variance of its weights is not finite at this sampling, so the quoted error
  and the convergence stop do not bound the error.
  - Measured hit rate: 16 of 17 runs of the at-risk configurations, including
    all 7 runs that read 2 % or more low.
  - Measured false-positive rate: 0 of 330 correct integrations.
  - Margins: main FW minimum 2.08, banked minimum 2.24, at-risk median 1.65.
- **Wording, for example:** "channel 8 (33 % of σ): per-point weights have
  tail index 1.6 < 2; the quoted ±0.9 % does not bound the error, and runs in
  this regime have read up to 7 % low. Raise --neval, or check against
  --neval 1e6."
- **What it does not do.** It does not change the stop, the budget or any
  number, and it cannot say whether a flagged run is wrong, only that its
  error bar is not evidence.
- **What the user decides:**
  - whether a warning-only change lands;
  - whether the threshold sits at 2.0 with main's FW slice at 2.08;
  - whether the index should instead raise the budget, which on these
    configurations means hundreds of iterations, 4 of 5 at the cap.

## Brief corrections

- **"Reproduce the FW deficit on main"**: it does not reproduce. Main is
  correct at the default budget (30 seeds, two independent sweeps, χ²/dof
  about 1) against both the 1e6 reference and the non-adaptive estimator. The
  deficit was fixed between V2 and main by #48 (bin-width bound, most of it),
  #51 (soft-all) and #81 (peripheral spine). The item's `closes_when` is met
  by its "or the default budget covers it" branch, shown against the
  non-adaptive estimate. Its detection branch is not met, and this report says
  why.
- **"The per-channel floor or allocation"** is not where the slice is lost.
  The floor-bound channels carry < 0.2 % of the slice, and the by-α
  allocation is nearly as low as Neyman.
- **"Iteration weighting (1/σ² combination rewarding confidently-low
  iterations)"**: there is no 1/σ² combination in any tree measured here. The
  combination was already unweighted, or count-weighted under Neyman before
  #75. The live version of this hypothesis is the stopping rule, and it holds
  in part.
- **The V2 reference** (0.0014867 ± 0.000002 pb, banked FW, three seeds at
  1e6) is 0.30 % below this session's 0.0014912 ± 0.0000009. The v2era tree's
  own 1e6 reference here is 0.0014873, matching V2's. That reference was
  itself mildly affected by the regime.

## Found

- **The convergence stop is biased low in a heavy-tail regime, and its scale
  factor cannot see it.** On prespine the stopping iteration correlates with
  σ at Pearson 0.80, and the early-stopped runs are the low ones (−2.2 % to
  −3.4 %). `stop_scale` is formed from iteration scatter, which is calm in
  exactly the runs that never drew a tail point. It is latent on main's FW
  slice, which stops at the minimum with index 2.1–2.4, and live on any
  process whose dominant channels have no map on a peaked region. To close:
  the G5 decision above, or a stopping rule that carries a point-level tail
  test.
- **The printed VEGAS χ²/dof overflows on a sliced fixed-beam run on main
  too.** Main's FW runs print χ²/dof as integers of 10⁸⁰ and more (seed
  20261101: `χ²/dof = 46878601…`). It comes from floor-bound channels whose
  iterations measure zero or near-zero variance (`combine_kept` floors at
  `f64::MIN_POSITIVE`). This is V2's Found 4, still open, and in main's
  default output. To close: compute the run-level χ²/dof over channels and
  iterations with non-zero variance, or report the σ-share-weighted
  statistic.
- **"13 of 13 channels accept less than one point in 4" after iteration 1 is
  a cold-start false alarm on this slice.** The later acceptances are
  0.35–0.76, yet the warning stays the run's only coverage message. To close:
  re-evaluate it after the warm-up, or word it as a first-iteration
  measurement.
- **The frozen-grid probe (`VG_PROBE_FREEZE`) is a cheap, exact non-adaptive
  oracle for any σ row.** It is 13 s for 8 × 1e6 points on this slice. Main
  has no such mode: `--niter` with warm-up still adapts. To close: a
  test-only or hidden flag that skips `refine_grid`, so a sampler-side bias
  can be bounded without a second generator.
