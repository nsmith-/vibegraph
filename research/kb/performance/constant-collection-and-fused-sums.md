---
type: Design
title: Constant collection and fused scaled sums
description: "collect_constant_factors, fuse_scaled_sums (Op::AddScaled), pair_config_weights and real-pool folding remove single-use constant scalings; measured on Zen 4."
status: draft
tags: [performance, evaluator, constant-folding, fma, fold]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: td-fill, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/topdown-zen4-results.md#L128-L191", title: "Top-down Zen 4 §3 (inside fill_arenas on the 2→6)"}
  - {id: td-changes, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/topdown-zen4-results.md#L209-L225", title: "Top-down Zen 4 §5 (ranked levers)"}
  - {id: td-impl, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/topdown-zen4-results.md#L226-L298", title: "Top-down Zen 4 §6 (constant collection and weighted JAMP sums)"}
  - {id: td-bare, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/topdown-zen4-results.md#L299-L355", title: "Top-down Zen 4 §7 (configuration amplitudes read bare)"}
  - {id: td-real, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/topdown-zen4-results.md#L356-L381", title: "Top-down Zen 4 §8 (real constant products to the real pool)"}
  - {id: fold-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/fold.rs#L215-L250", title: "Folded::build pass order"}
measured:
  - {commit: 5ced9bb, pr: 17, landed_in: aeb96a7, host: "AMD EPYC 9534 (Zen 4), bare metal, RHEL 9", command: "scripts/topdown_kit.sh"}
  - {pr: 17, landed_in: aeb96a7, host: "Intel Xeon Emerald Rapids, 4-vCPU cloud VM", command: "in-process A/B driver, 100 ms slices (400 ms on the 2→6), 40 per arm, pinned core"}
---

# Constant collection and fused scaled sums

Before these passes, about half of a large program's VM instructions were
single-use multiplications by constants. They now collapse into one weight per
term of a fused weighted sum, `Op::AddScaled`, read straight from the constant
pools. This is the form MadGraph emits: the coupling goes into the amplitude
call, and `JAMP += coef·AMP`.

## The finding that motivated it

Top-down counters on Zen 4 ([topdown-zen4](../performance/topdown-zen4.md))
showed the evaluator bounded by macro-op count and dependency latency, not
FLOPs. A census of the 2→6's program found every `MulScalarR` and nearly every
`MulScalarC` had exactly one consumer and multiplied by a pool constant, in
amplitude chains of two shapes:[^td-fill]

```
Metric → MulScalarR → AddScalar                                    (4 596 chains)
Metric → MulScalarR → MulScalarC → MulScalarR → AddScalar          (4 668 chains)
```

That is `amp × vertex coefficient × coupling × colour·symmetry factor`,
associated around the non-constant amplitude, so ordinary constant folding never
saw a constant subtree. **51% of the 2→6's VM instructions (18 648 of 36 506)
were these scalings**, costing 24–27% of cycles; the 4-lepton row had the same
structure (844 of 1 739).

## The passes, in `Folded::build` order

1. **`collect_constant_factors`** (before folding) merges each single-reader
   product into the product that reads it and gathers its constant factors into
   one canonically ordered, hash-consed sub-product.[^td-impl][^fold-rs]
2. **`pair_config_weights`** splits each configuration-bundle amplitude into a
   `(weight, value)` pair. The `Configs` bundle pins the bare value; the weight is
   a constant-pool leaf (or a unit coefficient). `Program::amp_weights` carries
   the weights beside `amp_locs`, and `eval_amp2` and `run_config_amps` multiply
   them back in from the bound pools. The JAMP term is then the product's only
   reader, so a second collection pass folds `sym·fermi · coupling · coeff` into
   one weight.[^td-bare] Per-diagram AMP2 semantics are in
   [per-diagram AMP2](../amplitudes/per-diagram-amp2.md).
3. **Constant folding** (`fold_constant_subgraphs`) turns each gathered
   sub-product into one pool entry. Composite constants are resolved at bind
   time, so an αs-dependent coupling still rescales correctly. A fold root that is
   a `Mul` tree over real pool leaves goes to the **real** pool (the real part of
   the complex product, whose imaginary part is exactly zero), unless it is read by
   an op that takes its constant operand from the scalar arena only (a scalar
   `Add`, a fused vertex's `g_L`/`g_R`); those stay complex.[^td-real]
4. **`fuse_scaled_sums`** (after folding) turns a scalar `Add` whose terms are
   single-reader `k · x` products, `k` a pool leaf, into `Op::AddScaled`. It
   lowers to `Instr::AddScaled`, which reads each weight from `consts_f` or
   `consts_c` and accumulates with `mul_add_fast`: two multiply-adds per
   real-weighted term, four per complex one.[^td-impl]

The zero-amplitude prune (see
[helicity expansion](../performance/helicity-expansion.md)) drops an `AddScaled`
term together with its weight, under the same bit-exact guard.

## Effect on the programs

Helicity-pruned, production order:[^td-impl][^td-bare]

| row | VM instructions: before → after collection/fusion → bare configs | arena KiB (`f64`): before → after collection/fusion |
|---|--:|--:|
| `ee_to_mumu` | 61 → 51 → 49 | 1.4 → 1.3 |
| `ee_to_wpwm` | 425 → 363 → 333 | 6.0 → 5.6 |
| `uux_to_uux` | 123 → 99 → 95 | 2.3 → 2.2 |
| `gg_to_gg` | 682 → 551 → 547 | 7.4 → 7.2 |
| `gg_to_ttx` | 328 → 253 → 222 | 3.6 → 3.5 |
| `ee_to_mumua` | 346 → 239 → 209 | 6.2 → 5.4 |
| `ee_to_mumu_tata_qcd0` | 1 739 → 1 096 → 891 | 22.3 → 18.4 |
| `uux_to_ccx_emmm_qcd0` | 36 506 → 21 815 → 17 163 | 450.9 → 288.2 |

On the 2→6 every `MulScalarR` is gone, with the real halves of the current
scalings (`ScaleFinR`, `ScaleFoutR`, `ScaleVecR`).

**The scalar arena does not shrink from reading configurations bare.** Every
`Metric` on the 2→6 is a configuration amplitude, and the bundle pins all 9 240
of them to the end of the pass; in the level-ordered schedule they are live
together at their level, which already sets the peak (scalar slots 9 280 →
9 284). On `gg_to_ttx` the peak grows (63 → 77). Shrinking it needs `AMP2`
accumulated inside the pass plus an order that retires a level's amplitudes
before the next fills: open as
[scalar-arena-holds-all-amplitudes-live](../backlog/performance/scalar-arena-holds-all-amplitudes-live.md).[^td-bare]

Real-pool folding moves only the multi-flow rows (`uux_to_uux` `AddScaled`
weights 0 real / 16 complex → 16 / 0; `gg_to_gg` 88 / 56 → 144 / 0), an estimated
0.4% of `gg_to_gg`'s ops, too small to time on the VM and not measured.[^td-real]

## Timing

In-process A/B on the Emerald Rapids VM: a driver links the previous commit's
library (renamed) and the new one, builds both evaluators for a row and width,
and alternates 100 ms slices (400 ms on the 2→6) on one pinned core, 40 apiece.
Speedup is total base time over total new time; the bracket is the 10th–90th
percentile of per-slice ratios. Separate-process runs on this VM spread up to
±35%, which the in-process design removes.[^td-impl]

Collection plus fusion, `target-cpu=native` (default-target speedups in the
last column):

| row | native speedup w1 / w4 / w8 | native ns/event w8 base → new | default target w1 / w4 / w8 |
|---|--:|--:|--:|
| `ee_to_mumu` | 1.04 / 1.12 / 1.04 | 111 → 107 | 1.03 / 0.94 / 1.01 |
| `gg_to_gg` | 1.10 / 1.08 / 1.04 | 549 → 528 | 1.02 / 1.04 / 1.01 |
| `gg_to_ttx` | 1.15 / 1.13 / 1.07 | 338 → 315 | 1.09 / 1.14 / 1.01 |
| `ee_to_mumu_tata_qcd0` | 1.16 / 1.13 / 1.17 | 2 147 → 1 841 | 1.11 / 1.07 / 1.03 |
| `uux_to_ccx_emmm_qcd0` | 1.13 / 1.33 / 1.40 | 62 013 → 44 405 | 1.21 / 1.20 / 1.22 |

Cumulative with bare configuration amplitudes, against the tree before both
(`5a5e377`): **1.18 / 1.35 / 1.55× on the 2→6** (w1 / w4 / w8; w8 slice spread
1.30–1.82) and 1.01–1.20× elsewhere. The step from bare configurations alone
(against `38c2410`) is resolved only as "a few percent" on a noisier VM run
(±15–20% slice spreads), consistent with removing 21% of the 2→6's
instructions, the cheapest ones.[^td-bare]

- **FMA hardware matters.** Default x86-64 has no FMA, so a weighted term is a
  multiply and an add; with `target-cpu=native` it is one FMA per component, and
  the gain is larger on every row.
- **Width 8 now beats width 4 on the 2→6 on Emerald Rapids** (44.4 against
  46.6 µs/event). The fused terms hold no arena slots, so the 2→6's arenas shrank
  36% (2.3 MiB at eight lanes, from 3.6 MiB), below the 2 MiB-L2 cliff that left
  width 8 only 1.03× ahead of width 4 in the
  [roofline census](../performance/roofline-census.md); that census figure is
  pre-fusion. Per-host lane results are in
  [lane throughput](../performance/lane-throughput.md).

## Exactness

The weights are products of the same constants, and a sum's terms are
re-associated (real weights first), a last-ulp change at the scale AGENTS.md
tolerates. The 2→6 is bit-identical at width 1 (all its weights are real ±1
symmetry factors); elsewhere passes agree to 1e-12 relative or better. All 48
`amplitude_oracle` processes pass against MadGraph, including the bit-exact
pruned-against-unpruned |M|² check, and `AMP2` and the per-diagram amplitudes
still match MadGraph.[^td-impl][^td-bare]

## What is left

Ranked by the same counter study: choosing the lane width per host, then more
independent work per dispatch (per-kind batched execution of op-blocked runs)
for the latency share, the lever the
[AOT study](../performance/aot-compilation-study.md) also pointed at. Bounds
checks (3.5–5.5%) and mispredicts on large rows (1–2%) stay
small.[^td-changes]

[^td-fill]: Top-down Zen 4 §3: per-handler cycles and the constant-coefficient chain census.
[^td-changes]: Top-down Zen 4 §5: ranked levers.
[^td-impl]: Top-down Zen 4 §6: the implemented collection and fusion, program sizes, A/B timing, exactness.
[^td-bare]: Top-down Zen 4 §7: configuration amplitudes read bare, weighted at read-out.
[^td-real]: Top-down Zen 4 §8: real constant products fold into the real pool.
[^fold-rs]: `Folded::build` in `vibegraph-lib/src/helas/eval/fold.rs`, which runs `collect_constant_factors`, `pair_config_weights`, `fold_constant_subgraphs` and `fuse_scaled_sums` in that order.
