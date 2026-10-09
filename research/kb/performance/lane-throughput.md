---
type: Measurement
title: "Lane versus scalar per-event cost, per host"
description: "LaneField per-event cost against scalar (0.57/0.32/0.25× at N=2/4/8 on Emerald Rapids), best width per host (Zen 4, Emerald Rapids), transpose exonerated; ARM not yet re-measured."
status: draft
tags: [performance, simd, lanes, benchmarks, hosts]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
measured:
  - {commit: 02e8b25, host: "Intel Xeon Emerald Rapids (family 6 model 207) at 2.1 GHz base, 4-vCPU Firecracker VM", command: "RUSTFLAGS='-C target-cpu=native' cargo bench -p vibegraph-lib --bench eval_strategies (and the x86-64-v3 and baseline builds)"}
  - {commit: db5fd03, host: "Intel Xeon Emerald Rapids, 4-vCPU Firecracker VM", command: "eval_strategies forward/lanes2/4/8, target-cpu=native"}
  - {commit: 5ced9bb, host: "AMD EPYC 9534 (Zen 4), bare metal", command: "scripts/topdown_kit.sh"}
  - {commit: 38c2410, host: "Intel Xeon Emerald Rapids, 4-vCPU cloud VM", command: "in-process A/B against 5a5e377, target-cpu=native"}
