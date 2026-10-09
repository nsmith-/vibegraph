---
type: Feasibility Study
title: "Tail-call-threaded dispatch (not adopted)"
description: "become-threaded dispatch is bit-identical but loses 1–9% to the match loop; preserve_none with register arenas only ties at lanes8; archived at tag study/threaded-dispatch."
status: draft
tags: [performance, dispatch, interpreter, tail-calls, nightly]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
measured:
  - {commit: 6bd7325, host: "Intel Xeon @ 2.8 GHz, Cascade Lake (family 6 model 85), 4-vCPU Firecracker VM, 1 MiB L2", command: "eval_strategies, nightly-2026-09-24, -C target-cpu=native, bench profile"}
  - {commit: 2008fbf, host: "Apple M3 Max, macOS 15.7, 16 MiB L2 per P-cluster", command: "scripts/bench_dispatch.sh (at the tag), min over rounds"}
sources:
  - {id: tds, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/threaded-dispatch-study-results.md#L13-L608", title: "Tail-call-threaded dispatch and the execution order — results"}
  - {id: tds1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/threaded-dispatch-study-results.md#L49-L122", title: "§1 The dispatcher; §2 three traps"}
  - {id: tds5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/threaded-dispatch-study-results.md#L371-L428", title: "§5 Profile of a threaded handler"}
  - {id: tds6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/threaded-dispatch-study-results.md#L429-L582", title: "§6 preserve_none; §7 what this leaves"}
---

# Tail-call-threaded dispatch (not adopted)

The question: does replacing `fill_arenas`'s single `match` dispatch with a chain of
per-opcode handlers that tail-call each other (`become`, nightly `explicit_tail_calls`)
beat the loop? It works, it is bit-identical, and it does not beat it. The code is not in
the tree: it is archived under the git tag `study/threaded-dispatch` (`4657c92`), with its
bit-identity test, `scripts/bench_dispatch.sh` and its CI job. What stayed is the
execution-order controls and tooling, covered in [execution order](execution-order.md).[^tds]

**Result**, geomean over the 8 bench rows, time relative to the `match` loop at the
production (op-blocked) order:

| arm | host | scalar | lanes4 | lanes8 |
|---|---|--:|--:|--:|
| threaded, Rust ABI | Cascade Lake (medians) | 1.016 | 1.015 | 1.052 |
| threaded, Rust ABI | M3 Max, 6 randomised layouts | 1.113 | 1.063 | 1.043 |
| threaded, `preserve_none` + register arenas | M3 Max, 6 randomised layouts | 1.058–1.063 | 1.013–1.014 | 0.998–1.000 |

The M3 Max Rust-ABI figures from a single memory layout were 1.087 / 1.021 / 1.010. The
randomised-layout figures supersede them: a single scalar row moves 10–22% with layout (see
below). On Cascade Lake, run-to-run drift is 1–4% per cell, so a geomean inside ±2% is a
tie, and the medians there were not examined for the layout effect.

## The dispatcher

`fill_arenas`'s per-instruction work is one `#[inline(always)] fn step(&Instr, loc,
&mut Arenas, &EvalEnv)`; the loop is `for … { step(…) }`. The threaded arm starts a chain
of handlers, one monomorphisation per opcode in a 256-entry `const` table:[^tds1]

```rust
fn handler<F: Real, const OP: u8>(vm: &mut Vm<'_, '_, F>, rest: &[Threaded]) {
    let [cur, tail @ ..] = rest else { unreachable!() };
    if cur.instr.kind() == OP {
        step(&cur.instr, cur.dest as usize, &mut vm.arenas, vm.env);
    } else {
        unreachable!()
    }
    become (Table::<F>::HANDLERS[cur.next as usize])(vm, tail)
}
```

Inside the `kind() == OP` branch `step`'s `match` folds to one arm, so instruction bodies
are textually shared and the arms differ in dispatch alone. Each `Threaded` record carries
the instruction, its destination and the next opcode, so a handler finds its successor in
the line it already loaded. At the tag the instruction set had 53 kinds (212 handler
instances over the bench's four fields), each ending in one `jmp *table(,%rax,8)`; the
instruction set now has 54 (`N_KINDS`, `vibegraph-lib/src/helas/eval/layout.rs`).

**Correctness.** `threaded_dispatch_matches_match_loop_bit_for_bit` ran both dispatchers in
one build and compared `AMP2`, `JAMP2` and per-helicity `|M|²` `to_bits` over every
MG-validated process at `f64` and `LaneField<4>`. A planted fault (a handler skipping
`step`) failed it.

## Traps met on the way

Each is a general lesson for any dispatcher change:

- **Three streams through `&mut Vm` lost 13.5%.** Reading `instrs[pc]`, `dest[pc]` and
  `opcodes[pc+1]` separately cost three bounds checks and a pointer hop per arena (44 hot
  instructions for `MulScalarC`). Fusing them into one record and holding the arenas by
  value reached parity.
- **`step(*instr, …)` by value cost the `match` loop 7% on lanes.** LLVM loaded all 20
  bytes of the instruction before the jump, and the register pressure spilled the loop
  counter. `step(&Instr)` with `match *instr` restored the shape.
- **A permuted `kind()` evicted the kernels.** Kinds that were a permutation of declaration
  order compiled `kind()` to a table lookup, whose cost tipped soft-`#[inline]` kernels
  over LLVM's threshold: 116 out-of-line `*_bare` calls against 0, and the threaded arm fell
  to 1.15 / 1.12 / 1.32. `Instr` is now declared in `kind` order, so `kind()` is the tag
  load. **Diff the out-of-line call census between builds, not only the timings.**

## Why it loses

- **The predictor already predicts the single site.** Intel's indirect predictor has been
  history-based since Haswell, so one dispatch site predicts about as well as one per
  handler (Rohou, Swamy and Seznec, "Branch prediction and the performance of interpreters — don't trust folklore", CGO 2015, measured the same). The M3's predictor leaves even
  less to win.
- **Where threading does win prediction, it pays it back in instructions.** Apple's
  bottleneck counters on `gg_to_gg` show threading cut discarded work from 8.3% to 3.7% and
  delivery stalls from 9.2% to 5.2%, yet it ran 4–10% slower. A threaded `Metric` handler is
  57 instructions against the arm's 40 plus a 10-instruction shared dispatch block. The +7
  are a frame record (cold panic calls make every handler non-leaf), four `Vm` loads of
  arena pointers and lengths, and the stream and kind checks.[^tds5]
- **Under the Rust ABI every handler is a fresh callee.** At `f64`, 16 of 53 handlers saved
  2–16 callee-saved registers (propagators 14, multivector `Fin`/`Fout` 16) on every
  dispatch; at lanes4, 32 of 53 saved up to 18. The `match` loop saves them once per pass.
- **On the 2→6 the dispatchers are indistinguishable** in every predictable order. Neither
  rescues an unpredictable one: a random within-level order costs 2.2× on the M3 Max under
  both (2.16× threaded, 2.21× `match`), because a per-handler site must still predict a
  random successor.
- **The dispatcher does not change which order wins**: every order ranks the same under both
  arms on both hosts.

## `preserve_none`

`become` keeps the Rust ABI. LLVM's `preserve_none` convention (no callee-saved registers;
on AArch64 24 argument registers, `x20`–`x28` then `x0`–`x7`, `x9`–`x15`) is what threaded
interpreters use, exposed on the pinned nightly as `extern "rust-preserve-none"`
(`#![feature(rust_preserve_none_cc)]`).[^tds6]

- **Step 1, the convention alone**, removed callee-saved saves from almost every handler
  and cut stack traffic 43% at `f64`. The frame record stays: it comes from being non-leaf.
- **Step 2, the arenas as arguments** (nine slices, 20 registers, plus one pointer for pools
  and environment) keeps arena pointers in registers down the chain. Arguments in
  `x0`–`x15` then spill around every out-of-line kernel or panic call (stack ops 342 → 675
  at `f64`), still cheaper than the Rust ABI's saves.
- Together they recover 4–6 points over the Rust ABI (step 1 worth 1.2 / 2.2 / 2.6, step 2
  3.8 / 2.7 / 1.9 at scalar / lanes4 / lanes8). Step 2 ties `match` at lanes8, trails by
  1.4% at lanes4 and 6% at scalar (the 2→6 11%), reproduced to half a point in two sweeps.
- **x86-64 cannot run step 2**: an LLVM codegen bug. A `preserve_none` function that
  `become`s itself through a function-pointer table is correct with up to 11 integer
  arguments and returns garbage at 12 under `-O` (segfaults at `-O0`): with all 12 argument
  registers in use LLVM loads the tail-call target into `rax`, the 12th argument register,
  so the callee receives its own address. Step 2 needs 20 registers, more than x86-64 has
  for this anyway. Step 1 builds and passes on x86-64 (under Rosetta); its timing there was
  not measured. The 25-line probe is not reported upstream.

**What remains at scalar width is structural**: the frame record, argument spills around
out-of-line calls, and per-instruction stream and kind checks, fixed costs that weigh most
where an instruction is cheapest. The one step left untried is a leaf handler, with every
panic path behind a `preserve_none` cold call, dropping the frame record and the spills
around it.

## Memory layout moves scalar rows by up to 22%

Two `preserve_none` sweeps disagreed by more than their own spread while the byte-identical
`match` binary moved +10% on the 2→6 and −17% on `ee_to_mumu`. The layout follows the
process environment: `ee_to_mumu` under `match` ran 3.92 µs with 0–128 bytes of extra
environment, 4.80 µs (+22%) with 136–300 and again near 640, and 3.9–4.0 µs from 768 bytes
to 8 KiB, each point steady to 1%. The windows are not periodic, and heap placement after
earlier benchmarks in the same process is a likelier mechanism than stack alignment
alone; it was not isolated. The bench scripts now pad the environment by a different
length each round, the same for every arm, and take min over rounds. The protocol is in
[benchmarking the evaluator](microbenchmark-protocol.md).

## Verdict

Not adopted. Threaded dispatch ties at best, needs nightly twice over (`explicit_tail_calls`
still warns as incomplete; `preserve_none` is a separate feature) and would pin a nightly, a
CI job and the interpreter's shape for a dispatcher that loses on the hosts that matter.
The prediction gain is real and measured; the Rust ABI spends it on register saves and
arena reloads, and `preserve_none` buys back only a tie at lanes8. The overhead the
interpreter does carry is mostly the arena round trip, not the dispatch jump (see
[interpreter overhead](interpreter-overhead-budget.md) and the
[program layout](evaluator-program-layout.md)).

Two costs common to both arms were found and fixed on the way: fermion propagators called
libm `hypot` through `num_complex`'s overflow-safe reciprocal (≈1% scalar, ≈2–3% with the
lane fallback's `memset_pattern16`); they now take `Complex::inv` (`conj / |z|²`). And two
profile artefacts not to chase: `libsystem_kernel` at ~4.5% is wall-clock samples of a
blocked thread, and samples on dylib import stubs symbolicate to the preceding text symbol
(`RawVec::reserve`), reading as an allocation that does not exist.

## Reproduce

```
git checkout study/threaded-dispatch
DISPATCH_ARMS="match threaded preservenone" scripts/bench_dispatch.sh 6 opblocked
cargo +nightly-2026-09-24 test -p vibegraph-lib --lib --features preserve-none-dispatch -- threaded
```

[^tds]: `threaded-dispatch-study-results.md`, status block and §3–§4, at `6bd7325` (Cascade Lake) and `2008fbf` (M3 Max).
[^tds1]: §1–§2.
[^tds5]: §4 counters and §5 profile (samply 4 kHz, M3 Max).
[^tds6]: §6–§7.
