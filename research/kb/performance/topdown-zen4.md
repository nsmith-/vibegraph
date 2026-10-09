---
type: Measurement
title: "Top-down counters on Zen 4"
description: "PMU slot accounting at widths 1/4/8 on EPYC 9534: the evaluator is bounded by macro-op count and FP dependency latency, not FLOPs, memory or branches; width 1 under native gets SLP-vectorised."
status: draft
tags: [performance, pmu, top-down, zen4, evaluator]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
measured:
  commit: 5ced9bb
  host: "AMD EPYC 9534 (Zen 4 Genoa, family 25 model 17), bare metal, RHEL 9 kernel 5.14, governor performance, boost on, pinned to CPU 255"
  command: "scripts/topdown_kit.sh (eval_loop example, release, -C target-cpu=native, feature eval-schedule-study), summarised by scripts/topdown_summary.py"
sources:
  - {id: td, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/topdown-zen4-results.md#L13-L225", title: "Top-down counters on Zen 4, §0–§5"}
---

# Top-down counters on Zen 4

The PMU reading the [roofline census](roofline-census.md) left open: how much of the
evaluator's time is dispatch, latency and memory. The 8 `eval_strategies` rows ran in the
`eval_loop` example at widths 1, 4 and 8 in the production (op-blocked) order, with arena
order (3 rows) and a level-shuffled order (the 2→6) as controls, plus cycle profiles of the
2→6 at widths 1 and 4.[^td]

**Answer.** At widths 1 and 4 the run retires 49–61% of dispatch slots, so it is bounded by
the number of macro-ops (58–64 per VM instruction on the 2→6, against 23 FP instructions in
the census). Most of the rest is back-end-core: in 37–48% of cycles nothing retires because
the oldest op waits on its operands, which is FP dependency latency. FP pipes, bytes and
branches are all below their ceilings.

| question | answer on Zen 4 |
|---|---|
| FLOP-bound? | No at widths 1 and 4 (1.2–2.2 FP passes per cycle of 4 pipes). Width 8: 2.1–2.8, the closest any resource gets. |
| Memory-bound? | No, except width 8 on large arenas. L1D demand misses 0.0–1.7% of loads at width 1; the 2→6 at width 8 misses 27% (3.6 MiB of arenas against a 1 MiB L2), back-end-memory 21%. |
| Mispredicts? | Small on large rows: 0.011–0.014 indirect mispredicts per VM instruction on the 2→6 (bad speculation 1–2%); 0.05–0.11 on small rows (4–8%, plus front-end refill). |
| Front end? | Not the limit: op cache hits 97–100% below width 8 (87–99% at 8); front-end-bound 12–18% on larger rows, mostly bandwidth. |

Retired indirect branches are 1.00–1.14 per VM instruction in every cell, one dispatch jump
per instruction, which confirms the per-instruction normalisation.

## Measurement caveats

- The NMI watchdog holds one of Zen 4's six core counters, so every six-event pass
  multiplexed: each event counted 83% of the time, FP-flop events 33–50%. Steady-state
  loops make the scaled counts usable; repeat passes agree on ns/event to ±3%.
- The clock moved between 3.1 and 4.1 GHz across cells (boost), so compare cycles, not ns.
- Cycle samples without IBS skid by a few instructions: handler-level shares are reliable,
  single-instruction weights approximate.
- The programs are those of `5ced9bb`, before [constant collection](constant-collection-and-fused-sums.md),
  which this measurement motivated and which removed about half the 2→6's instructions.

## Slot accounting, selected cells

Full table in the source (31 cells). Production order unless marked:

| cell | ns/event | cycles/VM instr | ops/VM instr | retiring % | back-end core % | back-end memory % | oldest-op-incomplete % cycles | L1D miss % |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| `gg_to_gg` w1 | 2 527 | 13.7 | 47 | 58 | 8 | 3 | 37 | 0.5 |
| `gg_to_gg` w4 | 828 | 17.9 | 62 | 60 | 12 | 8 | 40 | 3.1 |
| `gg_to_gg` w8 | 618 | 29.4 | 68 | 42 | 26 | 19 | 48 | 13.1 |
| `ee_to_mumu_tata_qcd0` w1 | 8 707 | 18.5 | 61 | 54 | 22 | 3 | 42 | 0.7 |
| `ee_to_mumu_tata_qcd0` w4 | 2 367 | 20.5 | 68 | 56 | 18 | 6 | 39 | 5.0 |
| `ee_to_mumu_tata_qcd0` w8 | 1 814 | 29.9 | 73 | 41 | 34 | 12 | 50 | 17.6 |
| 2→6 w1 | 178 385 | 17.2 | 58 | 57 | 26 | 2 | 42 | 1.7 |
| 2→6 w4 | 46 957 | 18.3 | 64 | 56 | 20 | 10 | 42 | 11.0 |
| 2→6 w8 | 36 033 | 28.6 | 67 | 39 | 34 | 21 | 55 | 27.0 |
| 2→6 w1 arena order | 242 952 | 24.9 | 59 | 40 | 26 | 22 | 58 | 2.1 |
| 2→6 w1 level-shuffled | 272 478 | 27.0 | 59 | 36 | 12 | 5 | 54 | 5.6 |

- **Widths 1 and 4 cost about the same per VM instruction.** On the larger rows cycles per
  VM instruction rise only 6–31% from width 1 to 4, for four times the arithmetic. The
  per-instruction cost is the instruction stream, not the FP work, as the roofline census
  inferred from timings alone.
- **3.1–3.6 macro-ops per cycle of a possible 6.** Only 4–13% of cycles wait on a load, so
  the back-end stall is latency, not memory.
- **Width 8 is the best width on every row of this host**, 1.18–1.38× fewer cycles than
  width 4 (the 2→6 1.28×), although Zen 4 runs a 512-bit op in two passes. The gain is
  fewer instructions per event. Width 8 is also where the run moves to the back end: FP
  pipe occupancy 53–70%, and on large arenas L1D misses climb. The best width is
  host-dependent; see [lane throughput](lane-throughput.md).
- **The production order is an ILP order.** Arena (producer-then-consumer) order costs
  1.45× cycles on the 2→6, 1.38× on the 4-lepton row and 1.10× on `gg_to_gg` at width 1,
  all in retire stalls (oldest-op-incomplete 42% → 58–75%, load waits 4% → 25%) with cache
  misses barely moving; at width 4 it costs 0.98–1.27×.
- **Op-blocking buys predictability on long programs.** Shuffling the 2→6 within its levels
  raises indirect mispredicts from 0.011 to 0.396 per VM instruction and costs 1.57×
  cycles, about 25 cycles per extra mispredict. This reproduces the M3 Max result on x86.
  See [execution order](execution-order.md).

## Inside `fill_arenas` on the 2→6

99% of cycle samples are inside `fill_arenas`, which inlines every kernel. Samples were
assigned to the handler whose jump-table target precedes them and divided by its instance
count:

| handler | % of VM instrs | w1 % cycles | w1 cycles/call | w4 % cycles | w4 cycles/call |
|---|--:|--:|--:|--:|--:|
| `Metric` | 25.3 | 17.1 | 11.6 | 21.6 | 15.6 |
| `MulScalarR` | 38.2 | 12.0 | 5.4 | 15.4 | 7.4 |
| dispatch + loop (shared) | 100 | 11.6 | 2.0 | 11.0 | 2.0 |
| `GammaVout` | 5.6 | 14.6 | 44.9 | 8.3 | 27.0 |
| `FfvVout` | 5.6 | 10.3 | 31.7 | 8.2 | 26.7 |
| `PropagateFout` | 2.4 | 5.5 | 40.1 | 5.7 | 44.4 |
| `MulScalarC` | 12.9 | 5.9 | 7.8 | 5.6 | 8.0 |

`Metric` at width 4 is close to ideal for an interpreter arm (3 bounds checks, 16 loads,
20 packed FP ops of which 10 are FMAs, 2 stores); its cost is four dependent chains.

**The constant-coefficient chains.** Every `MulScalarR` and `MulScalarC` in the 2→6 at this
commit had exactly one consumer and multiplied by a pool constant, in chains
`Metric → MulScalarR → AddScalar` and
`Metric → MulScalarR → MulScalarC → MulScalarR → AddScalar`: 51% of the VM instructions
(18 648 of 36 506), 24–27% of cycles. Constant folding could not see them because the
product is associated around the non-constant amplitude. The fix this pointed to, now in
`fold.rs`, is described in [constant collection](constant-collection-and-fused-sums.md).

## Width 1 under `target-cpu=native`

At width 1 the scalar `GammaVout` costs 45 cycles per call, more than the width-4 arm's
27 cycles for four events: LLVM's SLP vectoriser turns the scalar kernel into 512-bit code
(22 constant `zmm` loads, 14 `vpermi2pd`, 8 `vpermpd`, masked `vsubpd`), all two-pass on
Zen 4. Width 1 retires 18–38% of its FP uops as 512-bit, and measures 1.06–1.45× the census
flops (padding lanes counted) against 0.95–0.99× at widths 4 and 8. Only builds that set
`target-cpu` see it; the default x86-64 target has no AVX. If a width-1 path ever matters on
AVX-512 hosts, compare `-C target-cpu=native` with `-C target-feature=+avx2,+fma`.

## What it ranked

On this host: (1) collect and fuse the constant chains (done); (2) choose the lane width per
host; (3) more independent work per dispatch, such as per-kind batched execution of
op-blocked runs, for the latency share, which keeps code size flat where
[ahead-of-time compilation](aot-compilation-study.md) does not. Bounds checks (3.5–5.5%) and
large-row mispredicts (1–2%) stay small. How to run the kit is in
[benchmarking the evaluator](microbenchmark-protocol.md).

[^td]: `topdown-zen4-results.md` §0–§5, at `5ced9bb`.
