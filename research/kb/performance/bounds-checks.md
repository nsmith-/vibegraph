---
type: Design Decision
title: Arena bounds checks stay in the evaluator
description: "Unchecked arena access is worth at most 3.5–5.5% today; no shipped build uses unsafe, get_unchecked lives only behind the unchecked-study feature; which safe mechanisms LLVM accepts."
status: draft
tags: [performance, evaluator, bounds-checks, unsafe, codegen]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n17-question, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/17-bounds-check-elimination.md#L11-L87", title: "Note 17 §1–3 (question, branch census, method)"}
  - {id: n17-results, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/17-bounds-check-elimination.md#L88-L126", title: "Note 17 §4–5 (the coupled ceiling, push-era tree)"}
  - {id: n17-mechanisms, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/17-bounds-check-elimination.md#L127-L226", title: "Note 17 §6–9 (candidate mechanisms, go/no-go, resolution)"}
  - {id: n17-retest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/17-bounds-check-elimination.md#L227-L311", title: "Note 17 §10 (re-test 2026-10-04)"}
  - {id: x86-unchecked, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L117-L186", title: "x86 AVX2 study: get_unchecked on the dispatch loop"}
  - {id: run-rd-wr, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/run.rs#L1035-L1069", title: "run.rs rd/wr accessors"}
  - {id: cargo-features, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/Cargo.toml#L11-L25", title: "vibegraph-lib features (unchecked-study)"}
measured:
  - {host: "Intel Xeon Emerald Rapids (family 6 model 207), 4-vCPU Firecracker VM", command: "scripts/bench_schedule.sh 6 opblocked (arms orig=c4bc739, head, unchecked)"}
---

# Arena bounds checks stay in the evaluator

**Decision.** `fill_arenas` reads and writes its arenas with ordinary checked
indexing. No shipped build contains `unsafe`. The only `unsafe` in the
evaluator is `get_unchecked` inside the `rd`/`wr` accessors under the
`unchecked-study` feature, a study hook that measures the floor any
bounds-check removal could reach and is never enabled in normal
builds.[^run-rd-wr][^cargo-features]

**Why.** Removing every check is worth **3.5–5.5%** on the current tree, and the
safe mechanisms that LLVM accepts remove branches, not work, so their ceiling is
under that. The project has measured the `unsafe` alternative twice and declined
it both times as not worth an `unsafe` block in the amplitude core.

## How the checks are written

`fill_arenas` takes each arena once as a local slice (split field borrows of
`ScratchSpace`, so pointer and length stay in registers) and routes every access
through two `#[inline(always)]` accessors:[^run-rd-wr]

```rust
fn rd<T, I: SliceIndex<[T]> + Clone>(s: &[T], i: I) -> &I::Output {
    #[cfg(feature = "unchecked-study")]
    { debug_assert!(s.get(i.clone()).is_some(), "arena read out of range");
      unsafe { s.get_unchecked(i) } }       // SAFETY: study build only
    #[cfg(not(feature = "unchecked-study"))]
    { &s[i] }
}
```

The indices are correct by construction: `Program::build` draws every operand
and result slot from `0..arena_sizes[class]`, `ensure_sizes` grows each arena to
at least that, and `validate_arenas` re-checks the stream once per workspace in
debug and `extended-validation` builds. Writes are pre-sized direct-index
stores, not `Vec::push`.

## The ceiling today

Measured 2026-10-04 on the Emerald Rapids VM (`target-cpu=native`), three arms
round-robin with a different environment-padding layout each round: `orig` (the
tree before the accessors, `c4bc739`), `head` (accessors, checked) and
`unchecked` (accessors under the feature). Geomean over the 8 bench rows of
min-over-rounds time relative to `head`:[^n17-retest]

| arm | `forward` | lanes4 | lanes8 |
|---|--:|--:|--:|
| `orig` | 1.058 [1.02..1.12] | 1.021 [0.98..1.06] | 0.989 [0.92..1.03] |
| `unchecked` | 0.964 [0.91..1.01] | 0.960 [0.90..0.98] | 0.946 [0.89..0.98] |

- **Removing every check is worth 3.5–5.5%.** Unchecked beats head in 5 of 6
  rounds at every width; single cells move 7–30% between rounds on this VM, so
  only the geomeans carry weight.
- Under the feature each release `fill_arenas` instance has ~20% fewer
  instructions (e.g. 5 658 → 4 486) and ~73% fewer conditional jumps (240 → 65),
  with no panic references left. The 171 `helas::eval` unit tests pass with a
  debug assertion on every access.
- **The bound is the size of a codegen accident.** `orig` and `head` differ only
  in routing checked indexing through the accessors, yet `head` is 5.8% faster on
  `forward` (31 fewer conditional jumps, 282 → 251). A safe refactor that does not
  touch the checks moved the scalar path as much as removing them all.

The x86 study's 2–3% (scalar `forward` −2.84% mean, lanes4 −2.98%, lanes2 and
lanes8 near zero; AVX2 host, `be76771`) is the same measurement on an older
tree.[^x86-unchecked] The 3.5–5.5% above is the current bound.

The per-address profile that attributes ~20% of `fill_arenas` samples to
bounds-check branches (M3 Max, `9bad54c`) is a different host, commit and
method; it is not averaged with these, and it is placed beside the other
overhead estimates in
[the interpreter overhead budget](../performance/interpreter-overhead-budget.md).

## Safe mechanisms LLVM accepts

A rustc 1.97 probe of a three-variant dispatch loop (x86-64-v3, `-O`) found three
safe ways to bound a data index:[^n17-retest]

| mechanism | per access | verdict |
|---|---|---|
| clamp, `arena[i.min(len - 1)]` after a one-time non-empty guard | `cmp` + `cmov` (two µops where a fused `cmp`+`jbe` is one) | not worth building |
| mask, `arena[i & (len - 1)]` with power-of-two arena lengths | one `and` | not worth building |
| type-bounded `u16` index into a fixed `[T; 65536]` | nothing | a separate, unmeasured lever |

Clamp and mask remove branches, not work: their hot paths are as long as the
checked one (`Mul` arm 19 instructions against 20), and their smaller static
size is only the cold panic stubs. Their ceiling is under 5% and they would
likely land in the noise. The `u16` index's real effect is halving the
instruction records, a smaller instruction stream, which must be measured as
its own lever; programs with more than 65 536 values per arena would need a
fallback. Open as
[u16-operand-indices-unmeasured](../backlog/performance/u16-operand-indices-unmeasured.md).

Mechanisms that do not work, and why:[^n17-mechanisms]
- **Bind-time pre-resolution into references** (`&'a mut T`, `&'a [Cell<T>]`,
  split borrows): operand values are recomputed every point, so any per-point
  pre-pass must itself index the arena; the check moves, it does not vanish.
- **Hoisted asserts**: operand indices are opaque `u32` data from the instruction
  stream, not induction variables, so a hoisted `assert!(idx < len)` only removes
  the second, redundant check.
- **`OnceCell<T>` arenas**: `get` returns `Option<&T>`, an is-initialised branch
  replacing the bounds branch; and write-once semantics forbid the slot
  overwriting that liveness recycling relies on.

## Why the old +7–11% no longer applies

The first measurement (arm64, `rustc 1.94.1`) found a **+7–11%** ceiling, but
only when *both* the read bounds checks and the `Vec::push` capacity checks were
removed together; reads alone were neutral to negative (−0.4 to −3.5%). The
never-taken `grow_one` cold call forced spills around every write, and the
effect was coupled.[^n17-results] Writes have since become pre-sized
direct-index stores, so the `push` family and its coupling are gone, and the
3.5–5.5% is what remains. The census of that era (57 `panic_bounds_check`
sites, ~32 `grow_one` paths) describes a tree that no longer exists; on rustc
1.97 / x86, LLVM outlines the panic calls, so counting `panic_bounds_check`
symbols reads zero even with every check present, and conditional-jump counts
are the instrument.[^n17-question][^x86-unchecked]

## The `unsafe` prototype, and why it was reverted

The x86 study implemented note 17's escalation option: one getter/setter pair
per arena class on `ScratchSpace`, each a one-line `get_unchecked{,_mut}` under a
shared `// SAFETY:` doc, the dispatch body one `unsafe` block. It was
value-preserving (MadGraph gate bit-exact, lanes bit-identical to scalar) and
bought ~2–3% on the scalar path, neutral on SIMD. The removed branches were
near-perfectly predicted, so deleting them buys I-cache and scheduling, not
mispredict recovery. Reverted.[^x86-unchecked]

One trap it found is worth keeping for any accessor refactor: composite reads
must return a **reference**. Returning a `ComplexVector` by value is free at
`f64` but a 512 B memcpy per operand at `LaneField<8>`, which cost a uniform +60%
on lanes8.

## Re-measuring

```
RUSTFLAGS="-C target-cpu=native" CARGO_TARGET_DIR=target/unchecked-study cargo bench \
    -p vibegraph-lib --bench eval_strategies --no-run \
    --features eval-schedule-study,unchecked-study
BENCH_ARMS="unchecked=<that binary> orig=<a build of c4bc739>" \
    scripts/bench_schedule.sh 6 opblocked
cargo test -p vibegraph-lib --lib --features unchecked-study helas::eval
```

Count conditional jumps in the monomorphised `fill_arenas`, not panic symbols;
per-address attribution on macOS is in
[instruction-level profiling](../tooling/instruction-level-profiling.md).[^n17-retest]

[^n17-question]: Note 17 §1–3: the two branch families, the arm64 disassembly census, the probe method.
[^n17-results]: Note 17 §4–5: the coupled +7–11% ceiling on the `push`-era tree.
[^n17-mechanisms]: Note 17 §6–9: candidate safe mechanisms and why each fails.
[^n17-retest]: Note 17 §10: the 2026-10-04 re-test, the three LLVM-accepted mechanisms, the reproduce commands.
[^x86-unchecked]: x86 AVX2 study, "`get_unchecked` on the dispatch loop": prototype, codegen evidence, bench, revert.
[^run-rd-wr]: `rd`/`wr` in `vibegraph-lib/src/helas/eval/run.rs`.
[^cargo-features]: `unchecked-study` in `vibegraph-lib/Cargo.toml`.
