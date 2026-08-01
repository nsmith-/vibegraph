# x86 AVX2 evaluator perf study — results (branch `x86_avx2_perf`)

**Status: DRAFT — in progress.** Two of three planned changes landed and are measured;
the third (`get_unchecked` on the `fill_arenas` dispatch loop) is not yet done. Finish
in a fresh session, then flip this note to closed.

Host: AVX2 + FMA, no AVX-512 (`grep avx /proc/cpuinfo` → `avx avx2 fma`, `zmm`=0 in all
dumps). All builds `RUSTFLAGS="-C target-cpu=native"`, `--profile profiling`
(`inherits = release`, `debug = 1`).

Figure of merit: `eval_strategies` criterion bench, median over the 14-process ×
{`forward`, `lanes2`, `lanes4`, `lanes8`} grid (`forward` = scalar `f64`; `lanesN` =
`NumericArray<f64, N>` lane pack). Correctness net: `validate_helas_mg`
(`--features extended-validation`, 14/14 bit-exact-to-tolerance vs MadGraph) plus
`eval_m2_lanes_bit_identical_to_scalar` (SIMD == scalar, exact).

The study began from a review of `fill_arenas.asm` (the per-point typed-instruction
dispatch loop, `helas/eval/run.rs:566`). Three findings drove the work:

1. **No FMA emitted** despite an FMA host — LLVM never contracts `a*b + c` (FP
   contraction off by default); every complex multiply was `vmulpd` + `vaddsubpd`, every
   dot a `vmul`/`vadd` chain.
2. **Bounds checks everywhere** in the dispatch loop — every `arena[loc]` /
   `arena[operand]` access checked, including inside the `Add*` accumulation loops.
   ~118 `panic_bounds_check` trampolines. **Not yet addressed.**
3. **Inconsistent kernel inlining** — LLVM left the FFV/propagate/gamma-vout `*_bare`
   kernels as out-of-line calls; `validate_arenas` (a once-per-workspace one-shot) was
   `#[inline]` and bloating the hot frame.

## What landed

### Commit `1b47771` — inlining tune (finding 3)

Soft `#[inline]` on the six out-of-line `*_bare` kernels; `validate_arenas` →
`#[inline(never)]`. Under `extended-validation` the hot frame shrank 520 → 296 B and the
validation body (println/`resolve`/panic trampolines) moved out of line. Three of the
larger kernels (`ffv_fin`/`ffv_fout`/`gamma_vout`) inlined; the four biggest
(`propagate_{fin,fout,vector}`, `ffv_vout`, ~150–190 insns each) stayed calls even with
the hint — `#[inline]` raises the threshold but does not force.

Bench: scalar `forward` ~−4% mean (up to −10% on the two largest processes); narrow SIMD
neutral-to-slightly-slower (+4–5%), net ≈ 0. Kept because the mul_add work below was
expected to (and did) shift lane instruction pressure.

### FMA / `mul_add` (finding 1) — the headline

The un-fused arithmetic all lived in `helas/repr/lorentz.rs` leaf primitives, routed
through `num_complex`'s `core::ops::arith` operator impls (which LLVM can't contract
across). Converted the hot primitives to accumulate through the **real** fused
multiply-add `F::mul_add`:

- Two shared helpers `cmul(a,b)` / `cmul_add(a,b,c)` (complex `a*b`, `a*b+c` built from
  real `mul_add`).
- `ComplexVector::dot`, `dot_lorentz`, `Bispinor::slash_bispinor` (Ket + Bra),
  `left_current`, `right_current`, `scalar_bilinear`, and the real `p3_squared` / `m2`.

**Key constraint that fixed the whole approach.** `num_complex::Complex<T>: MulAdd`
requires `T: MulAdd<Output=T>`. `f64` has it, but the lane type `NumericArray<f64,N>`
implements only `Float::mul_add` (the *method*, via `Real: Float`) — **not** the
`num_traits::MulAdd` *trait*. So `Complex::mul_add` compiles for scalar but not for the
lanes, and adding `MulAdd` to the `Real` bound is an orphan-rule dead end (both foreign).
Routing through the real `mul_add` is the one path that fuses on **every** `F: Real`, and
using the same helper for scalar and lanes keeps `eval_m2_lanes` bit-identical to scalar.

