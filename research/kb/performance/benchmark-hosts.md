---
type: Measurement
title: The hosts behind the performance numbers
description: "CPU, caches, measured clock, ISA, OS and toolchain of each benchmark host (M3 Max, Emerald Rapids VM, Cascade Lake VM, Zen 4), and the M3 Max run-to-run noise floor."
status: draft
tags: [performance, hosts, measurement-method, noise]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n30-host, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/30-perf-baseline-timings.md#L44-L69", title: "Note 30 §1 (M3 Max host and both sides' builds)"}
  - {id: n30-repro, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/30-perf-baseline-timings.md#L179-L187", title: "Note 30 §3.3 (run-to-run reproducibility)"}
  - {id: n32-closeout, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/32-perf-addendum-plan.md#L638-L748", title: "Note 32 §5.3 (close-out measurements, loaded vs quiet host)"}
  - {id: aot-host, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/aot-kernels-study-results.md#L52-L96", title: "AOT study §1 (Emerald Rapids host and method)"}
  - {id: roofline-host, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/roofline-census-results.md#L39-L58", title: "Roofline census §1 (Emerald Rapids clock and peaks)"}
  - {id: x86-avx512, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/x86-avx2-perf-study-results.md#L399-L415", title: "x86 study: Emerald Rapids AVX-512 re-measurement host"}
  - {id: x86-avx2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/x86-avx2-perf-study-results.md#L14-L45", title: "x86 AVX2 study host and figure of merit"}
  - {id: cl-host, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/mg-comparison-cascade-lake-results.md#L38-L55", title: "Cascade Lake comparison §1 (host and builds)"}
  - {id: cl-uncovered, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/mg-comparison-cascade-lake-results.md#L215-L226", title: "Cascade Lake comparison §5 (what is not covered)"}
  - {id: td-host, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/topdown-zen4-results.md#L13-L38", title: "Top-down counters on Zen 4: host and caveats"}
  - {id: fact-noise, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/facts/m3-max-host-timing-noise.md#L13-L28", title: "Phase-B fact: M3 Max timing noise (replaced by this concept)"}
  - {id: tds-hosts, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/threaded-dispatch-study-results.md#L32-L40", title: "Threaded-dispatch study: Cascade Lake host block"}
  - {id: tds-m3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/threaded-dispatch-study-results.md#L221-L240", title: "Threaded-dispatch study §4: M3 Max host and min-over-rounds estimator"}
  - {id: host-info, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/validation/madgraph/host_info.py#L20-L65", title: "validation/madgraph/host_info.py"}
measured:
  - {commit: b0e08d3, host: "Apple M3 Max, macOS 15.7 (loaded, load average 6–25)", command: "pixi run --skip-deps validate"}
  - {commit: 8cdb180, host: "Apple M3 Max, macOS 15.7 (quiet, load average 1.8)", command: "pixi run --skip-deps validate"}
---

# The hosts behind the performance numbers

Absolute times are never compared across hosts; only ratios taken on one host
in one sitting are (the rule and its consequences are in
[the end-to-end timing protocol](../performance/end-to-end-timing-protocol.md)).
This concept records what each host is, so a reader can tell which numbers are
comparable and why a ratio may differ between them. How a microbenchmark is run
on any of them is in [benchmarking the evaluator](../performance/microbenchmark-protocol.md).

## The hosts

| host | CPU and caches | clock | ISA used | OS, toolchain seen | notes |
|---|---|---|---|---|---|
| **M3 Max** (desktop) | Apple M3 Max, 12 performance + 4 efficiency cores, 16 logical; 128 KiB L1d per P-core, 16 MiB shared L2 per P-cluster; 48 GiB | not exposed by the OS (`sysctl hw.cpufrequency*` empty); recorded as `null` | aarch64, NEON, no SVE | macOS 15.7, Darwin 24.6; `rustc 1.94.1` for the baseline series, `nightly-2026-09-24` for the dispatch study | no core affinity is possible; whole stretches of a cell can run ~2.1× slow on an efficiency core |
| **Emerald Rapids VM** | Intel Xeon family 6 model 207, 4-vCPU Firecracker microVM under KVM, 15 GiB; 48 KiB L1d and 2 MiB L2 per core, 260 MiB L3 | reports 2.1 GHz base; **measured 3.2 GHz** (dependent integer `add` chain, 3.16–3.32 G/s pinned to an idle core) | AVX-512 F/DQ/BW/VL, FP16, BF16, AMX; `target-cpu=native` resolves to `emeraldrapids`, whose LLVM tuning carries `prefer-256-bit` | Linux; `rustc 1.94.1` (x86 study), `1.97.0` (AOT study) | no PMU; shared with other sessions at times; separate-process runs spread up to ±35% |
| **Cascade Lake VM** | Intel Xeon @ 2.80 GHz, family 6 model 85 stepping 7, 4-vCPU Firecracker VM, 15 GiB, 1 thread per core; 32 KiB L1d, 1 MiB L2 per core, 33 MiB L3 | 2.8 GHz | AVX-512 F/DQ/CD/BW/VL + VNNI | Linux; `rustc 1.98.1` (MadGraph comparison), `nightly-2026-09-24` (dispatch study) | no PMU; run-to-run drift 1–4% per cell; provisioned 2026-09-25 |
| **Zen 4** (bare metal) | AMD EPYC 9534 "Genoa", family 25 model 17; 1 MiB L2 per core | boost 3.1–4.1 GHz across cells, governor `performance` | AVX-512, 512-bit ops executed as two passes | RHEL 9, kernel 5.14, perf 5.14 | the only host with a usable PMU; NMI watchdog holds one of six core counters |

Sources: M3 Max[^n30-host][^tds-m3], Emerald Rapids[^aot-host][^roofline-host][^x86-avx512],
Cascade Lake[^cl-host][^tds-hosts], Zen 4[^td-host].

An unnamed **AVX2 + FMA host without AVX-512** (`zmm` absent from every dump)
ran the first x86 evaluator study; its CPU model is not recorded.[^x86-avx2]
The Emerald Rapids and Cascade Lake VMs are different machines: the container
was reprovisioned from the first onto the second at about 23:25 UTC on
2026-09-25, and Emerald Rapids figures predate that.[^cl-host]

### What differs between hosts that changes a ratio

- **L2 size decides the widest useful lane width on large programs.** The 2→6's
  op-blocked arenas at eight lanes fit the M3 Max's 16 MiB L2, overflow
  Cascade Lake's 1 MiB and sat at Emerald Rapids' 2 MiB cliff; see
  [execution order](../performance/execution-order.md) and
  [lane throughput](../performance/lane-throughput.md).
- **FMA in the default target.** Default x86-64 codegen (`RUSTFLAGS` unset) has
  no FMA, so `mul_add_fast` rounds twice there; `target-cpu=native` gives one FMA
  per component. The validation layer and the recorded reference tables assume
  default codegen.[^cl-host]
- **MadGraph's denominator.** MadEvent's summed job CPU time does not scale with
  host speed the way `MATRIX1` does, so end-to-end ratios against MadGraph halve
  from the M3 Max to Cascade Lake while the per-point ratio is unchanged; see
  [integration against MadGraph](../performance/integration-vs-madgraph.md).
- **Hybrid cores.** On the M3 Max, MadEvent ran up to 16 concurrent jobs over 12
  performance and 4 efficiency cores with no affinity; a job on an efficiency
  core adds CPU-seconds without adding work.

## Build settings recorded with the M3 Max baseline

vibegraph: profile `release-debug` (thin LTO, `opt-level = 3`, `debug = 1`,
`debug_assertions = false`), feature `extended-validation`, `RUSTFLAGS` unset.
MadGraph: the pinned submodule, VERSION 3.7.1, driven through
`validation/madgraph/mg5_pinned.sh`; `GNU Fortran 14.3.0`; Fortran flags from
each generated `Source/make_opts` carry no `-O` and no `-march` beyond MadGraph's
own `GLOBAL_FLAG`; `nb_core = None`.[^n30-host] On Cascade Lake the MadGraph
`MATRIX1` modules were built by `build_amplitude.sh` with the environment's f2py
flags `-O3 -funroll-loops -march=nocona -mtune=haswell`, and MadEvent ran at most
4 concurrent jobs.[^cl-host]

## The M3 Max noise floor

Two identical `validate` passes on the M3 Max agree to a **median 0.8%, worst
3.4%** over the 43 measurements above 1 s. A claimed sub-1% speedup is not
measurable at the per-row level on this host.[^n30-repro]

A loaded host costs more than that, and it costs wall time rather than CPU
time. With background indexing (`mds_stores`, `mediaanalysisd`) holding the load
average at 6–25, the identical `pixi run --skip-deps validate` read **443.3 s**
wall (2 453.8 s user). The same command on the same day on a quiet host (load
1.8, fully warm) read **341.4 s** wall (1 305.6 s user), reproducing an
independent checkout's 342.6 s / 1 321.3 s to ~1% on both axes. The loaded run
also included recompiling two test binaries, so its excess over the quiet run
(+30%) is load plus recompile, not separable.[^n32-closeout]

Wall time is the wrong instrument for a CPU saving on this host, because
`validate` runs its rows concurrently: a CPU saving shows in wall only to the
extent the run is CPU-bound. One licensed budget cut moved CPU by −288 s and
wall by only −18 s in the same sitting.[^n32-closeout]

The Phase-B fact this concept replaces quoted "a noisy host costs ~12.7%". That
figure is a quiet-host wall *saving* from a set of code changes (341.4 s against
the previous 391 s reference, −12.7%), not a noise cost. The loaded run read
+13.4% above that 391 s reference, which is the number closest to "a noise
cost", and it too mixes load with recompilation.[^fact-noise][^n32-closeout]

## Tooling gap

`validation/madgraph/host_info.py` reads only macOS `sysctl`. On Linux its CPU
block is null, so the Cascade Lake run's `mg_timings.json` CPU identity was
filled in by hand from `/proc/cpuinfo`.[^cl-uncovered][^host-info] Open as
[host-info-null-cpu-block-on-linux](../backlog/hygiene/host-info-null-cpu-block-on-linux.md).

[^n30-host]: Note 30 §1, written from `target/validation-report/host.json` and the MadGraph pass's `timings.json` host block.
[^n30-repro]: Note 30 §3.3.
[^n32-closeout]: Note 32 §5.3, close-out measurements of 2026-08-05.
[^aot-host]: AOT study §1.
[^roofline-host]: Roofline census §1, including the clock measurement and the microarchitectural peaks.
[^x86-avx512]: x86 study, AVX-512 re-measurement host block.
[^x86-avx2]: x86 AVX2 study header.
[^cl-host]: Cascade Lake comparison §1.
[^cl-uncovered]: Cascade Lake comparison §5.
[^td-host]: Top-down counters on Zen 4, host and measurement caveats.
[^fact-noise]: `research/notes/facts/m3-max-host-timing-noise.md`, the Phase-B fact replaced here.
[^tds-hosts]: Threaded-dispatch study, Cascade Lake host block (caches, toolchain, drift).
[^tds-m3]: Threaded-dispatch study §4, M3 Max host (caches, E-core slow stretches).
[^host-info]: `validation/madgraph/host_info.py`, `sysctl`-only `host_block`.
