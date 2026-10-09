---
type: Measurement
title: "Where the interpreter's time goes, and how far each estimate reaches"
description: "The fill_arenas per-address profile (M3 Max, 9bad54c), x86 counter shares and the AOT lower bound side by side: most cost is the per-value arena round trip, not dispatch."
status: draft
tags: [performance, evaluator, interpreter, dispatch, profiling]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
measured:
  - {commit: 9bad54c, host: "Apple M3 Max, macOS 15.7.7", command: "samply on validate_sigma (sigma_gate_matches_madgraph), release-debug, extended-validation"}
  - {commit: 03c31e6, host: "Intel Xeon Emerald Rapids (family 6 model 207), 4-vCPU Firecracker VM", command: "cargo bench --bench aot_kernels (aot-study feature)"}
  - {commit: b504391, host: "Intel Xeon Emerald Rapids (family 6 model 207), 4-vCPU Firecracker VM", command: "cargo bench --bench aot_kernels (aot-mg-study feature)"}
sources:
  - {id: fas, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/fill-arenas-asm-study-results.md#L16-L231", title: "fill_arenas instruction-level study, §0–§5"}
  - {id: n31-e0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L488-L516", title: "Note 31 E0 (summary of the study)"}
  - {id: n31-e2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L618-L703", title: "Note 31 E2 (arena hoisting and the measured-dead items)"}
  - {id: aot5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/aot-kernels-study-results.md#L194-L226", title: "AOT study §5, Reading"}
  - {id: aotm7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/aot-kernels-study-results.md#L515-L549", title: "AOT study §M7, Reading"}
  - {id: n17-10, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/17-bounds-check-elimination.md#L227-L300", title: "Note 17 §10, bounds-check re-test (Emerald Rapids, 2026-10-04)"}
  - {id: td, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/topdown-zen4-results.md#L39-L52", title: "Top-down Zen 4, §1 Answers"}
  - {id: tds4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/threaded-dispatch-study-results.md#L297-L370", title: "Threaded-dispatch study §4 reading (M3 Max counters)"}
---

# Where the interpreter's time goes

The helicity program runs in `fill_arenas` (`vibegraph-lib/src/helas/eval/run.rs`), a
`match` over typed instructions that reads operands from and writes results to per-class
arenas. Four independent estimates say how its time divides. They differ in host, ISA,
commit and method, so each is given with its provenance and none is subtracted from
another. The reading they agree on: **most of the interpreter's overhead is the store and
reload of every value through its arena, and the dispatch jump itself is the smaller
part.**

| estimate | host, commit | method | what it says |
|---|---|---|---|
| per-address profile | M3 Max, `9bad54c` | samply PC samples on the linked binary | dispatch block 30.5%, loads 22.9%, FP 20.4%, bounds 20.4% of `fill_arenas` |
| bounds checks removed | Emerald Rapids, 2026-10-04 | `unchecked-study` build against head | 3.5–5.5% |
| mispredicts | M3 Max (`2008fbf`); Zen 4 (`5ced9bb`) | Instruments "discarded"; PMU bad speculation | ~1–2% on the 2→6 |
| interpreter vs compiled | Emerald Rapids, `03c31e6`/`b504391` | timing differences between renderings | ≥23–51% of scalar `forward` is non-arithmetic |

## The per-address profile (M3 Max, `9bad54c`)

`fill_arenas` was 33.6% of the busiest thread's self time in the σ gate
(`sigma_gate_matches_madgraph`, 14 009 leaf samples at 1 kHz), reproducing note 30's 34.2%.
Every instruction of the linked `release-debug` binary was classified by opcode and
weighted by samples.[^fas]

```
samples       %  #insns  kind
   4266  30.45%      10  loop control + dispatch block
   3211  22.92%     355  load
   2864  20.44%     451  FP arithmetic
   2854  20.37%     302  bounds check (cmp / branch / len load)
    485   3.46%      95  store
    262   1.87%     586  integer / address math
```

- **Dispatch is a jump table**: 38 `u16` entries, `ldrb`/`adr`/`ldrh`/`add`/`br`, one
  `br` in the function, checked entry by entry against the 38 `Instr` variants of that
  commit. There was no compare chain to fix.
- **The 30.5% block is ten instructions** executed once per program instruction. Within it,
  10.87% of `fill_arenas`'s samples sat on the jump-table `ldrh` that the `br` consumes, the classic load-dependent
  indirect-branch stall, and 16.6% on the `cmp` at the arms' merge point.
- **Bounds checks** were identified structurally: a branch into the `panic_bounds_check`
  tail plus its feeding `cmp` and the length load (108 panic sites). Only 12 samples
  landed in the cold tail; the cost is the hot-path guards.
- **Hot regions**: loop control 18.5%, `GammaVout` 18.2% (inlined, scalar-lane `d`
  register FMA chains, not packed), `Metric` 14.1%, dispatch 12.0%, `MulScalarR` 7.3%.
- A second classifier, bucketing by innermost inlined frame, gave 21.2% for `Vec` header
  dereference plus slice indexing, matching the 20.4% bounds-check figure.

**What the 20% bounds-check share includes.** The per-address figure counts the length
reloads and compares, and several of those loads also serve as the data path's header
loads. It is an upper bound on what removing checks could buy, not a measurement of it.
Removing every check on a later tree (Emerald Rapids, `unchecked-study`) measured
3.5–5.5%.[^n17-10] See [bounds checks](bounds-checks.md).

**Attribution limits.** samply records the oldest unretired instruction, so stalls are
charged to the waiting instruction (skid). The 30.45% block total is robust; its split
between the `cmp` and the `ldrh` is not. 10.7% of instructions carry DWARF line 0
(tail-merged blocks), forward-filled to the preceding arm, so sub-1% arm shares carry
about ±0.5%. The profile is of the σ gate's process mix weighted by integration cost, not
of one process, and it is one run of one binary. The method is in
[instruction-level profiling](../tooling/instruction-level-profiling.md).

### What has changed since `9bad54c`

The study predates the op-blocked execution order and arena hoisting. Read its
structural findings against the current code:[^n31-e2]

- **Header reloads.** The study found 143 loads re-fetching arena `ptr`/`len` off the
  `ScratchSpace` pointer, because LLVM could not prove arena stores miss the `Vec`
  headers (`MulScalarR`: 17 instructions around one `fmul.2d`). `fill_arenas` takes
  each arena once as a local slice through split field borrows; that cut the reloads
  to 20 and `eval_m2/forward` by 4.2% geomean. The 108 `panic_bounds_check` sites stayed:
  the indices come from the instruction stream, so hoisting did not make them hoistable.
- **Dispatch replication** (a `br` per arm, the study's §4.2 suggestion) is not
  expressible in safe Rust; the 2-way-unroll approximation was +7.7% and was dropped.
  Tail-call threading on nightly was measured separately and also loses: see
  [threaded dispatch](threaded-dispatch-study.md).
- **Forcing the four out-of-line kernels inline** (§4.3: `ffv_vout_bare`,
  `propagate_{vector,fin,fout}_bare`, each paying an sret stack temp, a 64-byte copy and
  rematerialised loop constants) was +0.18% alone and 2.5 points worse on top of
  hoisting: the round trips were store-to-load forwarded and the code growth cost more.
- The 640-byte frame (§4.4), the scalar-lane `GammaVout` (§4.5) and the 10.7% tail-merged
  instructions (§4.6) are observations of that binary, not measured levers.

## The ahead-of-time lower bound (Emerald Rapids)

Rendering the same program as straight-line Rust removes dispatch, operand decoding,
arena indexing and bounds checks, and keeps the arithmetic. With the kernels called out
of line (`aot_out`), `vm − aot_out` is therefore a lower bound on everything else the
interpreter spends: **at least 23–51% of scalar `forward`** (`ee_to_mumu` 31%,
`gg_to_gg` 49%, `ee_to_mumu_tata_qcd0` 27%).[^aot5] Scaled to the `aot_out` times, the
same arithmetic runs at 75–90% of the FP-issue ceiling of its operation mix, against
37–68% inside the interpreter (see [roofline census](roofline-census.md)).

The second half of the study splits that bound.[^aotm7] A MadGraph-style rendering (each
kernel reads its operands from slot arrays and writes its result back, one call per
instruction) removes dispatch and decode but keeps the memory round trip; it gains only
1.05–1.27× on the small rows. A by-value rendering, where values pass between kernels in
registers and the caller's frame, gains 1.4–2.2×. So of the interpreter's overhead,
decode and dispatch is the small share and **the arena round trip of every operand and
result is the larger**. This is inferred from three arms' differences, not from counters,
on a shared 4-vCPU VM with 10–20% per-cell layout noise; the conclusion rests on
factor-level differences that both runs reproduce. Why compiled programs still lose on
large processes is in [ahead-of-time compilation](aot-compilation-study.md).

## Counters

PMU readings come later and from other hosts.[^td] On Zen 4 (`5ced9bb`, see
[top-down on Zen 4](topdown-zen4.md)), indirect mispredicts are 0.011–0.014 per VM
instruction on the 2→6 (bad speculation 1–2%) and 0.05–0.11 on small rows (4–8%), and
the run retires 49–61% of slots: it is bounded by macro-op count and FP dependency
latency, not by branches or memory. On the M3 Max, Instruments puts discarded work at
1.3% of the op-blocked 2→6's cycles.

## Reading the estimates together

- The ~30% dispatch block on the M3 Max is real per-instruction work, but it is a sample
  share on one ISA at one commit, and skid inflates the merge-point `cmp`. The
  mispredict share is small on every host measured, so that block's cost is instructions
  executed, not prediction.
- The 20% per-address bounds share and the 3.5–5.5% elimination ceiling measure
  different things; quote the second when sizing a change.
- Instruction count is the lever the AOT and counter evidence both point at: fewer VM
  instructions per event (as [constant collection](constant-collection-and-fused-sums.md)
  did) or more work per dispatch.

[^fas]: `fill-arenas-asm-study-results.md` §0–§5, M3 Max at `9bad54c`; note 31 E0 restates its headline.
[^n31-e2]: Note 31 E2: arena hoisting merged as `82b68d1` (merge `9ac8858`); items 2–3 measured and not landed.
[^aot5]: AOT study §5 at `03c31e6`, run 3.
[^aotm7]: AOT study §M5–§M7 at `b504391`.
[^n17-10]: Note 17 §10, the 2026-10-04 re-test.
[^td]: Top-down Zen 4 §1; threaded-dispatch study §4 for the M3 Max counter.
