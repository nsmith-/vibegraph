---
type: Design
title: "Per-lane decisions in the scalar trait: masks, select, indices (draft)"
description: "What a comparison means on a lane scalar: a scalar trait without PartialOrd whose comparisons return masks, with select, explicit uniform/any/all branches and per-lane indices; the measured cost, FlashJet's clustering shape and cuTile's tile model."
status: draft
tags: [simd, lanes, real-trait, generics, gpu, clustering, design, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: item, resource: "../backlog/performance/real-has-no-lane-masks-or-bin-search.md", title: "Real has no lane-wise comparisons, selects or bin search"}
  - {id: lanes, resource: "simd-lane-evaluation.md", title: "Lane-batched evaluation of eval_m2 (the lane-uniformity contract)"}
  - {id: seam, resource: "../phase-space/sampler-and-map-seams.md", title: "The importance sampler and the phase-space map, designed together"}
  - {id: flashjet, resource: "https://github.com/jet-universe/FlashJet/tree/2adbfa054074a73d6e719ca2fd957cdc3a35f241", title: "FlashJet at 2adbfa0 (GPL-3.0; read, not built)"}
  - {id: cutile-py, resource: "https://github.com/NVIDIA/cutile-python/tree/285b3a78", title: "cuTile Python at 285b3a78 (release 1.7.0)"}
  - {id: cutile-rs, resource: "https://github.com/NVIDIA/cuda-rust/tree/9df9e7ce/cutile-rs", title: "cutile-rs (NVIDIA cuda-rust) at 9df9e7ce"}
  - {id: tile-ir, resource: "https://docs.nvidia.com/cuda/tile-ir/latest/sections/operations.html", title: "CUDA Tile IR operations"}
measured:
  - {commit: b5afbde, command: "cargo check -p vibegraph-lib with Real's supertraits reduced to Zero + One + FloatConst + the arithmetic operators"}
---

Draft for the user's review (2026-10-26). It answers the design questions of
[real-has-no-lane-masks-or-bin-search](../backlog/performance/real-has-no-lane-masks-or-bin-search.md)
and is written into the same PR as the
[sampler and map seams](../phase-space/sampler-and-map-seams.md), which need
none of it to start: the sampler trait takes its scalar as a type parameter and
is implemented at `f64` first.

Labels: **[code]** read in this repository at `b5afbde`; **[measured]** the
compile experiment below; **[FlashJet]**, **[cuTile]** read in those sources at
the pinned commits, by research agents, with the cited lines spot-checked;
**[inferred]** a judgement.

## The problem

- [code] `Real` (`helas/repr/mod.rs:97`) is `num_traits::Float` plus
  extras. `Float` requires `PartialOrd`, `NumCast` and `ToPrimitive`, so
  `LaneField<N>` must give `<` one answer for the whole pack (lexicographic),
  reduce predicates with any/all, and convert to an integer through lane 0
  (`lane_field.rs:20-26`).
- [code] Code that branches on `F` is correct on lanes only when every lane
  takes the same side. The evaluator relies on that by construction (mass,
  on-axis and at-rest forks; `helas/eval/lanes.rs`), pinned by
  `eval_m2_lanes_match_scalar`. Nothing stops a new branch that is not uniform:
  it compiles and gives one lane's answer to all of them.
- [code] The consumers waiting on per-lane decisions are:
  - the VEGAS draw: `(pos as usize).min(nbins − 1)` and a gather of two edges
    (`vegas.rs` `draw_into`);
  - the map's windows: `windowed` branches on `l < h`, `outside_window` returns
    a `bool`, and a two-interval window chooses per point
    (`diagram_channel.rs:1876-1905`);
  - cuts, with an early exit once every lane has failed;
  - kT clustering (`coupling/cluster/kt.rs`);
  - the PDF grid lookup, a search.

## What a comparison should mean: the options

1. **Keep `Float`; `<` stays lexicographic.** Per-lane decisions go through a
   companion trait. Misuse stays silent.
