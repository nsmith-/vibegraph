---
type: Measurement
title: Cost of the configuration-share accounting
description: "Where integrate's per-configuration AMP2 share accounting spent its time (the AMP2 read-out loop, warm-up iterations, rejected points, two slash kernels tipped out of line) and what removing it bought: p p > e+ e- j +11.6% → +5.7% instructions, e+ e- > u u~ g +3.6% → +0.3%, artifacts byte-identical."
status: draft
tags: [performance, integrate, multichannel, amp2, configuration, profiling, callgrind]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
measured:
  - commit: 0913689
    host: "cloud container, 4 vCPUs, Intel Xeon @ 2.30 GHz (family 6 model 173), rustc 1.97.0, shared with another session"
    command: "valgrind --tool=callgrind (one thread, RAYON_NUM_THREADS=1) on release builds of 250e885 (no shares), 582df79 (shares) and this commit: vibegraph integrate <card> --run-card validation/madgraph/config_share_<row>_run_card.dat --fixed-budget --neval 100000 --niter {3,8} --seed 1 -j 1"
  - commit: 0913689
    host: "cloud container, 4 vCPUs, Intel Xeon @ 2.30 GHz (family 6 model 173), rustc 1.97.0, shared with another session"
    command: "release builds of 250e885, 582df79 and this commit, alternated five times (order reversed on odd rounds): vibegraph integrate <card> --run-card validation/madgraph/config_share_<row>_run_card.dat --fixed-budget --neval 1000000 --niter 8 --seed 1, user CPU time from wait4"
---
# Cost of the configuration-share accounting

What the per-configuration `AMP2` shares of
[the integrate artifact](../pipeline/integrate-artifact.md#configuration-shares)
cost an integration, where that cost sat, and what was done about it. The
cards are the [configuration-share oracle](../validation/configuration-share-oracle.md)'s
`pp_eej` (`p p > e+ e- j`, 6 flavour groups, 30 shares, under a quarter of
points pass the cuts) and `ee_uuxg` (`e+ e- > u u~ g`, 1 subprocess, 5 shares,
about two thirds pass). Three builds: `250e885` (no shares), `582df79` (shares
as first built) and the optimised tree.

## Instruction counts

Callgrind counts are deterministic, which the shared host's wall and CPU
times are not (load average 4–12 during this work). One thread,
`--neval 100000 --seed 1`; `--niter 8` keeps 6 of 8 iterations, as the
timing runs do, `--niter 3` keeps 1.

| card | niter | no shares | shares | optimised | shares Δ | optimised Δ |
|---|--:|--:|--:|--:|--:|--:|
| `pp_eej` | 8 | 43.913 G | 48.998 G | 46.433 G | +11.58% | +5.74% |
| `pp_eej` | 3 | 13.430 G | 14.670 G | 13.799 G | +9.23% | +2.75% |
| `ee_uuxg` | 8 | 64.917 G | 67.283 G | 65.134 G | +3.64% | +0.33% |
| `ee_uuxg` | 3 | 31.078 G | 31.993 G | 30.981 G | +2.94% | −0.31% |

Where the shares' cost went, `pp_eej` at `--niter 8` (function self cost,
shares build minus no-shares build):

| item | Δ instructions | share of total |
|---|--:|--:|
| `BoundAmplitude::filled_amp2` (`AMP2` read off each group's evaluation) | +3.21 G | +7.31% |
| `off_shell_fin_bare` / `off_shell_fout_bare` called out of line, net of `fill_arenas` | +1.07 G | +2.45% |
| tile accumulation in `adapt_blocks_iteration` | +0.37 G | +0.84% |
| `ProtonIntegrand::split_term` | +0.28 G | +0.64% |
| `enhancement_weights`, the rest | +0.15 G | +0.3% |

- **`filled_amp2`** cost ~1870 instructions a call against ~16 000 for the
  `eval_m2` it follows: per helicity row it sliced the row per configuration,
  bounds-checking two slices per configuration, and for one-amplitude
  configurations that overhead outweighed the arithmetic.
- **The two slash kernels** were the only `*_bare` kernels without the soft
  `#[inline]` the
  [program layout](evaluator-program-layout.md) gives the rest. The
  no-shares build inlined them into `fill_arenas` anyway; adding the shares
  code tipped LLVM's fat-LTO decision and both became calls (1.9 M calls each
  at `--niter 8`), a cost on every |M|² evaluation, including those of runs
  and points that compute no share. This is the trap the layout concept
  records: diff the out-of-line call census between builds.
