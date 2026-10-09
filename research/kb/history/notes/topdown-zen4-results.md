---
type: Working Note
original_type: "Measurement"
title: "Top-down counters on Zen 4: where the evaluator's cycles go"
description: "Top-down PMU slot accounting of the evaluator on Zen 4 across lane widths, and the constant-folding and weighted-JAMP-sum changes it led to, A/B-timed on Emerald Rapids."
created: 2026-10-05
status: deprecated
tags: [performance, pmu, top-down, zen4, evaluator]
generated: {by: claude-code, at: 2026-10-05}
measured:
  - {commit: 5ced9bb, host: "AMD EPYC 9534 (Zen 4), bare metal, RHEL 9"}
  - {host: "Intel Xeon Emerald Rapids, 4-vCPU cloud VM"}
replaced_by: [performance/constant-collection-and-fused-sums, performance/topdown-zen4, performance/lane-throughput, performance/microbenchmark-protocol, performance/execution-order, amplitudes/per-diagram-amp2]
---
# Top-down counters on Zen 4: where the evaluator's cycles go

**Date:** 2026-10-05 · **Host:** AMD EPYC 9534 (Zen 4 "Genoa", family 25 model 17),
bare metal, RHEL 9 kernel 5.14, perf 5.14, governor `performance`, boost on,
pinned to CPU 255 · **Build:** `eval_loop` at `5ced9bb`, release,
`-C target-cpu=native`, feature `eval-schedule-study` · **Kit:**
`scripts/topdown_kit.sh` (5 s per pass, perf started disabled and switched on
around the timed loop only), summarised by `scripts/topdown_summary.py`.

This is the counter reading that `roofline-census-results.md` left open: on a host
with a PMU, how much of the evaluator's time is dispatch, how much is latency,
and how much is memory. The 8 bench rows ran at widths 1, 4 and 8 in the
production order. The controls were arena order (3 rows) and level-shuffled
order (the 2→6). Cycle profiles of the 2→6 at widths 1 and 4 were also taken.

Measurement caveats:
- The NMI watchdog holds one of Zen 4's six core counters, so every six-event
  pass multiplexed. Each event was counted 83% of the time, the FP-flop events
  33–50%. All runs are steady-state loops, so the scaled counts are usable.
  Repeat passes of a cell agree on ns/event to ±3%.
- The clock moved between 3.1 and 4.1 GHz across cells (boost), so this note
  compares cycles, not nanoseconds.
- Profiles sample `cycles` without IBS, so samples skid by a few instructions.
  Handler-level shares are reliable; individual instruction weights are only
  approximate.

## 1. Answers

| question | answer on Zen 4 |
|---|---|
| FLOP-bound? | No at widths 1 and 4: the four FP pipes do 1.2–2.2 passes per cycle. At width 8 they do 2.1–2.8 (the double-pumped 512-bit ops), the closest any resource gets to a limit. |
| Memory-bound? | No, except width 8 on large arenas. L1D demand misses are 0.0–1.7% of loads at width 1, and back-end-memory is 2–7% of slots on the larger rows. The 2→6 at width 8 misses 27% of loads (3.6 MiB of arenas against a 1 MiB L2), and back-end-memory reaches 21%. |
| Mispredicts? | Small on large rows, about 10% on small ones. Indirect mispredicts are 0.011–0.014 per VM instruction on the 2→6 (bad speculation 1–2%) and 0.05–0.11 on the small rows (bad speculation 4–8%, plus front-end refill). |
| Front end? | Not the limit: the op cache hits 97–100% of the time below width 8 (87–99% at width 8). Front-end-bound is 12–18% of slots on the larger rows, mostly bandwidth, not latency. |
| What is it, then? | At widths 1 and 4: **retiring 49–61% of slots**, so the run is bounded by the number of macro-ops: 58–64 per VM instruction on the 2→6 against 23 FP instructions in the census. The rest is mostly back-end-core stalls: in 37–48% of cycles nothing retires because the oldest op is still waiting on its operands. |

So the remaining headroom is instruction count first and latency second. FP,
bytes and branches are all below their ceilings. The cheapest instructions to
remove are identified in §3.

## 2. Slot accounting per cell

Columns:
- *ops*: retired macro-ops (`ex_ret_ops`).
- *FP passes*: FP uops, with a 512-bit uop counted twice, out of 4 pipes per cycle.
- *retiring … backend memory*: perf's `PipelineL1`/`PipelineL2` metrics, in % of dispatch slots.
- *oldest-op-incomplete*: `ex_no_retire.not_complete`, in % of cycles.

