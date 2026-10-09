---
type: Measurement
title: "Roofline census of the helicity evaluator"
description: "Exact op and byte counts against measured time on Emerald Rapids: scalar is FP-issue limited (51–68% of the mix ceiling), lanes have headroom, the 2→6 at lanes8 was L2-capacity bound."
status: draft
tags: [performance, roofline, evaluator, simd, memory-bandwidth]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
measured:
  commit: db5fd03
  host: "Intel Xeon Emerald Rapids (family 6 model 207), 4-vCPU Firecracker VM, 48 KiB L1d, 2 MiB L2"
  command: "RUSTFLAGS='-C target-cpu=native' cargo test -p vibegraph-lib --lib roofline -- --include-ignored --nocapture; cargo bench --bench eval_strategies"
sources:
  - {id: rc, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/roofline-census-results.md#L11-L272", title: "Roofline census results"}
  - {id: aot5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/aot-kernels-study-results.md#L194-L226", title: "AOT study §5 (the census ceiling against compiled arithmetic)"}
  - {id: td6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/topdown-zen4-results.md#L226-L298", title: "Top-down Zen 4 §6 (constant collection; width 8 on Emerald Rapids afterwards)"}
---

# Roofline census of the helicity evaluator

The question: is the evaluator limited by floating-point throughput or by moving values
between caches and registers? The census answers it without a PMU, by counting the work
exactly and dividing by measured time.[^rc]

**Answer, at `db5fd03`.** Not bytes. Arena traffic runs at 5–9 B/cycle at scalar width,
under a tenth of L1 bandwidth. Only 14–26% of FP operations are FMAs, so peak FLOPs is the
wrong ceiling. Against the FP-issue ceiling of the actual operation mix, large scalar rows
run at 51–68% and small ones at 37–53%. Lane paths sit at 25–52% of theirs. The one
memory-bound cell was the 2→6 at lanes8, whose 3.6 MiB working set overflowed the 2 MiB L2.

## Method

- **Arithmetic is counted, not modelled.** `roofline_census`
  (`vibegraph-lib/src/helas/eval/roofline.rs`, an ignored test) runs `eval_m2` at
  `F = Counted`, an `f64` newtype that tallies every operation: the production code path
  at another field. Every counted `|M|²` equals the `f64` one bit for bit (asserted per
  point). A dozen operations per event are the debug-build partonic-CM input check.
- **Traffic is counted from the compiled program**: each instruction's `arena_reads`
  operands plus the pool and momentum entries it indexes, and one written element. It
  counts values, not machine loads: the instruction stream, `Vec` headers, spills and
  by-value kernel copies are not in it, so the real L1 load rate is higher.
- An FMA is two flops. Negations are counted apart and excluded (17% of FP ops on the
  2→6, often folded into an FMA variant). The scalar build SLP-packs some complex pairs
  into xmm, so its FP-instruction count is an upper bound.
- **Clock: measured 3.2 GHz** (a pinned chain of dependent integer adds); the VM reports
  only its 2.1 GHz base. AVX-512 licence downclock under zmm would make lanes8 per-cycle
  figures somewhat higher than shown.
- Peaks are from the Golden Cove / Raptor Cove microarchitecture, not measured: two
  FMA-capable ports at every width (adds also on a third at scalar and ymm), so 4 / 16 /
  32 flops per cycle at scalar / ymm / zmm; L1 96–128 B/cycle of loads plus two stores;
  L2 64 B/cycle.

## The census (host-independent counts)

| process | VM instrs | flops/event | FMA share | arena B/event | flop/B | arena KiB (f64) |
|---|--:|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 61 | 1 629 | 16% | 5 904 | 0.28 | 1.4 |
| `ee_to_wpwm` | 425 | 9 372 | 15% | 53 256 | 0.18 | 6.0 |
| `uux_to_uux` | 123 | 2 662 | 18% | 12 032 | 0.22 | 2.3 |
| `gg_to_gg` | 682 | 10 186 | 14% | 66 832 | 0.15 | 7.4 |
| `gg_to_ttx` | 328 | 6 426 | 23% | 27 792 | 0.23 | 3.6 |
| `ee_to_mumua` | 346 | 10 915 | 23% | 38 224 | 0.29 | 6.2 |
| `ee_to_mumu_tata_qcd0` | 1 739 | 51 587 | 25% | 184 096 | 0.28 | 22.3 |
| `uux_to_ccx_emmm_qcd0` | 36 506 | 1 060 891 | 26% | 3 887 728 | 0.27 | 450.9 |

These are the programs before constant collection. That change and the bare configuration
amplitudes after it removed 20–53% of the VM instructions on these rows (the 2→6 to
17 163) and shrank the 2→6's arenas to 288 KiB at
`f64`, so instruction counts, arena sizes and achieved rates here describe `db5fd03`, not
the current tree. See [constant collection](constant-collection-and-fused-sums.md).

## Achieved rates and the FP-issue ceiling

Per-event time comes from `eval_strategies` criterion medians, one run, intervals ±5–10%
per cell, scalar rows also carrying the up-to-22% memory-layout confound of
[threaded dispatch](threaded-dispatch-study.md) §6. Every reading rests on factor-level
differences.

Fraction of the operation-mix ceiling reached (scalar and ymm: `max((mul+fma)/2, all/3)`
cycles per event; zmm: `all/2/8`; each range from without to with negations as port
work, read to about ±10%):

| process | scalar ns/event | scalar | lanes4 | lanes8 |
|---|--:|--:|--:|--:|
| `ee_to_mumu` | 385 | 38–44% | 26–30% | 27–32% |
| `ee_to_wpwm` | 1 803 | 47–53% | 32–36% | 33–37% |
| `uux_to_uux` | 639 | 37–44% | 27–32% | 25–30% |
| `gg_to_gg` | 2 482 | 37–42% | 26–29% | 28–31% |
| `gg_to_ttx` | 1 415 | 39–46% | 30–35% | 32–38% |
| `ee_to_mumua` | 1 678 | 55–66% | 40–48% | 38–45% |
| `ee_to_mumu_tata_qcd0` | 7 562 | 57–68% | 43–52% | 40–49% |
| `uux_to_ccx_emmm_qcd0` | 173 844 | 51–61% | 34–41% | 26–32% |

- **Scalar FP issue is busy, not saturated**: 1.1–1.7 FP instructions per cycle against
  2–3 FP-capable ports. A perfect schedule of the same operations would be at most about
  1.5–2× faster.
- **Lanes use the FP units less**: 0.5–0.8 vector FP instructions per cycle at lanes8, 25–40%
  of two zmm ports.
- **Bytes are far from the L1 limit**: at most 35 B/cycle (`ee_to_wpwm` lanes8) against
  96–128 B/cycle of loads. Arithmetic intensity is 0.15–0.29 flop/B on every row; machine
  balance is about 0.17 flop/B at L1 and 0.5 at L2, so an evaluator stripped of interpreter
  overhead would be FP-bound while its working set fits L1 and bandwidth-bound once it
  spills to L2.
- **The 2→6 had a capacity cliff at this commit**: 1.79× from width 1 to 2, 1.50× from 2 to 4,
  1.03× from 4 to 8 (other rows 1.2–1.4×). Its arenas were 1.8 MiB at lanes4 and 3.6 MiB
  at lanes8 against a 2 MiB L2. This is a property of the pre-collection program, not of
  the host: after constant collection the arenas are 2.3 MiB at lanes8 and width 8 beats
  width 4 on the same host (44.4 against 46.6 µs/event).[^td6] See
  [lane throughput](lane-throughput.md).

## Width-independent versus per-lane cost

Every width executes the same VM instructions, so cycles per pass-instruction
`T(N) = time × N / instrs` split into a part independent of `N` and a slope. Fitting widths
1, 2 and 4 (xmm and ymm share ports), **83–90% of a scalar VM instruction's 10–18 cycles
is width-independent**, and each lane adds 1.3–3.0 cycles. That fixed bucket holds
dispatch, operand indexing, bounds checks, call and copy glue, each kernel's dependent FP
chain, and FP issue itself (one ymm FMA takes the slot of one scalar FMA), so
width-independence cannot by itself separate dispatch from arithmetic. The per-lane slope
is 8–16 element FP operations per marginal cycle, at or past what two to three ymm ports
issue, and 32–73 B: FP throughput is the first width-proportional limit.

## What later measurements settled

- **The scalar gap to the ceiling is mostly interpreter overhead.** The census attributed
  the gap to "dependency latency" by elimination (mispredicts ~2% and bounds checks
  3.5–5.5% being small). The ahead-of-time study then ran the same arithmetic without the
  interpreter and reached about 75–90% of this ceiling on `gg_to_gg` and
  `ee_to_mumu_tata_qcd0`.[^aot5] See [interpreter overhead](interpreter-overhead-budget.md).
- **The latency/issue/front-end split** this note could not make came from PMU counters on
  Zen 4: retiring 49–61% of slots, bounded by macro-op count, then FP dependency latency.
  See [top-down on Zen 4](topdown-zen4.md).
- **Not settled**: per-row machine-load counts, L1 hit versus L2 traffic (only the 2→6's
  stalled scaling shows a capacity effect directly), and rates on other hosts. The census
  counts do not depend on the host; the rates do, and the M3 Max and Cascade Lake have
  different port counts and L2 sizes ([benchmark hosts](benchmark-hosts.md)).

[^rc]: `roofline-census-results.md`, all sections, at `db5fd03`.
[^td6]: Top-down Zen 4 §6, in-process A/B on an Emerald Rapids VM, `target-cpu=native`.
[^aot5]: AOT study §5, at `03c31e6`.