2. **Keep `Float`; "uniform or panic".** `LaneField`'s `<`, `==`, predicates
   and `ToPrimitive` debug-assert that every lane agrees and return the common
   answer. Misuse panics in debug builds, and only on a batch that is not
   uniform.
3. **Drop `Float` as a supertrait.** The scalar trait has no `PartialOrd`.
   Comparisons return an associated `Mask`, there is no integer conversion
   through one lane, and every decision on data is written as one of a few
   explicit forms. Misuse does not compile.

The user leans toward option 3. The research supports it:

- [cuTile] Every layer of NVIDIA's tile model makes the same choice:
  - comparing tiles gives a boolean tile (`tile<i1>` in Tile IR);
  - per-element choice is `select`/`ct.where`;
  - control flow accepts only a 0-d bool, and branching on a tile is a compile
    error (cuTile Python `_ir/type.py:541`, `_ir/op_impl.py:680`).
  - NVIDIA's Rust binding names its comparisons `lt_tile`, `eq_tile` and so on
    and deliberately does not implement `PartialOrd` (cutile-rs
    `cutile/src/_core.rs:3047`).
- [FlashJet] Its GPU and torch backends never compare at pack level. Every
  decision is an elementwise `where`, and the argmin is "min value, then lowest
  index among equals" (`triton_backend.py:93-103`).
- [inferred] For `f64` with `Mask = bool` the scalar is the 0-d case: `select`
  is an `if`, and `any`/`all` return their argument. So option 3 is bit-for-bit
  at `F = f64` by construction.

**Recommendation: option 3 as the end state.** Option 2's check survives in it
as an explicit method (`Mask::uniform`, below) rather than as a hidden meaning
of `<`.

## The trait (sketch)

```rust
pub trait Real:
    Copy + Send + Sync + 'static + Debug
    + num_traits::Num            // + − × ÷, zero, one; see "Equality" below
    + Neg<Output = Self> + FloatConst
{
    type Mask: Mask;
    type Index: Index<Mask = Self::Mask>;
    const LANES: usize;          // 1 for f64

    fn splat(x: f64) -> Self;    // replaces F::from(x).unwrap() and cast::<F>
    fn lane(self, i: usize) -> f64;      // unpack, e.g. for the sampler's scalar accumulate

    // per-lane math: forwards to f64's methods on each lane
    fn sqrt(self) -> Self;  fn abs(self) -> Self;  fn ln(self) -> Self;  fn exp(self) -> Self;
    fn powi(self, n: i32) -> Self;  fn powf(self, y: Self) -> Self;  fn min(self, o: Self) -> Self;
    fn max(self, o: Self) -> Self;  fn floor(self) -> Self;  fn mul_add_fast(self, a: Self, b: Self) -> Self;
    /* sin cos atan acos asin atan2 sinh cosh tanh ln_1p exp_m1 recip signum hypot … as used */

    // comparisons and predicates return masks
    fn lt(self, o: Self) -> Self::Mask;  fn le(…);  fn gt(…);  fn ge(…);  fn eq_lanes(…);
    fn is_finite(self) -> Self::Mask;    fn is_nan(self) -> Self::Mask;
    fn select(m: Self::Mask, a: Self, b: Self) -> Self;

    // per-lane indices
    fn floor_index(self, max: usize) -> Self::Index;  // min(floor(self), max), self ≥ 0
    fn from_index(i: Self::Index) -> Self;
    fn gather(table: &[f64], i: Self::Index) -> Self;
}

pub trait Mask: Copy + BitAnd<Output = Self> + BitOr<Output = Self> + Not<Output = Self> {
    fn splat(b: bool) -> Self;
    fn any(self) -> bool;
    fn all(self) -> bool;
    /// The one answer every lane agrees on; debug-asserts `any == all`.
    fn uniform(self) -> bool;
}

pub trait Index: Copy {
    type Mask;
    fn splat(i: usize) -> Self;
    fn add(self, k: usize) -> Self;
    fn eq_lanes(self, o: Self) -> Self::Mask;
    fn select(m: Self::Mask, a: Self, b: Self) -> Self;
    fn lane(self, i: usize) -> usize;
}
```