| cell | ns/event | cycles/VM instr | ops/VM instr | ops/cycle | FP passes/cycle | retiring | frontend | bad spec | backend core | backend memory | oldest-op-incomplete % cycles | L1D demand miss % | indirect misp/VM instr |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| `ee_to_mumu` w1 | 414 | 24.9 | 87 | 3.48 | 1.82 | 58 | 3 | 0 | 18 | 20 | 48 | 0.1 | 0.001 |
| `ee_to_mumu` w4 | 151 | 36.4 | 131 | 3.61 | 1.78 | 55 | 4 | 4 | 18 | 18 | 40 | 0.6 | 0.000 |
| `ee_to_mumu` w8 | 136 | 61.8 | 162 | 2.61 | 2.31 | 45 | 3 | 1 | 29 | 23 | 54 | 4.1 | 0.018 |
| `ee_to_wpwm` w1 | 1,927 | 16.7 | 53 | 3.17 | 1.57 | 49 | 29 | 5 | 12 | 4 | 44 | 0.6 | 0.093 |
| `ee_to_wpwm` w4 | 758 | 22.8 | 71 | 3.12 | 1.62 | 51 | 13 | 8 | 18 | 10 | 42 | 3.3 | 0.105 |
| `ee_to_wpwm` w8 | 546 | 36.3 | 81 | 2.23 | 2.32 | 36 | 8 | 6 | 32 | 18 | 53 | 11.9 | 0.102 |
| `uux_to_uux` w1 | 777 | 23.3 | 78 | 3.34 | 1.79 | 57 | 8 | 0 | 18 | 16 | 42 | 0.0 | 0.009 |
| `uux_to_uux` w4 | 252 | 28.7 | 104 | 3.63 | 1.81 | 61 | 7 | 1 | 17 | 15 | 40 | 1.3 | 0.002 |
| `uux_to_uux` w8 | 194 | 47.2 | 124 | 2.63 | 2.47 | 41 | 5 | 2 | 24 | 28 | 43 | 6.4 | 0.000 |
| `gg_to_gg` w1 | 2,527 | 13.7 | 47 | 3.46 | 1.24 | 58 | 24 | 6 | 8 | 3 | 37 | 0.5 | 0.068 |
| `gg_to_gg` w4 | 828 | 17.9 | 62 | 3.48 | 1.51 | 60 | 15 | 5 | 12 | 8 | 40 | 3.1 | 0.052 |
| `gg_to_gg` w8 | 618 | 29.4 | 68 | 2.33 | 2.12 | 42 | 7 | 7 | 26 | 19 | 48 | 13.1 | 0.076 |
| `gg_to_ttx` w1 | 1,603 | 17.9 | 57 | 3.20 | 1.56 | 55 | 17 | 6 | 16 | 7 | 41 | 0.4 | 0.074 |
| `gg_to_ttx` w4 | 484 | 21.5 | 70 | 3.27 | 1.49 | 55 | 17 | 4 | 15 | 9 | 41 | 2.0 | 0.066 |
| `gg_to_ttx` w8 | 381 | 31.2 | 79 | 2.54 | 2.31 | 43 | 12 | 4 | 24 | 17 | 50 | 5.7 | 0.064 |
| `ee_to_mumua` w1 | 1,949 | 20.4 | 67 | 3.30 | 2.08 | 53 | 18 | 2 | 22 | 6 | 43 | 0.1 | 0.067 |
| `ee_to_mumua` w4 | 612 | 26.7 | 82 | 3.09 | 1.71 | 53 | 14 | 7 | 18 | 9 | 40 | 2.6 | 0.113 |
| `ee_to_mumua` w8 | 453 | 39.2 | 92 | 2.34 | 2.58 | 37 | 9 | 7 | 30 | 18 | 55 | 7.3 | 0.105 |
| `ee_to_mumu_tata_qcd0` w1 | 8,707 | 18.5 | 61 | 3.29 | 2.12 | 54 | 17 | 4 | 22 | 3 | 42 | 0.7 | 0.050 |
| `ee_to_mumu_tata_qcd0` w4 | 2,367 | 20.5 | 68 | 3.33 | 1.84 | 56 | 14 | 6 | 18 | 6 | 39 | 5.0 | 0.059 |
| `ee_to_mumu_tata_qcd0` w8 | 1,814 | 29.9 | 73 | 2.45 | 2.78 | 41 | 7 | 5 | 34 | 12 | 50 | 17.6 | 0.078 |
| `uux_to_ccx_emmm_qcd0` w1 | 178,385 | 17.2 | 58 | 3.40 | 2.24 | 57 | 13 | 1 | 26 | 2 | 42 | 1.7 | 0.011 |
| `uux_to_ccx_emmm_qcd0` w4 | 46,957 | 18.3 | 64 | 3.47 | 1.93 | 56 | 12 | 2 | 20 | 10 | 42 | 11.0 | 0.014 |
| `uux_to_ccx_emmm_qcd0` w8 | 36,033 | 28.6 | 67 | 2.36 | 2.72 | 39 | 3 | 2 | 34 | 21 | 55 | 27.0 | 0.014 |
| `gg_to_gg` w1 arena | 3,292 | 15.0 | 47 | 3.17 | 1.14 | 44 | 11 | 3 | 19 | 24 | 75 | 0.3 | 0.043 |
| `gg_to_gg` w4 arena | 809 | 17.4 | 62 | 3.56 | 1.55 | 57 | 11 | 6 | 16 | 10 | 43 | 1.2 | 0.032 |
| `ee_to_mumu_tata_qcd0` w1 arena | 12,051 | 25.5 | 61 | 2.39 | 1.54 | 40 | 8 | 4 | 25 | 23 | 60 | 0.7 | 0.097 |
| `ee_to_mumu_tata_qcd0` w4 arena | 2,699 | 22.7 | 68 | 3.01 | 1.67 | 51 | 8 | 8 | 24 | 9 | 44 | 2.5 | 0.086 |
| `uux_to_ccx_emmm_qcd0` w1 arena | 242,952 | 24.9 | 59 | 2.35 | 1.54 | 40 | 8 | 4 | 26 | 22 | 58 | 2.1 | 0.091 |
| `uux_to_ccx_emmm_qcd0` w4 arena | 59,419 | 23.2 | 64 | 2.75 | 1.53 | 45 | 12 | 13 | 21 | 9 | 46 | 5.7 | 0.141 |
| `uux_to_ccx_emmm_qcd0` w1 levelshuffle | 272,478 | 27.0 | 59 | 2.18 | 1.42 | 36 | 28 | 19 | 12 | 5 | 54 | 5.6 | 0.396 |

