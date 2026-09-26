# vibegraph against MadGraph on one x86 host — results

**Status: measurement record, 2026-09-26.** Tree: PR #8's branch at `cf8b2b7`.
`main`'s phase-space map options (#7) were merged afterwards. They keep stock
runs bit-identical: `ee_to_mumu`, `gg_to_gg` and `pp_to_jj` at seed 20260719
reproduce §3's σ, error, iteration and evaluation counts to every printed
digit on the merged tree.
This note re-measures the three comparisons the README headlines, with **both
sides on one host in one sitting**, as note 30 §1 requires. Every earlier table
is from an Apple M3 Max.

| | this host | M3 Max (notes 30–32, 34) |
|---|--:|--:|
| per point, ours/MG cost, geomean over 19 processes | **0.87×** | 0.87× |
| time to 0.1% on σ, MG/ours, geomean over 5 rows | **1.67×** | 3.84× |
| integrand throughput, ours/MG, geomean over 26 rows | **3.97×** | 8.76× |

**Headline.** The per-point ratio carries across hosts unchanged. The two
end-to-end ratios halve, and the evaluators are not the cause:
- MadGraph's own `MATRIX1` is 3.02× slower here than on the M3 Max, uniformly
  (2.75–3.33× per process).
- Ours slows by the same factor: `pp_to_llj` costs 20.8 µs per integrand point
  here and 6.9 µs there.
- What does *not* slow 3× is the rest of MadEvent's summed job CPU time
  (`<cumulated_time>`), which is mostly not matrix element. It is only
  1.1–1.6× above the M3 bank's (§3).

## 1. Host and builds

| | |
|---|---|
| CPU | Intel Xeon @ 2.80 GHz, family 6 model 85 stepping 7 (Cascade Lake), 4-vCPU Firecracker VM, 1 thread per core, 33 MiB L3, AVX-512 F/DQ/CD/BW/VL + VNNI |
| vibegraph | `rustc 1.98.1`, `RUSTFLAGS` unset (default x86-64 codegen, SSE2, no FMA): the bench profile for §2, `release-debug` for §4, `release` for §3's CLI |
| MadGraph | the pinned submodule, VERSION 3.7.1, through `validation/madgraph/mg5_pinned.sh`, in the pixi `madgraph` environment: conda-forge `gfortran 14.4.0`, Python 3.11, LHAPDF 6.5.6 |
| MG `MATRIX1` modules | built by `build_amplitude.sh` under the environment's f2py flags: `-O3 -funroll-loops -march=nocona -mtune=haswell` (SSE3) |
| MadEvent runs | `time_stages.py`, one full `.mg5` script per process (generate, output, compile, launch) into a scratch area. `nb_core = None`; `Running:` peaks at 4 concurrent jobs on the 4 cores |

The container was reprovisioned onto this host at about 23:25 UTC on
2026-09-25. Everything in this note ran after that; the Emerald Rapids figures
in `x86-avx2-perf-study-results.md` are from before it.

MadEvent's point counts reproduce the M3 bank's exactly on all 27 runs (seeded
integration), so both hosts did the same MadGraph work and only the time
differs.

## 2. The matrix element, per point

`scripts/mg_perf_compare.sh`, run twice with the bench pinned to one core. The
19 processes are the committed `mg_timings.json` set, which is also the README's
earlier set. The bench's own 8 `BENCH_ROWS` were extended to all 19 through
`VIBEGRAPH_BENCH_EXTRA_PROCESSES`, with process strings from the same manifest
registry.

MadGraph's column is the per-row median of three pinned `gen_amplitude.py`
timing runs (10 000-point batch each). Most rows repeat within ±5%; single-run
outliers reached +33% (`gux_to_epemux`) and +41% (`pp_to_ll_qcd0`). Ours is the
mean of the two runs, which agree to 1–7% per row and 0.88× / 0.87× in geomean.