At `f64`: `Mask = bool`, `Index = usize`, and every method is the scalar
operation itself. At `LaneField<N>`: the mask is `wide`'s compare result, the
index is `[u32; N]`, and `select` is one blend instruction.

**Every decision on data takes one of four explicit forms**, each easy to find
with grep and each with a direct tile-model counterpart:

| form | when | lanes | tile model |
|---|---|---|---|
| `F::select(m, a, b)` | a per-point choice (window edges, a clamp, a fold step) | blend | `select` / `ct.where` |
| `if m.uniform()` | a fork that is uniform by construction (mass, on-axis, card constants) | one branch, debug-asserted | reduce to 0-d, then `if` |
| `if m.any()` / `m.all()` | an early exit or a reduction (every lane failed the cuts) | one branch | reduction, then `if`/`break` |
| scalar fallback | rare, deep divergence (clustering graph disagreements, `Err` exits) | unpack, run each lane as `f64`, repack | a separate kernel or host path |

## Cost, measured

[measured] In a throwaway worktree at `b5afbde`, `Real`'s supertraits were
reduced to `Zero + One + FloatConst` plus the arithmetic operators, and
`cargo check -p vibegraph-lib` (library only; tests not compiled) was run. It
reports 436 errors. Treat that as a lower bound, since some errors hide others.

| class | count | where | what it costs |
|---|---|---|---|
| comparisons on `F` (`<`, `>`, `<=`, `>=`, `==`, `!=` and bounds that need them) | 95 | `phasespace/diagram_channel.rs` 34, `helas/repr/lorentz.rs` 25, `cuts.rs` 18, `phasespace/channel.rs` 6, others ≤ 3 | the real work: each site becomes `select`, `uniform()` or `any/all` |
| missing methods | 167, across 23 names | `sqrt` 30, `max` 23, `abs` 19, `min` 6, `ln` 5, `infinity` 5 … | mechanical: methods of the new trait |
| casts (`F::from(x).unwrap()`, `NumCast`) | ~29 calls plus type errors | | mechanical: `F::splat` |
| `num_complex::Complex<F>` arithmetic | 74 | the evaluator | needs `F: Num`, so kept by keeping `Num` (below) |

The per-site decision is where the value lies. `lorentz.rs`'s forks are the
uniform ones the lane contract already lists, so they become `uniform()`. The
map's comparisons in `diagram_channel.rs` are per point, so they become
`select`. The cuts become masks with `any`/`all`.

## Equality and `num_complex`

[code] `C<F>` is `num_complex::Complex<F>`. Its arithmetic needs `F: Num`, and
`Num` requires `PartialEq`, so keeping `num_complex` keeps `==` on `F`. Two
ways:

- **Keep `Num` (chosen by the user, 2026-10-27).** `==` survives on `F`. On `LaneField` it becomes "uniform or
  panic" (option 2, for equality only). Most equality sites compare against
  card constants (`mass == 0`), which are uniform by construction.
- **Own complex type (deferred).** `F` then needs no `PartialEq`, and the remaining
  equality sites become `eq_lanes` with `select` or `uniform()`.

**The lift, measured.** [measured] In a throwaway worktree at `d57fbb1`,
`C<F>` was replaced by a bare `struct Cx<F> { re, im }` and
`cargo check -p vibegraph-lib --tests` was run. It reports 936 errors, almost
all of them the struct lacking an operation, so they measure the API surface,
not call sites to rewrite:

- **Methods:** `new` 90, `norm` 58, `zero` 33, `norm_sqr` 16, `one` 14,
  `conj` 10, `i` 9, `scale` 6, `is_zero` 1.
- **Operators:** `C ± C` 140, unary `−` 58, and mixed operands with `F`,
  `WaveformSlot` and the Lorentz types (≈ 40). Also `Sum`, `Display`,
  `LowerExp`, `From<F>`, and `==` on `Cx<f64>` (18, in tests).