Retired indirect branches are 1.00–1.14 per VM instruction in every cell. That is
one dispatch jump per instruction plus per-event fixed work on the small rows,
which confirms the per-instruction normalisation.

Readings:
- **Width 1 and 4 cost about the same per VM instruction.** On the larger rows,
  cycles per VM instruction rise only 6–31% from width 1 to width 4, for four
  times the arithmetic. The per-instruction cost is the instruction stream, not
  the FP work, as `roofline-census-results.md` §4 inferred from timings alone.
- **Zen 4 runs 3.1–3.6 macro-ops per cycle of a possible 6.** Front-end slots
  lost are 12–18%. The remainder is back-end-core: the oldest op is waiting on
  an operand in ~40% of cycles. Only 4–13% of cycles are waiting on a load, so
  this is FP dependency latency, not memory.
- **Width 8 is the best width on every row of this host.** It beats width 4 by
  1.18–1.38× in cycles, the 2→6 included (1.28×), even though Zen 4 runs a 512-bit op in two
  passes. The gain comes from fewer instructions per event, since FP issue
  stays the same. Width 8 is also where the run moves to the back end: FP pipe
  occupancy is 53–70%, and on large arenas L1D misses climb (the 2→6 reads 22%
  of its missed lines from beyond L2). Emerald Rapids found the opposite for
  the 2→6 (1.03×, `roofline-census-results.md`), so the best width depends on
  the host.
- **The production order is an ILP order.** Arena (producer-then-consumer)
  order costs 1.45× cycles on the 2→6, 1.38× on the 4-lepton row and 1.10× on
  `gg_to_gg` at width 1. The extra time is all retire stalls: the
  oldest-op-incomplete share goes from 42% to 58–75%, load waits from 4% to
  25%, and cache misses barely move. At width 4, arena order costs 0.98–1.27×.