sources:
  - {id: x86-ratio, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L399-L456", title: "x86 study, AVX-512 re-measurement: per-event ratio to scalar (NumericArray)"}
  - {id: x86-fix, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L505-L668", title: "x86 study: force-inlining probe, corrections, the fix, the two release builds"}
  - {id: x86-arm, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L275-L324", title: "x86 study, ARM results: lane ratios and the transpose isolated"}
  - {id: rc3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/roofline-census-results.md#L141-L159", title: "Roofline census §3 reading (the 2→6 capacity cliff)"}
  - {id: td2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/topdown-zen4-results.md#L53-L127", title: "Top-down Zen 4 §2 slot accounting"}
  - {id: td4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/topdown-zen4-results.md#L192-L225", title: "Top-down Zen 4 §4–§5"}
  - {id: td6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/topdown-zen4-results.md#L226-L298", title: "Top-down Zen 4 §6 timing table"}
---

# Lane versus scalar per-event cost, per host

`lanesN ÷ forward` in `eval_strategies` is a per-event ratio as it stands: every bar runs
16 events per criterion iteration (`bench_lanes` iterates `chunks_exact(N)` over 16 points),
so no normalisation is needed. Below 1 the lane path is cheaper per event. How lanes are
built is [the lane field](lane-field-over-wide.md); how they would be used is
[lane-batched evaluation](simd-lane-evaluation.md). The hosts are described in
[benchmark hosts](benchmark-hosts.md).

## Emerald Rapids, `LaneField`, `02e8b25`

`target-cpu=native` (AVX-512, `prefer-256-bit` tuning; `LaneField<8>` uses zmm anyway),
bench profile (fat LTO), the 8 `BENCH_ROWS`:[^x86-fix]

| process | forward µs / 16 ev | lanes2 | lanes4 | lanes8 |
|---|--:|--:|--:|--:|
| `ee_to_mumu` | 8.0 | 0.57× | 0.30× | 0.24× |
| `ee_to_wpwm` | 30.5 | 0.62× | 0.42× | 0.34× |
| `uux_to_uux` | 13.0 | 0.57× | 0.30× | 0.26× |
| `gg_to_gg` | 43.4 | 0.59× | 0.32× | 0.23× |
| `gg_to_ttx` | 24.8 | 0.59× | 0.33× | 0.23× |
| `ee_to_mumua` | 33.9 | 0.57× | 0.33× | 0.26× |
| `ee_to_mumu_tata_qcd0` | 163.5 | 0.50× | 0.31× | 0.23× |
| `uux_to_ccx_emmm_qcd0` | 3047.1 | 0.58× | 0.37× | 0.33× |
| **median** | | **0.57×** | **0.32×** | **0.25×** |
| **suite Σ** (cost-weighted) | | 0.58× | 0.37× | 0.32× |

Per-event throughput is 1.75× / 3.1× / 4.0× of scalar (median), and wider is cheaper per
event, as SIMD predicts. `ee_to_wpwm` and the 2→6 gain least at N = 8, one small and one
large, so it is not simply size; the run does not isolate why. Run-to-run noise on this VM
is about 2–4%; each configuration is one run.

The previous `NumericArray` field ran 5.6× / 7.9× / 10.0× *slower* than scalar (median) on
the same host and rows, so the field change is roughly a 10× / 25× / 40× per-event speedup
of the lane path. That comparison is the point of [the lane field](lane-field-over-wide.md)
and is why those older ratios are quoted at all.[^x86-ratio]

### The two x86 release builds (same host, same rows)

`release.yml` ships a baseline `x86_64-unknown-linux-musl` asset and an `x86-64-v3` one.
Each build against its own scalar:

| build | lanes2 | lanes4 | lanes8 |
|---|--:|--:|--:|
| `x86-64-v3` (AVX2 + FMA) | 0.59× | 0.33× | 0.34× |
| baseline (SSE2), before `mul_add_fast` | 0.18× | 0.17× | 0.18× |

- Under v3, lanes8 is two ymm halves and buys nothing over lanes4: N = 4 is that target's
  width.
- **The baseline row is stale.** It was measured when every scalar `f64::mul_add` on that
  target was a software FMA call, which the lanes avoided. `Real::mul_add_fast` then made
  baseline scalar `forward` 2.3–3.8× faster (see [`mul_add_fast`](mul-add-fast.md)), so the
  baseline lane ratio must be re-measured before it informs any decision.

## Best width depends on the host and on the program

| host, commit | 2→6 width 8 vs width 4 | other rows |
|---|---|---|
| Emerald Rapids, `db5fd03` (before constant collection) | 1.03× (no gain) | 1.2–1.4× |
| Emerald Rapids, `38c2410` (after constant collection) | width 8 ahead: 44.4 vs 46.6 µs/event | width 8 ahead on every row |
| Zen 4 (EPYC 9534), `5ced9bb` (before constant collection) | width 8 ahead by 1.28× in cycles | width 8 ahead by 1.18–1.38× |
| Cascade Lake (1 MiB L2), `6bd7325` | at lanes8 the op-blocked order's larger live set costs 15–18% against lower-footprint orders | — |
| M3 Max (16 MiB L2) | no working-set effect at any width | — |

- **The Emerald Rapids 1.03× was a working-set ceiling of the program at `db5fd03`, not a
  host property.** Its 2→6 arenas were 1.8 MiB at lanes4 and 3.6 MiB at lanes8 against a
  2 MiB L2.[^rc3] Constant collection removed the single-use scalings, which hold no arena
  slots once fused, shrinking the lanes8 working set to 2.3 MiB; width 8 then beat width 4
  on the same host.[^td6] Quote each figure with its commit.
- **On Zen 4 width 8 wins on every row**, even though 512-bit ops run in two passes,
  because it executes fewer instructions per event; it is also where the run turns
  back-end-bound (FP pipes 53–70% busy, the 2→6 missing 27% of L1D loads).[^td2]
- Under `target-cpu=native` on Zen 4, *width 1* gets SLP-vectorised into two-pass 512-bit
  code and its `GammaVout` costs more per call than width 4's for four events, so width-1
  numbers on AVX-512 hosts measure that, not scalar code.[^td4]
- A lane-aware order fallback (switch order when arena bytes × width exceeds L2) is open as
  [a backlog item](../backlog/performance/schedule-fallback-lane-blind.md); size its payoff
  at the width a production lane path would use.

## The transpose is exonerated

`eval_m2_lanes` = `pack_lane_points` (the AoS→SoA transpose, allocation included) then
`eval_m2_lanes_packed`; the `lanes{N}_prepacked` bench hoists the first out of the timed
region, so the difference prices the transpose. On the M3 Max the mean share was ≤ 0.65%
at every width, inside the 1–1.5% noise floor, a width-independent ~7 ns per event on
`ee_to_mumu` (one `Vec` allocation plus `n_ext × 4` lane packs).[^x86-arm] On Emerald Rapids
`_prepacked` sat −6.6% to +1.7% of the lane bar. Both measurements were taken under the
`NumericArray` field; the transpose code is unchanged by the field swap, and its absolute
cost is small against any lane bar measured since.

## ARM: no current number

The only ARM lane measurement (M3 Max) used `NumericArray` and showed the best width 2.4–2.9×
slower than scalar. That loss was the inlining failure that also sank x86, so it says
nothing about `LaneField` on NEON. No ARM lane ratio exists for the current field; it needs a
run on the user's machine ([backlog](../backlog/performance/lane-field-unmeasured-on-arm.md)).

[^x86-fix]: x86 study, "The fix" bench table and "The two x86 release builds", at `02e8b25`.
[^x86-ratio]: x86 study, "Per-event ratio to scalar" (NumericArray, same host and rows).
[^rc3]: Roofline census §3, at `db5fd03`.
[^td6]: Top-down Zen 4 §6, in-process A/B, `target-cpu=native`, on an Emerald Rapids VM.
[^td2]: Top-down Zen 4 §2, at `5ced9bb`.
[^td4]: Top-down Zen 4 §4–§5.
[^x86-arm]: x86 study, ARM results, the transpose isolated.