This is exactly where the earlier lane analysis said the bottleneck was: the SoA lane
layout can't use the hardware `vaddsubpd` complex idiom and was doing complex cross-terms
as manual `vmul`/`vperm`/`vblend` storms. FMA both fuses the mul-adds **and** restructures
away the shuffle tax — lanes2 `vpermpd` 71→20, `vblendpd` 89→36.

**Bench (before = pre-FMA / post-inline HEAD, after = +FMA; median Δ%, 56 cells):**

| strategy | mean | median | range |
|---|---|---|---|
| forward (scalar) | +3.55% | +2.86% | −10% … +14% |
| lanes2 | **−34.05%** | −35.18% | −44% … −16% |
| lanes4 | **−23.73%** | −28.20% | −35% … −0.1% |
| lanes8 | **−35.40%** | −33.57% | −53% … −18% |
| **ALL** | **−22.41%** | −28.88% | −53% … +14% |

The lanes2 pessimal case (previously *slower than scalar*) is cured — now −34%. lanes2
and lanes8 are the biggest winners; best single cell −53% (`bbx_to_ccx` lanes8).

**Scalar forward regressed ~+3.5%.** The scalar path already had the tight packed
`vmulpd`+`vaddsubpd` complex idiom; the real-`mul_add` decomposition trades that 2-op
complex-mul for more scalar FMAs. `forward` is the least-used path and this is small, so
shipped as-is. Possible follow-up: specialize `cmul`/`cmul_add` to call `Complex::mul_add`
when the scalar is `f64` (packed form) and fall back to the real decomposition for lanes —
recovers the scalar path at the cost of a type-specialization mechanism. Not pursued.

### Correctness note — FMA exposed a latent test-fixture NaN

FMA makes `m2()` *more* accurate (single rounding), so hand-crafted "massless" test
momenta round to m² ≈ −1e-16 → `m()` = √(neg) = **NaN** → NaN comparisons silently took
the wrong branch in three unit tests (a 0.45% completeness "mismatch" and two setup
`assert!` panics). Production never calls `.m()` on such momenta (only `.m2()`, where the
tiny-negative is harmless). Fixed in the test fixtures by comparing `m2()` (NaN-safe, the
honest "massless within tolerance" intent) instead of `m()`. MG gate and the lane
bit-identical test both pass. (Three *other* failures are pre-existing baseline failures
from the first Linux run — RNG seed-hash pins + a boosted-frame test — unrelated.)

## Not yet done — `get_unchecked` on the dispatch loop (finding 2)

The `fill_arenas` dispatch loop bounds-checks every arena access; the indices come from
the compiled `prog` stream (correct by construction — exactly what `validate_arenas`
proves at debug/`extended-validation` time). Converting the `arena[loc]` /
`arena[operand]` accesses to `get_unchecked`/`get_unchecked_mut` behind a `// SAFETY:`
pointing at the validation invariant should:

- delete the per-iteration `cmp`/`jbe` in the `Add{Scalar,Vector,Fin,Fout}` inner loops
  (`LBB…_61`/`_97`/`_127`/`_152` in the dumps),
- collapse ~100 `panic_bounds_check` trampolines, shrinking the function for I-cache.

Expected to help **all** strategies roughly uniformly (unlike FMA, which traded scalar
against SIMD). **To measure next session:** save a fresh criterion baseline off the
current FMA HEAD, apply the unchecked conversion, re-bench the 56-cell grid, and re-run
the MG + lane bit-identical gates. Then close this note with the combined
end-to-end delta vs the pre-study baseline.

## Reproduce

```
RUSTFLAGS="-C target-cpu=native" cargo bench -p vibegraph-lib --bench eval_strategies -- --save-baseline <name>
RUSTFLAGS="-C target-cpu=native" cargo test -p vibegraph-lib --test validate_helas_mg --features extended-validation --profile profiling
RUSTFLAGS="-C target-cpu=native" cargo test -p vibegraph-lib --lib --profile profiling eval_m2_lanes_bit_identical_to_scalar
```