- **Op-blocking buys predictability on long programs.** Shuffling the 2→6
  within its levels raises indirect mispredicts from 0.011 to 0.396 per VM
  instruction and costs 1.57× cycles. That is about 25 cycles per extra
  mispredict, with front-end-bound rising to 28% and bad speculation to 19%.
  This is the M3 Max result (`threaded-dispatch-study-results.md`), now
  reproduced on x86.

## 3. Inside `fill_arenas` on the 2→6

The cycle samples are 99% inside `fill_arenas`, which inlines every kernel.
Each sample was assigned to the handler arm whose jump-table target precedes
it, and divided by the handler's instance count in the program. That gives
cycles per call, where one call at width 4 serves four events:

| handler | count | % of VM instrs | w1 % cycles | w1 cycles/call | w4 % cycles | w4 cycles/call |
|---|--:|--:|--:|--:|--:|--:|
| `Metric` | 9 240 | 25.3 | 17.1 | 11.6 | 21.6 | 15.6 |
| `MulScalarR` | 13 932 | 38.2 | 12.0 | 5.4 | 15.4 | 7.4 |
| dispatch + loop (shared) | 36 506 | 100 | 11.6 | 2.0 | 11.0 | 2.0 |
| `GammaVout` | 2 048 | 5.6 | 14.6 | 44.9 | 8.3 | 27.0 |
| `FfvVout` | 2 048 | 5.6 | 10.3 | 31.7 | 8.2 | 26.7 |
| `PropagateFout` | 864 | 2.4 | 5.5 | 40.1 | 5.7 | 44.4 |
| `MulScalarC` | 4 716 | 12.9 | 5.9 | 7.8 | 5.6 | 8.0 |
| `AddScalar` (16 JAMP sums of 579 terms) | 16 | 0.0 | 4.1 | 1 607 | 5.3 | 2 212 |
| `PropagateFin` | 480 | 1.3 | 3.6 | 46.6 | 3.4 | 47.7 |

Instance counts come from a scratch census of `folded_hel().program()`.

`Metric` is close to ideal for an interpreter arm at width 4: 3 bounds checks,
16 loads, 20 packed FP ops (10 of them FMAs), 2 stores. Its cost is latency:
four dependent chains, each a multiply, two or three FMAs and an add. This is
the latency found in §2.

**The constant-coefficient chain.** A scratch census of what consumes each
value shows that every `MulScalarR` and `MulScalarC` in the 2→6 has exactly
one consumer. All 13 932 `MulScalarR` multiply by a `RealConst`, and 4 668 of
the 4 716 `MulScalarC` by a `ComplexConst`. They form amplitude chains of two
shapes (chain counts to within a few dozen):

```
Metric → MulScalarR → AddScalar                                    (4 596 chains)
Metric → MulScalarR → MulScalarC → MulScalarR → AddScalar          (4 668 chains)
```

By their operands these are presumably `amp × vertex coefficient × coupling ×
colour·symmetry factor`. The
existing constant folding collapses constant subtrees, but this product is
associated around the non-constant amplitude, so no constant subtree forms.
**51% of the 2→6's VM instructions (18 648 of 36 506) are these single-use
scalings.** Their arms plus their share of dispatch cost 24% of cycles at width
1 and 27% at width 4. The 4-lepton row has the same structure: 844 of its 1 739
instructions are such scalings, all single-use, with 616 of 616 real and 212 of
228 complex factors from the pool. On `gg_to_gg`, 294 of 682 instructions are
scalar multiplies, but their consumers were not checked.

Two levels of fix:
1. **Collect the constants.** Gather a single-use multiply chain's constant
   factors into one folded constant, giving one multiply per amplitude.
   Composite constants are already resolved at bind time, so an αs-dependent
   coupling still rescales correctly. This removes 9 384 instructions on the
   2→6, measured at ~13% of cycles at widths 1 and 4.
2. **Fuse the remaining multiply.** Put it into the producing `Metric`, or into
   the JAMP sum as a weighted `Σ kᵢ·ampᵢ`, so that no scaling instructions
   remain. That is ~24–27% of cycles minus a complex multiply per term. This is
   the form MadGraph emits: the coupling goes into the `VVV1_0`-style amplitude
   call, and `JAMP += coef·AMP`.

Both change floating-point association, a last-ulp effect at the scale
AGENTS.md says to tolerate. The figures are bounds from the measured per-arm
cost, and the change has not been built.

## 4. Width 1 under `target-cpu=native` on Zen 4