| process | MG ns/eval | ours ns/eval | ours/MG |
|---|--:|--:|--:|
| `ee_to_zh` | 628 | 555 | 0.88× |
| `ee_to_mumu` | 935 | 693 | 0.74× |
| `uux_to_uux` | 948 | 1 179 | 1.24× |
| `pp_to_ll_qcd0` | 970 | 680 | 0.70× |
| `ee_to_ttx` | 1 097 | 1 081 | 0.99× |
| `gg_to_ttx` | 2 089 | 2 606 | 1.25× |
| `ddx_to_epemg` | 2 512 | 1 893 | 0.75× |
| `ee_to_ee` | 2 530 | 1 752 | 0.69× |
| `uux_to_epemg` | 2 541 | 1 954 | 0.77× |
| `gu_to_epemu` | 2 554 | 1 845 | 0.72× |
| `ee_to_tatah` | 2 574 | 2 154 | 0.84× |
| `gux_to_epemux` | 2 581 | 1 884 | 0.73× |
| `ee_to_wpwm` | 2 594 | 3 353 | 1.29× |
| `gg_to_gg` | 3 187 | 4 357 | 1.37× |
| `ee_to_mumua` | 4 656 | 3 319 | 0.71× |
| `ud_to_epemud_qcd0` | 20 555 | 13 716 | 0.67× |
| `ee_to_mumu_tata_qcd0` | 20 649 | 14 378 | 0.70× |
| `uux_to_ccx_emmm_qcd0` | 346 119 | 303 392 | 0.88× |
| `bbx_to_ccx_emmm_qcd0` | 435 831 | 510 536 | 1.17× |

**Geometric mean 0.87×, range 0.67×–1.37×.**
- Thirteen of the nineteen are faster than MadGraph, and `ee_to_ttx` is at parity.
- The five slower are the colour-dense rows and `W⁺W⁻`: `gg_to_gg` 1.37×,
  `ee_to_wpwm` 1.29×, `gg_to_ttx` 1.25×, `uux_to_uux` 1.24×, and
  `bbx_to_ccx_emmm_qcd0` 1.17×.

The M3 Max read 0.87× over the same 19, with the same four colour-dense rows
leading the slow side.

## 3. Time to 0.1% on σ

Note 32 §7's protocol, unchanged:

```
vibegraph integrate <proc card> --run-card <run card> \
    --target-rel 0.001 --seed {20260719,20260720,20260721} -j 1
```

- **Our side:** wall clock over the whole process, one core
  (`taskset -c 2`). The cards are each row's banked `Cards/proc_card_mg5.dat`
  and `run_card.dat`, and `validation/madgraph/dy13_*` for Drell–Yan.
- **MadGraph's side:** this host's own run of the same process. Its
  `<cumulated_time>` is scaled by 1/δ² to the χ²-scaled δ each of our seeds
  reached, using the σ ± err of the same run's `results.dat`.

| row | ours s | δ quoted | δ χ² | our Mpts | iterations | MG CPU s | MG s @δ | MG/ours |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 1.10 | 0.0414% | 0.0461% | 0.72 | 6/6/6 | 9.3 | 7.9 | **7.2×** |
| `dy13_default` | 9.43 | 0.0881% | 0.0972% | 2.28 | 18/17/22 | 118.1 | 33.1 | **3.5×** |
| `gg_to_gg` | 12.07 | 0.0628% | 0.0649% | 0.72 | 6/6/6 | 10.2 | 11.2 | **0.93×** |
| `pp_to_jj` | 53.40 | 0.0924% | 0.0991% | 1.84 | 16/15/15 | 12.6 | 60.6 | **1.13×** |
| `pp_to_llj` | 362.26 | 0.0843% | 0.0997% | 17.44 | 142/160/134 | 16.1 | 178.3 | **0.49×** |

**Geometric mean 1.67× over the five rows, and 2.27× over the first four.**

- **Seed spread.** Wall time varies 2% across seeds on `ee_to_mumu` and
  `gg_to_gg`, 6% on `pp_to_jj`, 17% on `pp_to_llj` and 26% on `dy13_default`.
  The larger spreads follow the iteration count, as on the M3 Max.