- **Warm-up iterations** accumulated shares that `ChannelHistory::record`
  then dropped: 2 of 8 iterations' worth of share work bought nothing.
- **Rejected points** (three quarters of `pp_eej`'s) zeroed and summed all 30
  parts, twice (`ShareSplit` and the tile), to add zeros.

## What changed

All four changes keep every bit: the artifacts of the shares build and of the
optimised build are byte-identical at `--niter 3` and `--niter 8` on both
cards and at the timing budget (`cmp` of `grid.bin.zst`), and every σ_c is
the same.

1. `filled_amp2` walks each helicity row once, closing a configuration's
   coherent sum at its last amplitude (`AmplitudeEvaluator::config_ends`).
   The additions are the per-configuration loop's, in its order, which
   `amp2_along_the_row_matches_the_sum_per_configuration` pins bit for bit
   over the validated processes (Bhabha's merged t-channel γ/Z included).
   Per call: ~1870 → ~1250 instructions. The scale draw's own `AMP2`
   (`eval_amp2`) goes through the same loop, which is why `ee_uuxg` ends
   below its no-shares count at `--niter 3`.
2. `#[inline]` on `off_shell_fin_bare` and `off_shell_fout_bare`: the
   out-of-line census returns to zero and `fill_arenas` to its no-shares
   count.
3. Warm-up iterations accumulate no shares (`draw_iteration`'s `shares`).
4. A point of value zero touches no part: `ShareSplit` keeps its parts zero
   between points and clears only what a point wrote, and the tile
   accumulation skips a zero-valued point. A tile sum starts at `+0.0`, so it
   never holds `−0.0` and adding `+0.0` is the identity; the skip is exact.

What remains on `pp_eej` (+5.74%) is `filled_amp2` (+4.44%: a complex
multiply-add per amplitude per helicity row, with a pool lookup and two
bounds checks per amplitude), `split_term` (+0.58%), the tiles (+0.30%) and
`ShareSplit::finish` (+0.17%).

## Timing

Release builds, `--fixed-budget --neval 1000000 --niter 8 --seed 1`, the
default four threads, five rounds alternating the three builds (order
reversed on odd rounds), load average 3.6–4.0 (one other session building).
User CPU time from `wait4`, median and [min–max]; the paired column is each
round's ratio to the no-shares run of the same round, median of five.

| card | build | user (s) | wall (s) | vs no shares, paired |
|---|---|--:|--:|--:|
| `pp_eej` | no shares | 27.04 [26.76–28.00] | 7.37 [7.16–7.60] | |
| | shares | 29.69 [29.49–30.13] | 8.08 [7.91–8.18] | +9.6% (+7.3 … +11.0) |
| | optimised | 28.63 [28.06–29.05] | 7.60 [7.52–7.88] | +4.7% (+3.7 … +7.0) |
| `ee_uuxg` | no shares | 37.91 [37.46–38.48] | 10.56 [10.37–10.62] | |
| | shares | 39.95 [38.85–40.45] | 11.00 [10.84–11.09] | +5.5% (+1.0 … +6.7) |
| | optimised | 39.31 [39.10–39.60] | 10.87 [10.78–10.97] | +3.7% (+2.2 … +4.5) |

Every round's optimised artifact is byte-identical to the shares build's
(`cmp`, all ten), and all print `σ = 257.67 ± 0.18 pb` and
`σ = 346.04 ± 0.13 fb`. On `pp_eej` time follows the instruction counts. On
`ee_uuxg` it does not: +3.7% measured against +0.3% counted, and the
shares-to-optimised gain (0.6 s) is inside the shares build's own spread.
Callgrind counts instructions and not their cost, and the four-thread runs
share caches with the other session, so the counts are the firmer number
here; a quiet-host timing would settle it.

No opt-out flag was added. The remaining cost is a few per cent of user time,
and an opt-out would need the artifact to record that shares were not
measured (an absent list must not read as zero shares), which is a format
change ([artifact format versioning](../pipeline/artifact-format-versioning.md))
for that saving.