At width 1, the scalar `GammaVout` costs 45 cycles per call, more than the
width-4 arm's 27 cycles for four events. LLVM's SLP vectoriser turns the scalar
kernel into 512-bit code: 22 constant `zmm` loads, 14 `vpermi2pd`, 8 `vpermpd`
and masked `vsubpd`. Zen 4 executes all of these as two passes. The same effect
explains three readings:
- width 1 retires 18–38% of its FP uops as 512-bit uops;
- width 1 measures 1.06–1.45× the census flops (padding lanes are counted),
  while widths 4 and 8 measure 0.95–0.99×;
- width 1 runs `GammaVout` slower per call than width 4.

Only builds that set `target-cpu` see it; the default x86-64 target has no AVX.
Width 1 is not the production path when lanes are available, so this is a
note, not an action. If a width-1 path ever matters on AVX-512 hosts, compare
`-C target-cpu=native` against `-C target-feature=+avx2,+fma`.

## 5. What this changes

- The roofline note's open question has its answer: the fixed per-instruction
  cost is instruction count plus dependency latency. Dispatch mispredicts and
  memory are minor except where noted above.
- Ranked levers on this host:
  1. Collect and fuse the constant-coefficient chains (§3): about half the VM
     instructions on the 2→6 and the 4-lepton row, ~13% of cycles for collection alone,
     ~25% with fusion. Done; see §6.
  2. Choose the width per host: width 8 wins on every row on Zen 4 but
     barely on Emerald Rapids for the 2→6 (1.03×).
  3. More independent work per dispatch (per-kind batched execution of
     op-blocked runs) for the latency share. This is the lever the AOT studies
     pointed at, and it keeps code size flat.
- Bounds checks (3.5–5.5%) and mispredicts on large rows (1–2%) stay small, as
  measured before.

## 6. Implemented: constant collection and weighted JAMP sums

Both levels of §3 are now in `fold.rs`:
- **Collection.** `collect_constant_factors` runs before constant folding. It
  merges each single-reader product into the product that reads it and gathers
  the constant factors into one canonically ordered, hash-consed sub-product.
  The existing folding then turns that sub-product into one pool entry.
- **Fusion.** `fuse_scaled_sums` runs after folding. It turns a scalar `Add`
  whose terms are single-reader `k · x` products, with `k` a pool leaf, into
  the new `Op::AddScaled`. That op lowers to `Instr::AddScaled`, which reads
  each weight straight from `consts_f`/`consts_c` and accumulates with
  `mul_add_fast`: two multiply-adds per real-weighted term, four per complex
  one.
- **Pruning.** The zero-amplitude prune drops an `AddScaled` term together with
  its weight, under the same bit-exact guard as before.

Effect on the programs (helicity-pruned, production order):

| row | VM instructions before → after | arena KiB (f64) before → after |
|---|--:|--:|
| `ee_to_mumu` | 61 → 51 | 1.4 → 1.3 |
| `ee_to_wpwm` | 425 → 363 | 6.0 → 5.6 |
| `uux_to_uux` | 123 → 99 | 2.3 → 2.2 |
| `gg_to_gg` | 682 → 551 | 7.4 → 7.2 |
| `gg_to_ttx` | 328 → 253 | 3.6 → 3.5 |
| `ee_to_mumua` | 346 → 239 | 6.2 → 5.4 |
| `ee_to_mumu_tata_qcd0` | 1 739 → 1 096 | 22.3 → 18.4 |
| `uux_to_ccx_emmm_qcd0` | 36 506 → 21 815 | 450.9 → 288.2 |

On the 2→6 every `MulScalarR` is gone, along with the `ScaleFinR`,
`ScaleFoutR` and `ScaleVecR` halves of the current scalings. The 4 716
`MulScalarC` that remain compute the per-diagram amplitudes `A_d`. `Configs`
also reads `A_d` for `AMP2`, so it cannot be absorbed. Its JAMP term
`sym·fermi · A_d` is a real weight of the fused sum instead.

**Timing.** Measured in-process, on an Emerald Rapids cloud VM (4 vCPUs):
- A scratch driver links the previous commit's library (renamed) and this one.
- It builds both evaluators for a row and width, then alternates 100 ms slices
  (400 ms on the 2→6) of each on one pinned core, 40 slices apiece.
- The speedup is total base time over total new time. The bracket is the
  10th–90th percentile of the per-slice ratios.