- **Our draws are close to the M3 Max's.** `dy13_default` spends the same
  mean 2.28M points. `pp_to_llj` stops at 142/160/134 iterations here against
  140/156/145 there, 17.44M against 17.64M evaluations on average. That is
  consistent with the targets rounding differently: default x86-64 has no FMA,
  so `mul_add_fast` rounds twice. Either way, the points each row needs are
  essentially unchanged.
- **Why the ratio moved.** Our wall time rose 2.8–3.2× against the M3 Max
  (`gg_to_gg` 3.95 → 12.07 s, `pp_to_jj` 18.83 → 53.40 s, `pp_to_llj` 122.5 →
  362.3 s), in line with the 3.02× `MATRIX1` slowdown. MadGraph's CPU-seconds
  rose only 1.10× (`ee_to_mumu`) to 1.47× (`pp_to_llj`).

**Where MadGraph's CPU time goes.** `gg_to_gg` is an example. Its `MATRIX1` accounts for
434 257 × 3.19 µs = 1.4 s of 10.2 CPU-s here, and 434 257 × 1.02 µs = 0.44 s of
7.1 CPU-s on the M3 Max. The remaining 8.8 and 6.7 CPU-s are everything else a
MadEvent job does: process start-up, grid I/O, phase-space and PDF work. That
part grew 1.3×.

Two readings are possible, and this sitting cannot tell them apart:
1. This host is simply less than 3× slower on that kind of work.
2. The M3 bank's `cumulated_time` was inflated. There, madevent ran 16
   concurrent jobs over 12 performance and 4 efficiency cores (note 30 §1). A
   job placed on an efficiency core, or waiting for a performance core, adds
   CPU-seconds without adding work.

Either way, the same-host ratio is the one this note reports. The M3 Max table
compared its own two sides, but on a hybrid CPU whose MadGraph denominator may
be inflated.

## 4. Integrand throughput

Note 30 §5.3's construction, unchanged. The 26 rows are note 31 §6.4's.
- **Our side.** `seeds × neval × niter` points from each row's report record,
  over its `duration_s`. The rows come from single-threaded runs of the two
  gates that write the integrals rows:

  ```
  RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1 taskset -c 2 cargo test -p vibegraph-lib \
    --profile release-debug --features extended-validation --test <validate_sigma|validate_hadronic> \
    -- --nocapture --test-threads=1
  ```

  Both passed: 6 and 13 tests, in 330 s and 733 s wall.
- **MadGraph's side.** Field 4 of this host's `results.dat` over its
  `<cumulated_time>`. `pp_to_ll` divides by the `dy13_default` run, as in
  note 30.
- **Budgets.** Our points per row are today's budgets, which differ from note
  30's. That does not bias the comparison: throughput is per point.

