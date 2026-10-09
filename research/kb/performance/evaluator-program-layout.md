---
type: Design
title: "The helicity program: typed instruction stream, SoA arenas and the fill_arenas loop"
description: "Binary Mul peepholed into typed Instr variants, SoA per-type arenas with 8 B tagged Const, hoisted arena slices, kernel inlining, and the fill_arenas alternatives measured dead."
status: draft
tags: [performance, evaluator, instruction-stream, arenas, interpreter]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n13-sketch, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/13-typed-repr-conventions-design.md#L289-L354", title: "Note 13 §7 (kernel factor-out; Stage A fusion deferred)"}
  - {id: n15-sizes, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/15-eval-optimization-plan.md#L134-L162", title: "Note 15 §1.5 (measured sizes; Const packing)"}
  - {id: n15-track1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/15-eval-optimization-plan.md#L183-L261", title: "Note 15 §2 (eval-layout sessions A0–A6)"}
  - {id: n20-census, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/20-eval-perf-2-plan.md#L91-L115", title: "Note 20 (Mul instruction census)"}
  - {id: n20-s1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/20-eval-perf-2-plan.md#L118-L160", title: "Note 20 S1 mul-split"}
  - {id: n20-s2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/20-eval-perf-2-plan.md#L161-L185", title: "Note 20 S2 dag-validate-once"}
  - {id: n31-e2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L618-L703", title: "Note 31 §E2 (fill_arenas overhead reduction)"}
  - {id: tds-traps, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/threaded-dispatch-study-results.md#L93-L122", title: "Threaded-dispatch study §2 (three traps)"}
  - {id: x86-inline, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L48-L60", title: "x86 AVX2 study: inlining tune"}
  - {id: layout-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/layout.rs#L1-L440", title: "layout.rs (arena classes, OperandRef, Instr, N_KINDS, kind)"}
  - {id: program-struct, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/layout.rs#L527-L562", title: "layout.rs Program"}
  - {id: op-const, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/op.rs#L289-L355", title: "op.rs ConstKind and the packed Const"}
  - {id: run-fill, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/run.rs#L1070-L1120", title: "run.rs fill_arenas"}
  - {id: lower-fuse, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/lower.rs#L599-L640", title: "lower.rs chiral-pair FFV fusion"}
measured:
  - {commit: 95fca7f, host: "Apple M3 Max", command: "cargo bench -p vibegraph-lib --bench eval_strategies"}
  - {commit: 82b68d1, host: "Apple M3 Max", command: "cargo bench -p vibegraph-lib --bench eval_strategies; scripts/mg_perf_compare.sh"}
---

# The helicity program: typed instruction stream, SoA arenas and the `fill_arenas` loop

The evaluator compiles each process once into a **program**: a flat stream of
typed instructions over per-class result arenas, interpreted per phase-space
point by `fill_arenas`. This concept describes that representation and the loop
shape, and records the alternatives measured and rejected. The IR the program
is lowered from is in [the flat op IR](../amplitudes/flat-op-ir.md); how
instructions are ordered is in [execution order](../performance/execution-order.md).

## Pipeline

Diagrams are lowered to a hash-consed `Ast<Sym>`, folded to an `Ast<Const>` with
constants in bind-time pools, expanded over helicity combinations (see
[helicity expansion](../performance/helicity-expansion.md)), and lowered by
`Program::build` into one `Instr` per node.[^layout-rs]

- **Packed leaves.** `Const` is one `u32`: a 2-bit `ConstKind`
  (`Complex`/`Real`/`Ext`/`None`) in the top bits and a 30-bit pool index, so
  `Node<Const>` is 8 B. Pool kind is not a function of the op alone (`CoeffRat`
  folds to a real or complex entry), so `Const::kind` is the source of truth for
  it.[^op-const][^n15-sizes]
- **Typed slots.** Every node's output class is known statically
  (`NodeAnalysis::out_type`), so an operand is a `(class, index)` pair resolved
  at build time, and the runtime reads it straight from the arena its class
  fixes: no per-value tag, and the bra/ket resolution and chirality the generic
  dispatch decided at runtime are baked into the instruction. Fixed-class
  operands are a bare `u32` index; the mixed-class ones (momentum read-offs,
  `Add*` and `AddScaled` term lists) are `OperandRef`s (3-bit class, 29-bit
  index) in a shared `Program::operands` table.

## The arenas

`ScratchSpace` holds six result arenas (`N_ARENAS = 6`, structure of arrays) and
a per-point momentum pool:[^layout-rs][^run-fill]

| arena | element |
|---|---|
| reals | `F` |
| scalars | `C<F>` |
| vectors | `ComplexVector<F>` (4 complex) |
| multivectors | `Multivector<F>` |
| fin (kets) | `Bispinor<F, Ket>` (4 complex) |
| fout (bras) | `Bispinor<F, Bra>` (4 complex) |
| moms | `LorentzVector<F>`, one per distinct signed combination of external momenta |

Elements are bare: no momentum travels with a current. Every current's momentum
is a compile-time signed combination of external momenta, independent of
helicity, so `resolve_moms` fills the momentum pool once per point and
propagators read their momentum by id. Arenas are sized by liveness (a slot is
recycled once its last reader has run), grown once per workspace by
`ensure_sizes`, and never cleared: every slot is written before it is read
within a pass. Boxing the large variants was rejected early: the 96 B currents
are the hot majority, and a pointer chase per operand read costs more than the
enum padding it saves.[^n15-sizes]

## The instruction set

`Instr` has **54 variants** (`N_KINDS = 54`), and `Instr::kind()` is a dense
explicit `match` returning `0..54` in declaration order.[^layout-rs] The families:

- constants and externals: `ComplexConst`, `RealConst`, `External{Scalar,Vector,Fin,Fout}`;
- propagators: `Propagate{Scalar,Vector,Fin,Fout}` (input, mass, width, momentum id);
- sums: `Add{Scalar,Vector,Fin,Fout,Multivector}` over an operand slice, and the
  fused weighted sum `AddScaled` (see
  [constant collection and fused sums](../performance/constant-collection-and-fused-sums.md));
- the **typed multiply peephole**: `MulScalarC`, `MulScalarR`, and
  `Scale{Vec,Fin,Fout,Mv}{C,R}`;
- vertex kernels: `GammaVout`, `GammaFin`/`GammaFout`, the fused chiral
  `FfvVout`/`FfvFin`/`FfvFout`, projectors, `Gamma5*`, `Bilinear`,
  `Pseudoscalar`, `Metric`, the multivector, sigma and Fierz kernels;
- roots: `Flows`, `Hels`, `Configs`, which compute nothing and mark what the
  read-out reads.

`Program` carries `instrs`, a parallel `dest` stream (result slot per
instruction), `arena_sizes`, the `operands` and `mom_operands` tables, the root
kind, and `amp_locs` / `amp_weights` for per-diagram AMP2. A third stream, `loc`,
exists only under `debug_assertions` or `extended-validation`.[^program-struct]

### The binary-Mul peephole

An instruction census over the 14 pruned, expanded programs found `Mul` was
57.6% of all instructions and every one binary: `real × scalar` 62.1%,
`scalar × scalar` 23.8%, current scalings the rest.[^n20-census] `Program::build`
therefore peepholes each `Mul` by its operands' storage classes into one typed
variant, a single field read and one arithmetic op; the `…R` arms do a
real-scale instead of promoting a real to a complex. **There is no generic
fallback.** Binary-Mul is a production invariant (every Mul-emitting site goes
through `reduce_balanced` or an explicit binary add; folding, CSE and helicity
expansion preserve arity; any all-real product is folded away), and `build`
asserts it (arity 2, ≤1 non-scalar operand, no `real × real`): a violation is a
hard panic, not a slow path. Any future e-graph extraction must re-balance to
binary before `build`. The test-only generic evaluator stays variadic, which
keeps the reference oracle structurally independent. Bit-exact; it helped every
process by 13–44% on the tree of the time.[^n20-s1]

### Fusion that exists

- the typed multiply peephole above;
- chiral-pair FFV fusion at lowering: `lower.rs`'s `FuseCtx` makes the walk
  emit one `FfvVout`/`FfvIout`/`FfvOout` node with operands `[a, f, g_L, g_R]`
  at the chiral site, skipping the projector level;[^lower-fuse]
- `AddScaled`, which absorbs single-use constant scalings into a weighted sum.

A general peephole / instruction-selection pass over the full kernel catalog was
deferred pending profiling and never built.[^n13-sketch] Later counters put the
evaluator's limit at macro-op count and dependency latency
([topdown-zen4](../performance/topdown-zen4.md)), and the constant chains were
the cheapest macro-ops to remove; the next lever named there is more independent
work per dispatch (per-kind batched execution), not more fused kernels.

## The loop

`fill_arenas` clears the reuse stamp (see
[arena reuse](../performance/arena-reuse-cache.md)), sizes the arenas, takes
each arena **once** as a local `&mut [T]` through split field borrows, then runs
`for (instr, &loc) in instrs.iter().zip(dest)` with a single `match *instr`.
Every arena access goes through the `rd`/`wr` accessors (checked indexing; see
[bounds checks](../performance/bounds-checks.md)).[^run-fill]

Shape decisions, each measured:

| choice | measured effect |
|---|---|
| **hoist arenas into local slices** (kept) | header reloads 143 → 20; `eval_m2/forward` geomean −4.23% (14/14 rows); `MATRIX1` geomean ratio 1.29× → 1.23×[^n31-e2] |
| soft `#[inline]` on the `*_bare` kernels, `validate_arenas` `#[inline(never)]` (kept) | scalar `forward` ~−4% mean, up to −10% on the two largest; the four biggest kernels (`propagate_{fin,fout,vector}`, `ffv_vout`) may still stay calls; `#[inline]` raises the threshold, it does not force[^x86-inline] |
| fermion-current CSE in `left_current`/`right_current` (kept) | the four spinor products named once (`cmul_add` fusion had blocked CSE of the sibling `cmul`); fermion rows −2.4 to −3.6%, `gg_to_gg` unchanged; reassociating[^n31-e2] |
| dispatch replication, 2-way unrolled (dead) | +7.7% geomean; true threaded dispatch is not expressible in safe Rust, and the unroll doubled the function and evicted six kernels |
| force-inline / out-param the sret kernels (dead) | +0.18% alone, 2.5 pp worse on top of hoisting: the 64 B round trips were store-to-load forwarded and the code growth cost more |
| "packed-complex `GammaVout`" (not a lever) | packing is a codegen outcome, not a source-level choice in safe Rust |
| tail-call threaded dispatch (not adopted) | ties at best; see [threaded dispatch](../performance/threaded-dispatch-study.md) |

Traps found while restructuring the dispatcher, each worth checking in any
future loop change:[^tds-traps]
- **Pass the instruction by reference.** `step(*instr, …)` by value made LLVM
  load all of the ~20-byte `Instr` before the jump and spill the loop counter:
  7% slower on lanes. `match *instr` on a reference restores the shape.
- **Keep `kind()` the identity on the tag.** A permuted `kind()` compiled to a
  table lookup, which tipped soft-`#[inline]` kernels out of line (116
  out-of-line `*_bare` calls against 0). `Instr` is declared in `kind` order for
  this reason. Diff the out-of-line call census between builds, not only the
  timings.
- **Fuse parallel streams per handler** if a dispatcher reads them per step: a
  first threaded layout reading `instrs[pc]`, `dest[pc]` and the next opcode from
  three streams lost 13.5%.

## Debug cross-checks

Per-node output-type and constness assertions became tautological once every
instruction writes a fixed arena by construction, and were deleted. The one
property that is not tautological, momentum-routing and pool-index consistency,
is a property of the compiled DAG, so `validate_arenas` checks it **once per
workspace** on the first point, under `debug_assertions` / `extended-validation`
only. A negative test checks it still catches a corrupted momentum-table index.
This also made `extended-validation` timings honest (3.1–5.4× faster there);
timing claims still come from release `eval_strategies`.[^n20-s2]

## Instruction width

Padding `Node<Const>` from 8 B to 32 B left ns/eval flat within ±2–3% on every
process, including the 2→6, on the pre-SoA tree: instruction-stream width was
not the bottleneck then.[^n15-track1] Narrower operand indices are a separate
open lever on the current tree
([u16-operand-indices-unmeasured](../backlog/performance/u16-operand-indices-unmeasured.md)),
and the [interpreter overhead budget](../performance/interpreter-overhead-budget.md)
places the arena round trip, not decode, as the larger cost.

## Gate

Every layout change is gated on `tests/amplitude_oracle.rs` against MadGraph,
byte equality where it claims bit-for-bit, plus the banked row files'
digests.[^n31-e2]

[^n13-sketch]: Note 13 §7, steps 4–5: kernel factor-out (`kernel.rs`) and the deferred fusion stage.
[^n15-sizes]: Note 15 §1.5: `Const` packing correction, boxing rejected, SoA and momentum pooling as the structural wins.
[^n15-track1]: Note 15 §2: sessions A0–A6 (A0 instruction-width finding).
[^n20-census]: Note 20, measured motivation for the Mul split.
[^n20-s1]: Note 20 S1 `mul-split`.
[^n20-s2]: Note 20 S2 `dag-validate-once`.
[^n31-e2]: Note 31 §E2 and E2b: arena hoisting, the dead items, the fermion-current CSE, and the gate.
[^tds-traps]: Threaded-dispatch study §2.
[^x86-inline]: x86 AVX2 study, inlining tune (`cf3f35b`).
[^layout-rs]: `layout.rs`: module doc, `N_ARENAS`, `OperandRef`, `Instr`, `N_KINDS`, `Instr::kind`.
[^program-struct]: `Program` in `layout.rs`.
[^op-const]: `ConstKind` and `Const` in `op.rs`.
[^run-fill]: `ScratchSpace` and `fill_arenas` in `run.rs`.
[^lower-fuse]: `FuseCtx` in `lower.rs`.
