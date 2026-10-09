---
type: Measurement
title: "Integration against MadGraph: throughput and time to accuracy"
description: "Integrand points per CPU-second and CPU seconds to 0.1% on σ against MadEvent on M3 Max and Cascade Lake, and why cumulated_time puts an O(1) factor into every such ratio."
status: draft
tags: [performance, madgraph, integration, throughput, measurement]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n30-throughput, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/30-perf-baseline-timings.md#L352-L407", title: "Note 30 §5.3 (throughput on a denominator that means something)"}
  - {id: n31-throughput, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L1036-L1103", title: "Note 31 §6.4 (throughput recomputed)"}
  - {id: n32-tta, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L865-L955", title: "Note 32 §7 (time to a target accuracy, protocol)"}
  - {id: n34-tta, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/34-draw-followup-plan.md#L338-L415", title: "Note 34 §3 (time to accuracy, remeasured post-sprint)"}
  - {id: cl-summary, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/mg-comparison-cascade-lake-results.md#L11-L37", title: "Cascade Lake comparison: headline"}
  - {id: cl-tta, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/mg-comparison-cascade-lake-results.md#L100-L156", title: "Cascade Lake comparison §3 (time to 0.1% on σ)"}
  - {id: cl-throughput, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/mg-comparison-cascade-lake-results.md#L157-L214", title: "Cascade Lake comparison §4 (integrand throughput)"}
  - {id: cl-uncovered, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/mg-comparison-cascade-lake-results.md#L215-L226", title: "Cascade Lake comparison §5 (not covered)"}
  - {id: integrate-cli, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-cli/src/integrate.rs#L285-L335", title: "integrate CLI budget and convergence flags"}
measured:
  - {host: "Apple M3 Max", command: "RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1 per-row validate_sigma / validate_hadronic"}
  - {commit: 098c9e2, host: "Apple M3 Max", command: "vibegraph integrate <proc card> --run-card <run card> --target-rel 0.001 --seed {20260719,20260720,20260721} -j 1"}
  - {commit: cf8b2b7, pr: 8, landed_in: 6e15ad3, host: "Intel Xeon @ 2.80 GHz, Cascade Lake (family 6 model 85), 4-vCPU Firecracker VM", command: "taskset -c 2 vibegraph integrate … --target-rel 0.001 -j 1; per-row gates under RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1"}
---

# Integration against MadGraph: throughput and time to accuracy

Two end-to-end comparisons with MadEvent, each taken with both sides on one host
in one sitting. **Integrand throughput** prices a point. **Time to a target
accuracy** is what a user waits for: it also sees whether the points were worth
drawing. The per-point matrix-element comparison (`MATRIX1`) is a different
measurement, in [matrix element against MadGraph](../performance/matrix-element-vs-madgraph.md);
the protocol rules are in
[the end-to-end timing protocol](../performance/end-to-end-timing-protocol.md).

Headline, geomeans:[^cl-summary]

| | Cascade Lake VM | M3 Max |
|---|--:|--:|
| per point, ours/MG `MATRIX1` cost, 19 processes | 0.87× | 0.87× |
| time to 0.1% on σ, MG/ours, 5 rows | **1.67×** | **3.84×** |
| integrand throughput, ours/MG, 26 rows | **3.97×** | **8.76×** |

The per-point ratio carries across hosts unchanged; both end-to-end ratios
halve. The evaluators are not the cause (below).

## Integrand throughput

**Construction.** Our points are `seeds × neval × niter` VEGAS evaluations from
each row's report record, over its single-thread `duration_s` (per-row protocol,
`RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1`). MadGraph's are field 4 of
`SubProcesses/results.dat` (MadEvent's own count of points behind the banked
result) over that file's `<cumulated_time>`, the summed CPU seconds of its
Fortran jobs, which takes its job farm out of the comparison. `pp_to_ll` divides
by the `dy13_default` run.[^n30-throughput][^n31-throughput][^cl-throughput]

| row | M3 Max ours/MG | Cascade Lake ours/MG |
|---|--:|--:|
| `ee_to_mumu` | 37.3× | 15.7× |
| `ee_to_ee` | 24.1× | 9.7× |
| `ee_to_ttx` | 25.3× | 8.9× |
| `ee_to_wpwm` | 13.8× | 4.7× |
| `ee_to_zh` | 34.6× | 13.2× |
| `uux_to_mumu` | 29.9× | 10.8× |
| `uux_to_uux` | 3.9× | 1.9× |
| `gg_to_ttx` | 3.7× | 1.9× |
| `gg_to_gg` | 1.9× | 1.1× |
| `ee_to_mumua` | 10.7× | 5.2× |
| `ee_to_tatah` | 19.9× | 7.3× |
| `uux_to_epemg` | 2.5× | 1.7× |
| `ddx_to_epemg` | 2.7× | 1.6× |
| `gu_to_epemu` | 5.1× | 2.7× |
| `gux_to_epemux` | 5.3× | 2.7× |
| `ee_to_mumu_tata_qcd0` | 13.7× | 7.0× |
| `ud_to_epemud_qcd0` | 5.8× | 3.1× |
| `pp_to_ll` | 23.3× | 9.3× |
| `pp_to_bb` | 4.0× | 2.0× |
| `pp_to_bb_qcd2` | 3.5× | 1.5× |
| `pp_to_bb_fixed` | 12.5× | 4.1× |
| `pp_to_jj` | 4.6× | 1.7× |
| `pp_to_ll_scalefact2` | 12.9× | 6.1× |
| `pp_to_llj_fixed` | 9.1× | 3.8× |
| `pp_to_llj` | 8.2× | 4.2× |
| `pp_to_llj_dyn` | 6.8× | 3.3× |
| **geomean** | **8.76×** | **3.97×** |

The shape is the same on both hosts: the cheapest leptonic rows lead, and the
pure-gluon and colour-dense 2→2 rows sit at the floor. `gg_to_gg` is the floor
everywhere: no PDF work to win back and the densest colour algebra in the census.
Throughput is per point, so a budget change is invisible in it by construction.

## Time to 0.1% on σ

**Protocol.** Ours: `vibegraph integrate <proc card> --run-card <run card>
--target-rel 0.001 --seed {20260719,20260720,20260721} -j 1`, wall clock over the
whole process (model load, enumeration and evaluator compilation inside).
MadGraph: the same run's `<cumulated_time>` and σ ± err, scaled by 1/δ² to the
χ²-scaled δ each of our seeds reached. On the M3 Max the MadGraph side is the
banked run; on Cascade Lake it is that host's own run of the same
process.[^n32-tta][^cl-tta]

| row | channels | M3 Max: ours s | MG s @δ | MG/ours | Cascade Lake: ours s | MG s @δ | MG/ours |
|---|--:|--:|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 2 | 0.39 | 6.9 | **17.6×** | 1.10 | 7.9 | **7.2×** |
| `dy13_default` | 4 | 3.09 | 26.0 | **8.4×** | 9.43 | 33.1 | **3.5×** |
| `gg_to_gg` | 4 | 3.95 | 8.7 | **2.2×** | 12.07 | 11.2 | **0.93×** |
| `pp_to_jj` | 19 | 18.83 | 48.7 | **2.6×** | 53.40 | 60.6 | **1.13×** |
| `pp_to_llj` | 24 | 122.54 | 121.2 | **0.99×** | 362.26 | 178.3 | **0.49×** |
| **geomean** | | | | **3.84×** | | | **1.67×** |

M3 Max at `098c9e2`; `pp_to_llj` needs 140–156 iterations (16.8–18.7M
evaluations) and converges at the CLI's default `--max-iters 500`; its σ
(505.54–505.81 pb) sits +0.18–0.23% above the banked 504.63 ± 1.67, consistent
with MadGraph's own last-three combination reading 0.146% below its iterations
recombined by point count.[^n34-tta][^integrate-cli] On Cascade Lake the draws
are close to the M3 Max's (`pp_to_llj` 142/160/134 iterations, 17.44M against
17.64M evaluations; default x86-64 has no FMA, so `mul_add_fast` rounds twice
and the stop lands differently).[^cl-tta]

The ratio decomposes, and the decomposition closes arithmetically: time ratio ≈
throughput ratio ÷ points ratio. On `pp_to_llj` (M3 Max) 9.0× MadGraph's points
at ~8–9× its throughput gives ~1; on Cascade Lake 4.2× throughput over 8.9× the
points gives 0.47× against 0.49× measured. `pp_to_llj`'s remaining deficit is map
quality (9× the points for the same accuracy) times its χ²/dof ≈ 1.38, both
pointing at the cut-boundary concentration at the map's lower edge.[^n34-tta]
Seed spread follows the iteration count: 2% on `ee_to_mumu` and `gg_to_gg`, up
to 17% on `pp_to_llj` and 26% on `dy13_default`. How `--target-rel` decides to
stop is in [the convergence stop rule](../phase-space/convergence-stop-rule.md).

## Why the ratios halve between hosts

MadGraph's `MATRIX1` is 3.02× slower on Cascade Lake than on the M3 Max,
uniformly (2.75–3.33× per process), and ours slows by the same factor
(`pp_to_llj` 20.8 µs per integrand point against 6.9 µs). What does not slow 3×
is the rest of MadEvent's summed job CPU, which is mostly not matrix element:
it rose only 1.10× (`ee_to_mumu`) to 1.47× (`pp_to_llj`). On `gg_to_gg`,
`MATRIX1` is 1.4 of 10.2 CPU-s on Cascade Lake and 0.44 of 7.1 CPU-s on the M3
Max; the remainder (process start-up, grid I/O, phase space, PDFs) grew only
1.3×.[^cl-summary][^cl-tta]

Two readings fit, and one sitting cannot tell them apart: Cascade Lake is simply
less than 3× slower on that kind of work; or the M3 Max bank's `cumulated_time`
was inflated, because madevent ran 16 concurrent jobs over 12 performance and 4
efficiency cores, and a job on an efficiency core, or waiting for a performance
core, adds CPU-seconds without adding work. Either way, every ratio built on
`<cumulated_time>` carries a host-dependent factor of order one, and only a
same-host ratio is reported. Open as
[madevent-cumulated-time-host-dependence](../backlog/performance/madevent-cumulated-time-host-dependence.md).

## Caveats on every MadGraph column

- Whether `results.dat`'s point count includes the survey pass as well as the
  refine passes was never established, so a systematic factor of order one sits
  on every MadGraph throughput entry; it cannot depend on process complexity, so
  the trend across rows survives it.[^n30-throughput] (Open in
  [timing-baseline-unmeasured-stages](../backlog/performance/timing-baseline-unmeasured-stages.md).)
- Our per-point work is not MadGraph's per-point work: ours carries the
  multichannel map, the VEGAS grid, the cuts and the live configuration draw;
  MadGraph's carries its own analogues. Throughput is integrand throughput, not a
  matrix-element ratio.
- Parallel scaling (`-j 16`) and unweighting efficiencies are M3 Max figures
  only; the 4-core VMs cannot measure the first, and the second is a ratio of
  trial counts independent of host. The 2→6 integrand costs and the banked-layer
  wall were not re-measured on Cascade Lake.[^cl-uncovered]
- Same-host only: the M3 Max and Cascade Lake columns are never divided into
  each other except to locate the denominator effect above.

[^n30-throughput]: Note 30 §5.3, the construction and its two caveats.
[^n31-throughput]: Note 31 §6.4, M3 Max throughput recomputed (the "now" column used here).
[^n32-tta]: Note 32 §7, time-to-accuracy protocol.
[^n34-tta]: Note 34 §3, M3 Max time to accuracy at `098c9e2`, including `pp_to_llj`'s convergence and the iteration-cap change.
[^cl-summary]: Cascade Lake comparison, headline and the `MATRIX1` slowdown.
[^cl-tta]: Cascade Lake comparison §3.
[^cl-throughput]: Cascade Lake comparison §4.
[^cl-uncovered]: Cascade Lake comparison §5.
[^integrate-cli]: `--max-iters` (default 500) and `--target-rel` in `vibegraph-cli/src/integrate.rs`.