- Separate-process runs on this VM spread by up to ±35%, which is what the
  in-process design removes.

| row | width | default target: speedup | `target-cpu=native`: base → new ns/event | native speedup [q10–q90] |
|---|--:|--:|--:|--:|
| `ee_to_mumu` | 1 / 4 / 8 | 1.03 / 0.94 / 1.01 | 384 → 369 / 164 → 146 / 111 → 107 | 1.04 / 1.12 / 1.04 [0.81–1.24] |
| `ee_to_wpwm` | 1 / 4 / 8 | 1.09 / 1.05 / 1.06 | 1 551 → 1 414 / 717 → 612 / 521 → 452 | 1.10 / 1.17 / 1.15 [0.98–1.38] |
| `uux_to_uux` | 1 / 4 / 8 | 1.00 / 1.03 / 1.03 | 633 → 568 / 268 → 233 / 177 → 159 | 1.11 / 1.15 / 1.12 [1.01–1.34] |
| `gg_to_gg` | 1 / 4 / 8 | 1.02 / 1.04 / 1.01 | 2 141 → 1 939 / 817 → 756 / 549 → 528 | 1.10 / 1.08 / 1.04 [0.89–1.21] |
| `gg_to_ttx` | 1 / 4 / 8 | 1.09 / 1.14 / 1.01 | 1 338 → 1 163 / 456 → 403 / 338 → 315 | 1.15 / 1.13 / 1.07 [0.87–1.38] |
| `ee_to_mumua` | 1 / 4 / 8 | 1.11 / 1.04 / 1.06 | 1 723 → 1 574 / 592 → 513 / 465 → 420 | 1.09 / 1.15 / 1.11 [1.00–1.23] |
| `ee_to_mumu_tata_qcd0` | 1 / 4 / 8 | 1.11 / 1.07 / 1.03 | 7 223 → 6 214 / 2 427 → 2 155 / 2 147 → 1 841 | 1.16 / 1.13 / 1.17 [1.05–1.48] |
| `uux_to_ccx_emmm_qcd0` | 1 / 4 / 8 | 1.21 / 1.20 / 1.22 | 143 584 → 127 373 / 61 857 → 46 622 / 62 013 → 44 405 | 1.13 / 1.33 / 1.40 [0.97–1.60] |

A second default-target sweep reproduced the 2→6 at 1.19 / 1.11 / 1.14 and the
other rows within about ±0.05.

Readings:
- **FMA hardware matters.** The default x86-64 target has no FMA, so there the
  weighted sum is a multiply and an add. With `target-cpu=native` it is one
  FMA per component, and the gain is larger on every row.
- **Width 8 no longer hits the L2 cliff on the 2→6.** The fused terms hold no
  arena slots, so its arenas shrink by 36% (2.3 MiB at 8 lanes, from 3.6 MiB).
  Width 8 now beats width 4 here (44.4 against 46.6 µs/event), where §2 and
  the roofline note found a 1.03× ceiling on Emerald Rapids.
- **Results are unchanged to within rounding.** The weights are products of
  the same constants, and a sum's terms are re-associated (real weights
  first). The 2→6 is bit-identical at width 1: its weights are all real
  `±1` symmetry factors. Elsewhere passes agree to 1e-12 relative or better.
  All 48 `amplitude_oracle` processes pass against MadGraph, including the
  bit-exact pruned-against-unpruned `|M|²` check.

## 7. Configuration amplitudes read bare, weighted at read-out

After §6 the 4 716 `MulScalarC` left on the 2→6 computed the per-diagram
amplitudes `A_d = (coupling·coeff) · Metric`. These were the values the
`Configs` bundle pins for `AMP2`, and they had two readers. `fold.rs` now
splits each bundle amplitude into a `(weight, value)` pair
(`pair_config_weights`):
- The bundle pins the bare value. The weight is a constant-pool leaf, or a
  unit coefficient when there is none.
- `Program::amp_weights` carries the weights alongside `amp_locs`.
- `eval_amp2` and `run_config_amps` multiply them back in from the bound pools.

The JAMP term is then the product's only reader, so a second collection pass
folds `sym·fermi · coupling · coeff` into its `AddScaled` weight.