| row | our points | our s | our kpts/s | MG points | MG CPU s | MG kpts/CPU-s | ours/MG |
|---|--:|--:|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 180 000 | 0.45 | 399 | 236 641 | 9.3 | 25 | 15.7× |
| `ee_to_ee` | 800 000 | 2.42 | 331 | 233 174 | 6.9 | 34 | 9.7× |
| `ee_to_ttx` | 180 000 | 0.59 | 303 | 217 112 | 6.4 | 34 | 8.9× |
| `ee_to_wpwm` | 320 000 | 2.12 | 151 | 228 879 | 7.1 | 32 | 4.7× |
| `ee_to_zh` | 180 000 | 0.35 | 511 | 219 576 | 5.7 | 39 | 13.2× |
| `uux_to_mumu` | 180 000 | 0.49 | 365 | 215 607 | 6.4 | 34 | 10.8× |
| `uux_to_uux` | 240 000 | 3.20 | 75 | 262 909 | 6.8 | 39 | 1.9× |
| `gg_to_ttx` | 480 000 | 7.00 | 69 | 278 197 | 7.5 | 37 | 1.9× |
| `gg_to_gg` | 240 000 | 5.23 | 46 | 434 257 | 10.2 | 43 | 1.1× |
| `ee_to_mumua` | 640 000 | 3.40 | 188 | 546 061 | 15.0 | 36 | 5.2× |
| `ee_to_tatah` | 480 000 | 2.33 | 206 | 230 642 | 8.2 | 28 | 7.3× |
| `uux_to_epemg` | 480 000 | 6.85 | 70 | 633 977 | 15.4 | 41 | 1.7× |
| `ddx_to_epemg` | 480 000 | 7.14 | 67 | 557 306 | 13.5 | 41 | 1.6× |
| `gu_to_epemu` | 480 000 | 7.59 | 63 | 196 468 | 8.2 | 24 | 2.7× |
| `gux_to_epemux` | 480 000 | 7.54 | 64 | 181 778 | 7.8 | 23 | 2.7× |
| `ee_to_mumu_tata_qcd0` | 800 000 | 14.39 | 56 | 686 608 | 87.0 | 8 | 7.0× |
| `ud_to_epemud_qcd0` | 960 000 | 19.26 | 50 | 283 965 | 17.7 | 16 | 3.1× |
| `pp_to_ll` | 4 320 000 | 17.82 | 242 | 3 066 042 | 118.1 | 26 | 9.3× |
| `pp_to_bb` | 2 250 000 | 34.25 | 66 | 387 950 | 11.8 | 33 | 2.0× |
| `pp_to_bb_qcd2` | 2 250 000 | 46.62 | 48 | 401 968 | 12.4 | 32 | 1.5× |
| `pp_to_bb_fixed` | 2 250 000 | 24.31 | 93 | 191 164 | 8.6 | 22 | 4.1× |
| `pp_to_jj` | 3 750 000 | 121.14 | 31 | 229 869 | 12.6 | 18 | 1.7× |
| `pp_to_ll_scalefact2` | 2 250 000 | 18.11 | 124 | 193 558 | 9.6 | 20 | 6.1× |
| `pp_to_llj_fixed` | 7 500 000 | 152.93 | 49 | 176 036 | 13.8 | 13 | 3.8× |
| `pp_to_llj` | 4 500 000 | 97.46 | 46 | 176 272 | 16.1 | 11 | 4.2× |
| `pp_to_llj_dyn` | 7 500 000 | 202.33 | 37 | 176 036 | 15.5 | 11 | 3.3× |

**Geometric mean 3.97×, range 1.1× (`gg_to_gg`) to 15.7× (`ee_to_mumu`).**

- **Same shape as the M3 Max's 8.76×, compressed by the same MadGraph
  denominator as §3.** The cheapest leptonic rows lead, and the pure-gluon and
  colour-dense 2→2 rows sit at the floor.
- **Consistent with §3.** `pp_to_llj` draws 4.2× more points per second here
  and needs 17.44 / 1.95 ≈ 8.9× more of them for the same δ. That gives
  0.47×, against the measured 0.49×.

## 5. What these do not cover

- **Parallel scaling** (`-j 16`) and the **unweighting efficiencies** stay
  M3 Max figures. The first needs more than 4 cores. The second is a ratio of
  trial counts, which does not depend on the host.
- **The 2→6 integrand costs** (`probe_2to6_eval_cost`,
  `probe_2to6_density_decomposition`) and the banked-layer wall time were not
  re-measured.
- **`host_info.py` reads only macOS `sysctl`.** On Linux its CPU block is null,
  so the work-area `mg_timings.json`'s CPU identity was filled in by hand from
  `/proc/cpuinfo`.

## Reproduce

```
pixi install -e madgraph
VIBEGRAPH_FETCH_CONSENT=1 pixi run fetch-pdf && pixi run fetch-pdf-multigrid && pixi run fetch-refdata
bash -c '. validation/fetch_common.sh && vg_ensure_submodule'
# MadEvent runs, one full .mg5 script per process, into a scratch area:
pixi run -e madgraph python validation/madgraph/time_stages.py --out target/mg-host/runs <rows>
# per point: symlink the launched process directories over the bundle's thin
# ones in validation/madgraph/output, then
pixi run -e madgraph bash validation/madgraph/build_amplitude.sh <rows>
pixi run -e madgraph taskset -c 2 python validation/madgraph/gen_amplitude.py <rows>   # ×3, median
VIBEGRAPH_BENCH_EXTRA_PROCESSES='name=process;…' taskset -c 2 scripts/mg_perf_compare.sh
```