- **Files:** `helas/vertex.rs` 348, `helas/repr/lorentz.rs` 266,
  `helas/eval/run.rs` 102, `helas/eval/kernel.rs` 75, the rest ≤ 35.
- **The `f64` boundary:** 10 files use `Complex64` directly, notably the UFO
  layer (`ufo/expr.rs`, `ufo/mod.rs`), which is `f64`-only and can keep
  `num_complex` with a conversion where couplings enter the evaluator.

[inferred] Mirroring `num_complex`'s names keeps call sites unchanged. The
work is one module of operator impls (macro-generated: `C∘C`, `C∘F`, `F∘C`,
the assign forms, `Neg`, `Sum`, the formatting traits), roughly 300–400
lines, plus the `Complex64` conversions. The risk is bit identity: `*` and `/`
must copy `num_complex`'s formulas exactly, and `norm` is `re.hypot(im)`, not
`norm_sqr().sqrt()` (`num-complex-0.4.6/src/lib.rs:217`). A test pins every
operation bit-for-bit against `num_complex` at `f64` on random and edge-case
inputs (zeros, infinities, subnormals). One PR, inside stage B2.

## Jet clustering, from FlashJet

[FlashJet] FlashJet is a generalized-kt package (GPL-3.0, so we take ideas,
not code). It runs one program per event with the particles across lanes
(Triton), a lock-step batch with an alive mask (torch), or one thread per event
(C++). Its argmin takes the minimum value, then the lowest slot index among
equals. A pair beats the beam only if strictly smaller. It keeps slots fixed
rather than compacting, because compacting "would renumber slots and change
how argmin ties break" (`_cpu_kernel.cpp:14-17`). It also found that ΔR²
needs an explicit FMA at every site: without one, a 1-ulp difference flipped
ties in 27% of C/A events (`triton_large.py:301-311`).

What that means for our clustering, 2–8 partons per point:

- [code] Our clustering diverges less than FlashJet's. In a batch of one
  process, `n` is the same on every lane, the loop runs a fixed `n − 2` steps
  (`kt.rs:674`), and every lane visits the same pairs in the same order
  (`kt.rs:592-593`).
- [inferred] **One lane per event**, the torch shape. FlashJet's GPU shape (one
  event per program, particles across lanes, at least 16 wide) would leave most
  lanes idle at 2–8 partons.
- [inferred] **The argmin is a fold over the fixed pair order:**
  `better = v.lt(best) & admissible; best = select(better, v, best); win = Index::select(better, k, win)`.
  The strict `<` keeps "first minimiser in visit order" (`kt.rs:16-18`)
  bit-for-bit at `f64`, with no cross-lane reduction.
- [inferred] **Fixed slots and an alive mask** replace the left shift of
  `lines` (`kt.rs:776-779`). They must keep the visit order; a test has to pin
  that, because the code tests positions (`iwin < 3`).
- **Per-lane graph narrowing** has no FlashJet counterpart. [code] Like
  MadGraph's `cluster.f`, a pair is admissible only if some diagram of the
  process still contains a line with that leg set. `graphs`, the diagrams
  consistent with every merge so far, is narrowed after each winning merge
  (`kt.rs:684-694`). The narrowing is an intersection of sorted lists of
  diagram indices (`MergeTable::narrow`, `graph.rs:159`), and the table
  `id_cl` maps a leg set to its diagrams (`graph.rs:96-98`). Lanes that merged
  different pairs hold different lists, so the next step's admissibility
  differs per lane. Two ways:
  - **A per-lane bitset (chosen by the user, 2026-10-27).** `graphs` becomes `⌈G/64⌉` `u64`
    words per lane, with `G` the process's diagram count, and `id_cl` a bitset
    per leg set. Narrowing is a word-wise AND, admissibility is "any word
    non-zero", and both are lane-wise integer operations with no branch. It
    needs an integer lane type beside `Index`.
  - **A scalar fallback on disagreement.** Run the batch in lanes while every
    lane picks the same winner, and unpack to scalar on the first
    disagreement. [inferred] The winner depends on each point's kinematics, so
    in a batch of random points the lanes disagree at the first step almost
    always once `n ≥ 4`. The fallback would then be the common path, and the
    lanes would buy little.