| row | VM instructions §6 → §7 | scalar-arena slots §6 → §7 |
|---|--:|--:|
| `ee_to_mumu` | 51 → 49 | 12 → 13 |
| `ee_to_wpwm` | 363 → 333 | 64 → 66 |
| `uux_to_uux` | 99 → 95 | 25 → 27 |
| `gg_to_gg` | 551 → 547 | 118 → 118 |
| `gg_to_ttx` | 253 → 222 | 63 → 77 |
| `ee_to_mumua` | 239 → 209 | 72 → 74 |
| `ee_to_mumu_tata_qcd0` | 1 096 → 891 | 416 → 419 |
| `uux_to_ccx_emmm_qcd0` | 21 815 → 17 163 | 9 280 → 9 284 |

**The scalar arena does not shrink.** On the 2→6, every `Metric` is itself a
configuration amplitude's value, and the bundle pins all 9 240 of them to the end
of the pass either way. Before, half of them were consumed by the `MulScalarC`
that scaled them, but in the level-ordered schedule all `Metric`s are live
together at their level. That level width already sets the peak, so pinning
the bare value in place of its scaled copy leaves it unchanged. On `gg_to_ttx`
the peak grows, since values that the scaling used to release now stay pinned.
Shrinking this arena needs `AMP2` accumulated inside the pass, so the
amplitudes need not outlive their JAMP sum, together with an order that
retires a level's amplitudes before the next level fills.

Timing, in-process A/B on the Emerald Rapids VM with `target-cpu=native`
(method as in §6, ratios with their 10th–90th percentile slice spread):

| row | width | vs §6 (38c2410) | vs before §6 (5a5e377) |
|---|--:|--:|--:|
| `ee_to_mumu` | 1 / 4 / 8 | 1.04 / 1.06 / 1.00 | 1.02 / 1.10 / 0.98 |
| `ee_to_wpwm` | 1 / 4 / 8 | 1.08 / 1.02 / 1.07 | 1.07 / 1.13 / 1.14 |
| `uux_to_uux` | 1 / 4 / 8 | 1.05 / 1.03 / 1.01 | 1.09 / 1.08 / 1.10 |
| `gg_to_gg` | 1 / 4 / 8 | 1.02 / 0.91 / 1.00 | 1.01 / 1.15 / 1.05 |
| `gg_to_ttx` | 1 / 4 / 8 | 1.07 / 1.01 / 1.06 | 1.16 / 1.19 / 1.20 |
| `ee_to_mumua` | 1 / 4 / 8 | 1.07 / 1.06 / 1.06 | 1.15 / 1.12 / 1.09 |
| `ee_to_mumu_tata_qcd0` | 1 / 4 / 8 | 1.01 / 1.04 / 1.05 | 1.14 / 1.18 / 1.19 |
| `uux_to_ccx_emmm_qcd0` | 1 / 4 / 8 | 1.01 / 1.11 / 1.04 | 1.18 / 1.35 / 1.55 [1.30–1.82] |

The VM was noisier for this run than for §6: most slice spreads are ±15–20%, so
the step over §6 is resolved only as "a few percent", consistent with removing
21% of the 2→6's instructions, the cheapest ones. The cumulative column is the
summary: 1.18–1.55× on the 2→6 and 1.01–1.20× elsewhere. `AMP2` and the
per-diagram amplitudes still match MadGraph in `amplitude_oracle`.

## 8. Real constant products fold into the real pool

The analysis types every constant product as a complex scalar (`mul_out`), so
`fold_constant_subgraphs` put even an all-real product, such as a colour
coefficient times a symmetry factor, in the complex pool. Its readers then paid
for a complex scale with a zero imaginary part.

A fold root that is a `Mul` tree over real pool leaves now goes to the real
pool. The exception is a root read by an op that takes its constant operand
from the scalar arena only, such as a scalar `Add` or a fused vertex's `g_L`/`g_R`.
Such a root stays complex, so only `Mul` and `Configs` weights see a real fold.
The pool value is the real part of the complex product it was before, whose
imaginary part is exactly zero.

| row | `AddScaled` real / complex weights, before → after |
|---|--:|
| `uux_to_uux` | 0 / 16 → 16 / 0 |
| `gg_to_gg` | 88 / 56 → 144 / 0 |
| every other bench row | unchanged |

Only the multi-flow rows move, because elsewhere the remaining complex weights
all include a coupling. The saving is two multiply-adds per affected term,
an estimated 0.4% of `gg_to_gg`'s ops (112 of about 27 000), far below what this VM can time. It was not
measured.

