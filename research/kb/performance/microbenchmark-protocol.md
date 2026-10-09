---
type: Procedure
title: "Benchmarking the evaluator"
description: "eval_strategies (16 events per iteration, BENCH_ROWS), min over rounds, layout-noise padding, in-process A/B slicing, call and lane asm census, re-measuring lanes on a new host."
status: draft
tags: [performance, benchmarks, methodology, criterion, simd]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n15-21, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/15-eval-optimization-plan.md#L262-L280", title: "Note 15 §2.1, the honest bench versus the extended-validation harness"}
  - {id: n15-24, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/15-eval-optimization-plan.md#L546-L585", title: "Note 15 §2.4, cross-platform rerun kit"}
  - {id: n18-kit, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L697-L748", title: "Note 18 H4, AVX-512 rerun kit"}
  - {id: tds2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/threaded-dispatch-study-results.md#L93-L122", title: "Threaded-dispatch study §2, three traps"}
  - {id: tds4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/threaded-dispatch-study-results.md#L221-L296", title: "Threaded-dispatch study §4, min over rounds on the M3 Max"}
  - {id: tds5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/threaded-dispatch-study-results.md#L371-L428", title: "Threaded-dispatch study §5, profile pitfalls"}
  - {id: tds6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/threaded-dispatch-study-results.md#L460-L511", title: "Threaded-dispatch study §6, the memory-layout confound"}
  - {id: tds-repro, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/threaded-dispatch-study-results.md#L583-L608", title: "Threaded-dispatch study, Reproduce"}
  - {id: td0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/topdown-zen4-results.md#L13-L38", title: "Top-down Zen 4, kit and caveats"}
  - {id: td6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/topdown-zen4-results.md#L226-L298", title: "Top-down Zen 4 §6, in-process A/B timing"}
  - {id: x86-repro, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L205-L230", title: "x86 study, Reproduce and ARM setup (profiles, 16 events per bar)"}
  - {id: x86-bench, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L558-L568", title: "x86 study, bench changes (BENCH_ROWS)"}
  - {id: x86-alg, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L700-L748", title: "x86 study, algebraic float protocol (pinned core, interleaved rounds)"}
  - {id: x86-repro2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L835-L844", title: "x86 study, AVX-512 Reproduce"}
---

# Benchmarking the evaluator

How to time a change to the helicity evaluator so the number means something. The binding
rules on statistics and evidence are in `AGENTS.md`; this page is the evaluator-specific
recipe and the traps behind it. Profiling (where time goes, as opposed to how much) is in
[profiling](../tooling/profiling.md) and
[instruction-level profiling](../tooling/instruction-level-profiling.md); the MadGraph
per-point comparison is [`mg_perf_compare`](../tooling/mg-perf-compare.md).

## 1. Correctness before timing

A timing claim needs a green gate on the same tree and flags:
- `pixi run validate-amplitudes` (`vibegraph-lib/tests/amplitude_oracle.rs`), the MadGraph
  amplitude oracle. A tolerance failure on new silicon is a finding, not noise.
- `cargo test -p vibegraph-lib --release --lib lanes` for anything touching lane code
  (`eval_m2_lanes_match_scalar`, `lanes4_lanes8_match_scalar`, bit-exact). Run it under the
  same `RUSTFLAGS` as the bench.
- Name the change's class: order-preserving changes must be bit-identical (compare
  `to_bits`, or digest the banked row files); reassociating changes gate at the oracle's
  tolerances.

## 2. The bench

`cargo bench -p vibegraph-lib --bench eval_strategies` (bench profile: `release`, fat LTO):

- **Strategies**: `forward` (scalar `eval_m2`), `lanes{2,4,8}` (`eval_m2_lanes` at
  `LaneField<N>`), `lanes{N}_prepacked` (transpose hoisted out, so the difference prices it).
- **Every bar is 16 events per iteration**: `bench_lanes` iterates `chunks_exact(N)` over the
  same 16 points, so `lanesN ÷ forward` is already a per-event ratio.[^x86-repro]
- **Rows**: the fixed `BENCH_ROWS` set (`ee_to_mumu`, `ee_to_wpwm`, `uux_to_uux`, `gg_to_gg`,
  `gg_to_ttx`, `ee_to_mumua`, `ee_to_mumu_tata_qcd0`, `uux_to_ccx_emmm_qcd0`). Each names a
  `validation/manifest.toml` row with an `mg_amplitude` table, whose process string is benched,
  built against the interned SM; a row naming a UFO model of its own is rejected rather than
  silently benched as an SM process (which once happened to four `*_smlimit`
  rows).[^x86-bench] `VIBEGRAPH_BENCH_EXTRA_PROCESSES='name=process;…'` adds rows for a
  study.
- **Order studies**: with `--features eval-schedule-study`, `VIBEGRAPH_EVAL_SCHEDULE`
  selects the execution order at program build, so one binary serves every order
  (`scripts/bench_schedule.sh`).
- **Never time under `extended-validation`.** That feature compiles the per-node
  cross-check (`cross_check_node`) into the `eval_m2` loop and roughly doubles ns/eval. The
  honest numbers are the bench's; the gates' `duration_s` is not a benchmark.[^n15-21]
- Kernel-level changes below the 8-row bench's resolution go to
  `benches/lorentz_kernels.rs` (throughput and latency-chain shapes, `slash` as an
  unchanged control); see [float reassociation](kernel-float-reassociation.md).

## 3. Estimators and noise, per host

| host | estimator | noise floor | source |
|---|---|---|---|
| M3 Max (12P + 4E, no affinity control) | **min over rounds**; whole stretches run ~2.1× slow on E-cores, so the median is unusable (it once put a 1.087 cell at 1.275) | ~1–1.5% run to run on stable cells | [^tds4] |
| Cascade Lake VM (4 vCPU) | criterion median, cells round-robin A B A B | 1–4% per cell; geomean inside ±2% is a tie | threaded-dispatch study |
| Emerald Rapids VM (4 vCPU) | three interleaved rounds on one pinned core, per-round ratio to a control | 2–4% run to run; code layout alone moves cells up to ~10%; separate-process runs spread up to ±35% | [^x86-alg][^td6] |

Report a geometric mean of per-row ratios with the per-row range beside it, and the host.
Absolute times are compared only on one host in one sitting.

**Memory layout is a confound on scalar rows.** The byte-identical `match` binary moved +10%
on the 2→6 and −17% on `ee_to_mumu` between two sweeps; `ee_to_mumu` ran 3.92 µs with 0–128
bytes of extra environment and 4.80 µs (+22%) with 136–300. `scripts/bench_schedule.sh`
therefore pads the environment by a different length each round, the same for every arm, and
min over rounds takes each cell's best layout.[^tds6] Lanes columns moved at most 2 points;
single scalar rows move 10–22%. A sweep measured in a single layout carries a few points of
layout error on its scalar geomean.

**In-process A/B slicing** removes most of that on a noisy VM:[^td6] a scratch driver links
the previous commit's library (renamed) and the new one, builds both evaluators for a row and
width, then alternates 100 ms slices (400 ms on the 2→6) of each on one pinned core, 40
slices apiece. Speedup is total base time over total new time; the bracket is the 10th–90th
percentile of per-slice ratios.

**Wall-clock artefacts**: `RUSTFLAGS` leaks from the shell profile (one host exports
`-C target-cpu=native`; run with `env -u RUSTFLAGS` when the tables assume default codegen);
sibling processes on a shared host moved a whole-table geomean by ~13%.

## 4. Check the code you are timing

- **Diff the out-of-line call census between builds, not only the timings.** A permuted
  `Instr::kind()` once compiled to a table lookup whose cost tipped soft-`#[inline]` kernels
  out of line (116 calls against 0), and passing `Instr` by value made LLVM load all 20 bytes
  before the jump and spill the loop counter (7% on lanes).[^tds2]
- **Returning a lane composite by value copies it.** A `ComplexVector` at `LaneField<8>` is
  512 B; an accessor that returned by value instead of `&T` cost a uniform +60% at lanes8 with
  scalar unaffected.
- **Lane code**: `RUSTFLAGS="-C target-cpu=native" scripts/dump_lane_asm.sh 'fill_arenas'`
  reports, per monomorphisation, instructions, packed `pd` on xmm/ymm/zmm, scalar `sd`,
  `calls`/`arith_calls`, `memcpy` and an inlining verdict. Read `arith_calls` and the verdict
  first: out-of-line callees census as packed whatever the lanes cost. Packed `pd` on zmm is
  genuine 8-lane AVX-512; xmm presence means nothing (scalar float math lives there too);
  ymm-only despite `avx512f` is LLVM's `prefer-256-bit` (try `-C target-cpu=znver4` or
  `x86-64-v4`). Legacy mangling omits generic arguments, so instances differ only by hash
  suffix; the widest-register instance is the widest N.
- **Profiles**: `samply` on the bench binary with `--profile-time`; build with
  `CARGO_PROFILE_BENCH_DEBUG=line-tables-only` or use the `release-debug` profile (thin LTO,
  `debug = 1`; the old `profiling` profile no longer exists). Don't chase `libsystem_kernel`
  samples (a blocked thread's wall clock) or dylib import stubs, which symbolicate to the
  preceding text symbol (`RawVec::reserve`).[^tds5] Sample skid makes a dispatch tail look
  hot; per-instruction shares say where the core waits, not what an instruction costs.

## 5. Counters

- **Linux, bare metal with a PMU**: `scripts/topdown_kit.sh` (5 s per pass, perf started
  disabled and switched on around the timed loop of the `eval_loop` example) and
  `scripts/topdown_summary.py`. Pin the core, set the governor to `performance`, compare
  cycles rather than ns when boost moves the clock, and expect multiplexing when the NMI
  watchdog holds a counter.[^td0] Example reading: [top-down on Zen 4](topdown-zen4.md).
- **Apple silicon**: `scripts/xctrace_bottlenecks.sh <eval_strategies binary> '<filter>'
  <orders…>` (Instruments CPU Counters, bottleneck mode, no `sudo`) gives useful / delivery /
  processing / discarded shares. "Discarded" is all flushed work after any misprediction, not
  an indirect-branch count.[^tds-repro]
- The cloud VMs expose no PMU; a counter-free reading is the [roofline census](roofline-census.md).

## 6. Re-measuring lanes on a new host

1. Confirm the vector units: `grep -m1 -o 'avx512[a-z0-9]*' /proc/cpuinfo` (or the ARM
   equivalent). On x86 the default target is SSE2 with no FMA.
2. Fix `RUSTFLAGS` (e.g. `-C target-cpu=native`, or `x86-64-v3` for the release asset's
   target) and keep it for every step: scalar and lanes are compared against each other, and
   bit identity holds only between builds that agree on `HARDWARE_FMA`.
3. Run the lane gate (§1), then the bench (§2), then the census (§4).
4. Read `lanesN ÷ forward` per row and suite Σ; record the host, commit and flags. Compare
   with [lane throughput](lane-throughput.md). If N = 8 wins, a 16-wide row is a one-line
   `bench_lanes::<16>` addition.[^n18-kit]

## 7. Re-running the MadGraph per-point comparison on another host

Regenerate the MG side natively (`pixi run -e madgraph generate-amplitude`; the checked-in
`mg_*.so` and `mg_timings.json` are host-specific), run the oracle, then
`scripts/mg_perf_compare.sh` (`--skip-bench` re-joins existing criterion results; each row
prints its measurement date). Raise codegen on both sides together or neither, and compare
ratios, never absolute ns; MG's side is a warm-up plus one batch, so shifts under ~10% are
noise.[^n15-24] Results: [matrix element against MadGraph](matrix-element-vs-madgraph.md).

Hosts and their quirks are listed in [benchmark hosts](benchmark-hosts.md).

[^n15-21]: Note 15 §2.1; `cross_check_node` in `vibegraph-lib/src/helas/eval/run.rs` is `#[cfg(any(test, debug_assertions, feature = "extended-validation"))]`.
[^n15-24]: Note 15 §2.4 (its `validate-helas-mg` step is now `validate-amplitudes`).
[^n18-kit]: Note 18 H4, AVX-512 rerun kit.
[^tds2]: Threaded-dispatch study §2.
[^tds4]: Threaded-dispatch study §4.
[^tds5]: Threaded-dispatch study §5.
[^tds6]: Threaded-dispatch study §6.
[^tds-repro]: Threaded-dispatch study, Reproduce.
[^td0]: Top-down Zen 4, header and caveats.
[^td6]: Top-down Zen 4 §6.
[^x86-repro]: x86 study, ARM results preamble (16 events per bar; `release-debug` replaced `profiling`).
[^x86-bench]: x86 study, bench changes.
[^x86-alg]: x86 study, algebraic float arithmetic protocol.