- **FMA.** ΔR² and the `kt` measures must use `mul_add_fast` consistently at
  every site, or lanes and scalar diverge on ties.

## The other consumers

- **VEGAS draw.** It needs no search: `k = (u·nbins).floor_index(nbins − 1)`,
  then two `gather`s. Accumulate stays scalar `f64`: a lane batch unpacks with
  `Index::lane` and `Real::lane` (seam design). Refine stays scalar `f64`, run
  once per iteration over bins; vectorise it only if a profile shows it matters.
- **Windows.** `windowed` returns `(lo, hi, Mask)`, and `outside_window`
  returns a `Mask`. A two-interval window selects its interval per lane.
- **Cuts.** `alive = alive & pass(p)` per cut, with `if !alive.any() { break }`,
  and the weight is `select(alive, w, 0)`.
- **PDF lookup.** A per-lane search. It can be a compare-and-count over the
  knots, or an analytic index where the grid is uniform in `ln x`. Left to
  [pdf-lookup-f64-only](../backlog/performance/pdf-lookup-f64-only.md).

## GPU, and the limits of designing for it

- [cuTile] Option 3's operations map one-to-one onto the tile model: `Mask` to
  `tile<i1>`, `select` to `select`, `any`/`all`/`uniform` to a reduction to 0-d
  followed by `if`/`break`, `gather` to a masked `gather`. cuTile has no
  `any`/`all` and no `searchsorted`; they are written as sum or max reductions.
- [cuTile] It is NVIDIA-only (CUDA 13.1+, compute capability 8.x–12.x). The
  Rust binding is early stage and needs nightly Rust. Tile shapes are powers of
  two, which suits `N ∈ {2, 4, 8}`. `float64` is supported, but f64 throughput
  on consumer GPUs is a small fraction of f32 [inferred].
- [cuTile] Reductions are deterministic per device but in an order the
  implementation chooses, and atomic and scatter order is unspecified. A GPU
  backend is therefore validated statistically, never bit-for-bit, consistent
  with AGENTS.md's "ULP exactness is never the target". The sampler's
  scalar-`f64` accumulate keeps the CPU paths exact.
- [inferred] The design targets the tile model in general, not cuTile. GPU
  portability supports option 3 but does not decide it. The decisive argument
  is correctness on `LaneField` today.

## Staging (for the user to choose)

- **A. Option 3 in one step.** Rewrite `Real` and classify the 95 comparison
  sites in one PR. It is byte-identical at `f64` and gated by the existing lane
  test. It is a large diff, mostly mechanical.
- **B. Two steps (chosen by the user, 2026-10-26).**
  1. Add the mask vocabulary as a companion trait implemented for `f64` and
     `LaneField`, and add the debug "uniform or panic" check to `LaneField`'s
     `PartialOrd`, `PartialEq` and `ToPrimitive`. This is small, catches misuse
     immediately, and lets windows, the VEGAS lane draw and cuts adopt masks.
  2. Fold the companion into `Real` and drop `Float`, `PartialOrd` and
     `NumCast`, converting the remaining comparison sites. The end state is the
     same as A.

Either way, the sampler refactor is independent: it is implemented at `f64`
through the trait's scalar parameter.

## Open questions

1. Staging: **B**, decided 2026-10-26.
2. Equality: **keep `Num` and `num_complex`**, decided 2026-10-27. `==` on
   `LaneField` is uniform-or-panic; the own complex type stays an option with
   its lift measured above.
3. Clustering's per-lane graph narrowing: **a per-lane bitset**, decided
   2026-10-27.
4. The index lane: `[u32; N]` (gather-friendly), or `usize` lanes?
5. `Mask::uniform` in release builds: lane 0's answer (no cost), or keep the
   check (one reduction per fork)?
