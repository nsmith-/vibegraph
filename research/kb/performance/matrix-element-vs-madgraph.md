---
type: Measurement
title: "Matrix element per point against MadGraph's MATRIX1"
description: "ns per evaluation against MATRIX1 over 19 processes. At v0.2.0 on an Ice Lake VM: geomean 0.85× at default flags and 0.74× with both sides at x86-64-v3; earlier 0.87× (Cascade Lake, M3 Max)."
status: draft
tags: [performance, madgraph-comparison, matrix-element, benchmarks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
measured:
  - {commit: 3cc7f9c2, tag: v0.2.0, host: "Intel Xeon (Ice Lake, family 6 model 106), 4-vCPU KVM guest, AlmaLinux 10.2, rustc 1.99.1, gfortran 14.4.0", command: "VIBEGRAPH_BENCH_EXTRA_PROCESSES='…' taskset -c 2 scripts/mg_perf_compare.sh (×2), RUSTFLAGS unset or '-C target-cpu=x86-64-v3'; gen_amplitude.py ×3, median", data: "CERN host: /home/ncsmith/bench-v0.2.0/results/phase1/{cfg1,cfg2} (driver logs, make_opts_flags.txt, f2py_effective_flags.txt, isa_check.txt)"}
  - {commit: cf8b2b7, pr: 8, host: "Intel Xeon @ 2.80 GHz, Cascade Lake (family 6 model 85), 4-vCPU Firecracker VM", command: "VIBEGRAPH_BENCH_EXTRA_PROCESSES='…' taskset -c 2 scripts/mg_perf_compare.sh (×2); gen_amplitude.py ×3, median"}
  - {host: "Apple M3 Max (darwin), 2026-08-06, quiet host, MG side regenerated the same day", command: "env -u RUSTFLAGS scripts/mg_perf_compare.sh"}
  - {commit: 62d78e4, host: "Apple M3 Max, 2026-08-05", command: "scripts/mg_perf_compare.sh, after arm against before arm e951045, min over two alternating rounds"}
  - {commit: 405d18b, host: "Apple M3 Max, rustc 1.94.1, 2026-07-28", command: "scripts/mg_perf_compare.sh"}
sources:
  - {id: cl, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/mg-comparison-cascade-lake-results.md#L11-L99", title: "vibegraph against MadGraph on one x86 host, §0–§2"}
  - {id: cl-repro, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/mg-comparison-cascade-lake-results.md#L227-L241", title: "Cascade Lake results, Reproduce"}
  - {id: n32-6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L794-L864", title: "Note 32 §6, MATRIX1 re-measurement 2026-08-06"}
  - {id: n31-68, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L1242-L1303", title: "Note 31 §6.8, sprint-level mg_perf_compare"}
  - {id: td6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/topdown-zen4-results.md#L226-L380", title: "Top-down Zen 4 §6–§8, constant collection, bare configuration amplitudes, real pool"}
  - {id: n15, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/15-eval-optimization-plan.md#L546-L582", title: "Note 15 §2.4, cross-platform rerun kit"}
  - {id: n20, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/20-eval-perf-2-plan.md#L59-L90", title: "Note 20, fresh vs-MG measurement 2026-07-28"}
---

# Matrix element per point against MadGraph's MATRIX1

The narrow comparison: `eval_m2/forward` criterion medians (helicity- and colour-summed
`|M|²` at one phase-space point, scalar `f64`) divided by the bench's points per iteration,
against MadGraph's standalone `MATRIX1` ns per evaluation from
`validation/madgraph/output/mg_timings.json`. Both sides use default codegen. The tool is
[`mg_perf_compare.sh`](../tooling/mg-perf-compare.md). Integrand throughput and time to
accuracy, which carry the phase-space map, VEGAS, cuts and clustering, are a different
measurement: [integration against MadGraph](integration-vs-madgraph.md).

**Headline at v0.2.0 (Ice Lake VM): geomean 0.85× (ours/MadGraph) over 19 processes at
default flags, and 0.74× with both sides compiled for x86-64-v3.** Fourteen rows are faster
than MATRIX1 at default flags and sixteen at x86-64-v3; the slow side is still `uux_to_uux`,
`gg_to_gg` and `ee_to_wpwm`. Ours is the scalar interpreter in both configurations, with no
lane batching. The earlier tables (0.87× on Cascade Lake and the M3 Max) predate constant
collection and fused scaled sums, bare configuration amplitudes and real-pool folding
(`fold.rs`).[^td6] See [constant collection](constant-collection-and-fused-sums.md).

## Ice Lake VM, v0.2.0 (`3cc7f9c2`, 2026-11-09)

Both codes on one host in one sitting ([benchmark hosts](benchmark-hosts.md)), pinned to
core 2. vibegraph: the glibc `cargo bench` build of `eval_strategies` at the tag, not the
musl release asset; the mean of two runs, which agree to 2.1% (default) and 2.8% (v3) per
row. MadGraph: the pinned submodule (3.7.1), the per-row median of three `gen_amplitude.py`
runs, process directories launched with `--nb-core 4`.

The two configurations:
- **Default.** `RUSTFLAGS` unset. MadGraph with `global_flag = -O`, `aloha_flag` and
  `matrix_flag` empty, set explicitly so MadGraph's `__debug__` path cannot add
  `-fbounds-check`.
- **x86-64-v3 on both sides.** `RUSTFLAGS="-C target-cpu=x86-64-v3"`. MadGraph with
  `global_flag = '-O -march=x86-64-v3'` and `aloha_flag = -march=x86-64-v3`, and
  `-march=x86-64-v3` in `build_amplitude.sh`'s `--f77flags`, with the environment's
  `-march=nocona` removed from `FFLAGS` (see the trap below). Every compile line was
  checked for the flag, and `objdump` found ymm instructions in every row's module and in
  `libdhelas.a`.

In both, MadGraph's `MATRIX1` itself is compiled by f2py at `-O3 -funroll-loops` (the
harness's flags), while `libdhelas.a` and `libmodel.a` take the `make_opts` flags above:
"MadGraph defaults" describes the libraries only.

| process | default: MG ns | ours ns | ours/MG | v3: MG ns | ours ns | ours/MG |
|---|--:|--:|--:|--:|--:|--:|
| `ee_to_zh` | 424 | 418 | 0.99× | 411 | 354 | 0.86× |
| `uux_to_uux` | 601 | 856 | 1.43× | 583 | 694 | 1.19× |
| `pp_to_ll_qcd0` | 628 | 506 | 0.81× | 601 | 447 | 0.74× |
| `ee_to_mumu` | 631 | 500 | 0.79× | 600 | 430 | 0.72× |
| `ee_to_ttx` | 766 | 792 | 1.03× | 755 | 666 | 0.88× |
| `gg_to_ttx` | 1 564 | 1 666 | 1.06× | 1 403 | 1 301 | 0.93× |
| `ee_to_ee` | 1 677 | 1 359 | 0.81× | 1 601 | 1 097 | 0.69× |
| `ee_to_wpwm` | 1 771 | 2 287 | 1.29× | 1 612 | 1 761 | 1.09× |
| `ee_to_tatah` | 1 835 | 1 499 | 0.82× | 1 774 | 1 220 | 0.69× |
| `gu_to_epemu` | 1 947 | 1 343 | 0.69× | 1 778 | 1 025 | 0.58× |
| `uux_to_epemg` | 1 948 | 1 364 | 0.70× | 1 775 | 1 043 | 0.59× |
| `ddx_to_epemg` | 1 955 | 1 368 | 0.70× | 1 777 | 1 044 | 0.59× |
| `gux_to_epemux` | 1 962 | 1 340 | 0.68× | 1 776 | 1 052 | 0.59× |
| `gg_to_gg` | 2 343 | 2 750 | 1.17× | 2 044 | 2 346 | 1.15× |
| `ee_to_mumua` | 3 461 | 2 375 | 0.69× | 3 108 | 1 817 | 0.58× |
| `ud_to_epemud_qcd0` | 14 737 | 9 453 | 0.64× | 12 884 | 7 291 | 0.57× |
| `ee_to_mumu_tata_qcd0` | 14 892 | 9 999 | 0.67× | 13 263 | 7 864 | 0.59× |
| `uux_to_ccx_emmm_qcd0` | 292 181 | 201 065 | 0.69× | 267 537 | 162 796 | 0.61× |
| `bbx_to_ccx_emmm_qcd0` | 355 824 | 319 188 | 0.90× | 317 690 | 267 320 | 0.84× |

**Default: geomean 0.85×, range 0.64×–1.43×, 5 rows above 1.0. x86-64-v3: geomean 0.74×,
range 0.57×–1.19×, 3 rows above 1.0.** x86-64-v3 speeds ours up by 1.24× (geomean) and
MadGraph by 1.08×. Against the Cascade Lake table (different host, ratios only),
`gg_to_gg` moved from 1.37× to 1.17× and `bbx_to_ccx_emmm_qcd0` from 1.17× to 0.90×, while
`uux_to_uux` moved the other way, 1.24× to 1.43×, which is not explained.

## Cascade Lake, `cf8b2b7` (2026-09-26)

Both codes on one host in one sitting, as cross-host absolute times are out of scope.[^cl]
vibegraph: `rustc 1.98.1`, `RUSTFLAGS` unset (SSE2, no FMA), bench profile, pinned to one
core with `taskset`. MadGraph: the pinned submodule (3.7.1) in the pixi `madgraph`
environment, `MATRIX1` modules built by `build_amplitude.sh` under the environment's f2py
flags (`-O3 -funroll-loops -march=nocona -mtune=haswell`, SSE3). The MG column is the per-row
median of three pinned `gen_amplitude.py` runs (10 000-point batches; most rows repeat within
±5%, single-run outliers reached +33% and +41%). Ours is the mean of two runs, which agree to
1–7% per row and 0.88× / 0.87× in geomean. The bench's 8 rows were extended to the 19 with
`VIBEGRAPH_BENCH_EXTRA_PROCESSES`.

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

**Geomean 0.87×, range 0.67×–1.37×.** The five slower rows are `gg_to_gg` 1.37×,
`ee_to_wpwm` 1.29×, `gg_to_ttx` 1.25×, `uux_to_uux` 1.24×, `bbx_to_ccx_emmm_qcd0` 1.17×;
`ee_to_ttx` is at parity. MATRIX1 is 3.02× slower here than on the M3 Max, uniformly
(2.75–3.33× per process), and ours slows by about the same factor, so the per-point ratio
carries across hosts unchanged while end-to-end ratios do not. `host_info.py` reads only
macOS `sysctl`, so on Linux the CPU block of `mg_timings.json` is filled in by hand from
`/proc/cpuinfo`.

## M3 Max, 2026-08-06

Same 19 rows, quiet host, MG side regenerated the same day, run with `env -u RUSTFLAGS`
because that host's shell profile exports `-C target-cpu=native` while every recorded table
is default codegen: **geomean 0.87×, range 0.65×–1.54×, 13 of 19 below 1.0.**[^n32-6] The
slow rows are the same: `uux_to_uux` 1.54×, `gg_to_gg` 1.34×, `gg_to_ttx` and `ee_to_wpwm`
1.23×. The commit was not recorded.

Two traps this run exposed:
- **Sibling load.** A mid-sprint 0.95× was taken with other agent sessions resident on the
  host. The quiet re-run's improvement is attributed to host conditions, not proven (no
  bisect): holding the vibegraph medians fixed and swapping only the MG table gives 0.829×
  against the 2026-08-05 snapshot and 0.870× against the regenerated one, since MG itself
  read 4.8% faster that day. Note 32 likens this −13% to `validate`'s −12.7%, but that
  figure is the addendum's quiet-host wall *saving* from code changes (341.4 s against the
  earlier 391 s reference), not a host-load effect; the loaded `validate` run read +13.4%
  over the same reference, mixed with recompilation
  ([benchmark hosts](benchmark-hosts.md)). No clean host-load measurement backs the
  attribution.
- **A stale MG table joins to nothing.** A pre-`host`-block `mg_timings.json` has no
  `processes` key and read as an empty table; the script now names that case instead of
  reporting the empty join.

## The earlier points (14 processes, M3 Max)

| date | vibegraph commit | geomean ours/MG | beating MG |
|---|---|--:|--:|
| 2026-07-28 | `405d18b` | 1.24× (0.72×–1.69×) | — |
| 2026-08-05, before arm | `e951045` | 1.25× | 3 of 14 |
| 2026-08-05, after arm | `62d78e4` | 0.98× | 8 of 14 |

The 2026-08-05 pair ran both trees in one sitting, alternating round by round with each row
taking the minimum of two rounds, because the after arm alone read 0.98× in one round and
1.09× in the other.[^n31-68] Every row improved (−11% to −30%, evaluator −21.6% overall),
which matched the sum of the individual changes' own measurements (−21.5% predicted). The
1.24× point of 2026-07-28 is a trajectory point only.[^n20]

## Reading and reproducing

- **Ratios, never absolute ns**: clocks and microarchitectures differ, and MATRIX1 is
  straight-line Fortran while ours is an interpreter, so the gap need not be constant across
  hosts. MG's side is a warm-up plus one batch, so ratio shifts under ~10% are noise.[^n15]
- **Codegen fairness**: default codegen on both sides, or raise both together
  (`RUSTFLAGS="-C target-cpu=…"` *and* the same `-march` in `build_amplitude.sh`'s
  `--f77flags` and in the run card's `global_flag`/`aloha_flag`), never one side only.
- **The pixi `madgraph` environment's `FFLAGS` overrides `--f77flags`.** It carries
  `-march=nocona -mtune=haswell`, which f2py places after `--f77flags`; the last `-march`
  wins, so a `-march` passed only through `--f77flags` silently does not reach
  `MATRIX1`. `make_opts` shows nothing wrong. Check the "Fortran f77 compiler" line of the
  build log, and the instruction set of the built module. A run that missed this
  understated MadGraph's v3 speed by up to about 15% on the rows where `MATRIX1` dominates.
- **Quote multi-word run-card flags**: an unquoted `set global_flag -O -march=…` makes
  MadGraph lowercase `-O` to `-o`. The checked-in `mg_*.so` modules and `mg_timings.json`
  are host-specific; regenerate them on the host being measured.
- **Correctness first**: `pixi run validate-amplitudes` (`tests/amplitude_oracle.rs`) before
  any timing claim.
- Cascade Lake recipe:[^cl-repro]

```
pixi run -e madgraph python validation/madgraph/time_stages.py --out target/mg-host/runs <rows>
pixi run -e madgraph bash validation/madgraph/build_amplitude.sh <rows>
pixi run -e madgraph taskset -c 2 python validation/madgraph/gen_amplitude.py <rows>   # ×3, median
VIBEGRAPH_BENCH_EXTRA_PROCESSES='name=process;…' taskset -c 2 scripts/mg_perf_compare.sh
```

The hosts are described in [benchmark hosts](benchmark-hosts.md).

[^cl]: `mg-comparison-cascade-lake-results.md` §0–§2, at `cf8b2b7` (PR #8's branch).
[^n32-6]: Note 32 §6.
[^n31-68]: Note 31 §6.8.
[^n20]: Note 20, fresh vs-MG measurement.
[^cl-repro]: Cascade Lake results, Reproduce.
[^td6]: Top-down Zen 4 §6–§8 (in-process A/B on an Emerald Rapids VM).
[^n15]: Note 15 §2.4, cross-platform rerun kit, "Reading the result".
