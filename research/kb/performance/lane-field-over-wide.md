---
type: Design Decision
title: "LaneField<N> as a newtype over wide"
description: "The SIMD lane field is a wide::f64xN newtype implementing num_traits::Float because numeric-array arithmetic would not inline; alternatives weighed, and the lane exactness contract."
status: draft
tags: [performance, simd, lanes, wide, inlining]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
measured:
  - {commit: 02e8b25, host: "Intel Xeon Emerald Rapids (family 6 model 207), 4-vCPU Firecracker VM", command: "RUSTFLAGS='-C target-cpu=native' scripts/dump_lane_asm.sh 'fill_arenas'; cargo bench --bench eval_strategies"}
  - {host: "Apple M3 Max, macOS", command: "samply record on the fat-LTO eval_strategies binary, --profile-time 20"}
sources:
  - {id: x86-arm-samply, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L325-L377", title: "x86 study, ARM results: where the time actually is (samply)"}
  - {id: x86-disasm, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L457-L557", title: "x86 study, AVX-512: the disassembly, the force-inlining probe, what this corrects"}
  - {id: x86-fix, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L569-L650", title: "x86 study, AVX-512: the fix, LaneField<N> over wide"}
  - {id: x86-relaxed, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L669-L699", title: "x86 study: relaxed scalar multiply-add"}
---

# `LaneField<N>` as a newtype over `wide`

**Decision.** The SIMD lane field the evaluator runs on for batched evaluation is
`LaneField<N>` (`vibegraph-lib/src/helas/eval/lane_field.rs`), a newtype over `wide`'s
`f64x2` / `f64x4` / `f64x8` that implements `num_traits::Float` and `FloatConst` itself,
so every `F: Real` code path runs on it unchanged. It replaced a `NumericArray<f64, N>`
alias from the `numeric-array` crate.[^x86-fix] How lanes are used is
[lane-batched evaluation](simd-lane-evaluation.md); what they buy per host is
[lane throughput](lane-throughput.md).

## Why: the previous lane field did not inline

The `NumericArray` field lost to scalar per event on every host and width (median 5.6× /
7.9× / 10.0× slower at N = 2 / 4 / 8 on Emerald Rapids; 2.4–2.9× at best on the M3 Max).
Two independent readings located the cause in the field's arithmetic, not the vector
width, the transpose or the dispatch:

- **Profile (M3 Max).** The scalar `fill_arenas` owned 93% of `forward` time with every
  kernel inlined. At lanes8, `fill_arenas` was 42%, and `Complex::mul` (19.7%),
  `NumericArray`'s `Float::mul_add` (12.2%) and `Complex::neg` (7.5%) ran as out-of-line
  calls, with 4.8% more in `memmove` shuttling 128-byte temporaries in and out of
  them.[^x86-arm-samply]
- **Disassembly (Emerald Rapids, `02e8b25`).** The widest lane monomorphisations held 1–6
  FP instructions among 25–31 k, and about 1 250 out-of-line arithmetic calls:
  `GenericArray::from_iter` (769, every elementwise op), `NumericArray` `mul_add` (245),
  `clone` (124), `Complex::mul` (92). Each callee was correctly vectorised but read
  operands through pointers, wrote its result to memory and ended in `vzeroupper`;
  `from_iter` ran an unrolled *scalar* loop with a length check per element.[^x86-disasm]

The attributes explain it: `numeric-array` 0.6.1's operators are `#[inline(always)]` but
forward to `GenericArray::zip`/`map` → `FromIterator::from_iter`, a soft `#[inline]` in
`generic-array` 1.4.4, and `num_complex`'s operators are soft `#[inline]` too. None of
these crates is ours, and `#[inline(always)]` on a caller does not propagate into callees,
so **no attribute in this tree could force the inlining.** A global
`-C llvm-args=-inline-threshold=2000` probe took lanes2/lanes4 from 5.6× / 7.9× to
1.65× / 1.57× of scalar but still left lanes8 at 6.5× with 75 `memcpy`s: inlining was most
of the story, and a global codegen flag is a probe, not a fix.

The lane-width readings taken under `NumericArray` (including the AVX2 study's "FMA makes
lanes 24–35% faster" and "lanes4 benefits least") measured call and copy overhead, not
SIMD, and say nothing about whether lanes pay. They are superseded.

## Alternatives weighed

| candidate | fit with `Real = Float + FloatConst + Copy + …` | verdict |
|---|---|---|
| **`wide` 1.7** | statically dispatched on the enabled vector units, on stable; each op one intrinsic, so even soft-`#[inline]` bodies inline; has exactly `f64x2/4/8`; `f64x8` uses zmm whenever `avx512f` is on, bypassing LLVM's `prefer-256-bit`; no `num_traits` impls | chosen, newtype supplies the impls |
| `fearless_simd` 1.0, `pulp` | built for runtime multiversioning: each vector carries a SIMD-level token, so `Float`'s token-less constructors (`zero()`, `NumCast::from`, `FloatConst::PI()`) have nothing to build from | would restructure the evaluator around their dispatch entry points; runtime CPU dispatch is a separate decision |
| `simba` | own `SimdRealField` trait family, no `num_traits::Float` | would rewrite the `Real` bound across the library |
| `std::simd` | nightly only | the repository builds on stable |

## What the newtype does

- `+ − × ÷`, `neg`, `sqrt`, `abs` and the multiply-add are the packed `wide` operation;
  every other `Float` method runs per lane through `f64`'s own method.
- The packed operations are the IEEE-exact ones (correctly rounded `+ − × ÷ sqrt`,
  sign-bit `abs`/`neg`), so each lane is bit-identical to the scalar `f64` computation.
- **The multiply-add.** The evaluator calls `Real::mul_add_fast` on both sides, never
  `Float::mul_add` ([`mul_add_fast`](mul-add-fast.md)). On `LaneField` it is a packed
  hardware FMA where `HARDWARE_FMA` holds (x86 with `fma`, aarch64) and a packed product and
  sum otherwise, the same operations scalar `f64` performs under that method, so lanes and
  scalar round alike on every target. `LaneField`'s own `Float::mul_add` keeps the
  single-rounding contract on every target (a software FMA per lane without hardware
  FMA); nothing on the evaluation path calls it.
- Comparisons and predicates keep the pack-level semantics the lane-uniformity contract is
  written against: `==` when every lane is equal, lexicographic `<`/`>`/`partial_cmp`,
  `is_nan`/`is_infinite`/`is_sign_negative` if any lane holds, `is_finite`/`is_normal`/
  `is_sign_positive` only if all do; conversion to a primitive reads lane 0. The module doc
  of `lane_field.rs` is the authority.

**Exactness contract, current.** Every extracted lane equals the scalar `eval_m2` bit for
bit on every target, pinned by `eval_m2_lanes_match_scalar` and `lanes4_lanes8_match_scalar`
(`assert_eq!` on `to_bits`, `vibegraph-lib/src/helas/eval/run.rs`). Unit tests pin every
packed op bit for bit against `f64` at N = 2 / 4 / 8, with the multiply-add pinned to
whichever semantics `HARDWARE_FMA` claims, so the flag cannot drift from `wide`'s own
per-width condition. Bit equality holds between builds that agree on `HARDWARE_FMA`;
across that flag results agree to rounding. There is no baseline-x86-64 exception: both fields take the multiply-add through
`mul_add_fast`, so the code has neither the relative `1e-10` lane tolerance
(`LANE_UNFUSED_REL_TOL`) nor the `FUSED_MUL_ADD` flag that the study record describes.[^x86-relaxed]

## Census of the result

`dump_lane_asm.sh 'fill_arenas'`, `target-cpu=native` on Emerald Rapids at `02e8b25`:

| instance | insns | packed pd (xmm / ymm / zmm) | scalar sd | arith calls | memcpy |
|---|--:|--:|--:|--:|--:|
| lanes2 | 5 467 | 1 742 / 0 / 0 | 0 | 0 | 1 |
| lanes4 | 5 738 | 0 / 1 734 / 0 | 0 | 0 | 1 |
| lanes8 | 6 090 | 0 / 0 / 1 734 | 0 | 0 | 11 |

Every lane op inlines, each width lands on the register class it should, lanes8 is genuine
8×f64 AVX-512, and the bodies shrank 2–5×. Lanes then beat scalar at every width on every
row (median 0.57× / 0.32× / 0.25× of scalar per event at N = 2 / 4 / 8). Scalar `forward`
was unchanged within noise.

`dump_lane_asm.sh` guards the blind spot this exposed (counting packed ops per function
censuses out-of-line leaves as packed): it reports `calls` /
`arith_calls` per function and an inlining verdict, and reads its width verdict from packed
arithmetic on zmm rather than any zmm use.

## Open

ARM has no measurement under `LaneField`
([backlog](../backlog/performance/lane-field-unmeasured-on-arm.md)); on aarch64 `wide` is
NEON `f64x2` and wider packs are pairs of it. Lane evaluation is not used in production
([backlog](../backlog/performance/lane-batching-not-in-production.md)).

[^x86-fix]: x86 study, "The fix: `LaneField<N>` over `wide`", at `02e8b25`.
[^x86-arm-samply]: x86 study, ARM results, "Where the time actually is".
[^x86-disasm]: x86 study, AVX-512 re-measurement: disassembly, force-inlining probe, corrections.
[^x86-relaxed]: x86 study, "Relaxed scalar multiply-add"; current code in `vibegraph-lib/src/helas/eval/lanes.rs` and `lane_field.rs` module docs.
