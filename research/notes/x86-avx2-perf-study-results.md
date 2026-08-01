# x86 AVX2 evaluator perf study — results (branch `x86_avx2_perf`)

**Status: CLOSED (2026-07-31).** Two changes landed (inlining tune + FMA); the third
(`get_unchecked`) was implemented, measured, and then **reverted** — its ~2–3% did not
justify retiring the evaluator's 100%-safe-Rust invariant. It survives here as the
measurement that finally answers note 17's open question. Combined shipped delta vs the
pre-study baseline is in "Cumulative outcome" below.

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
   Prototyped as a narrowly-audited `unsafe` `get_unchecked` core (note 17 §7(b)'s
   escalation option), measured, and **reverted** — see below.
3. **Inconsistent kernel inlining** — LLVM left the FFV/propagate/gamma-vout `*_bare`
   kernels as out-of-line calls; `validate_arenas` (a once-per-workspace one-shot) was
   `#[inline]` and bloating the hot frame.

## What happened (2 shipped, 1 reverted)

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

### `get_unchecked` on the dispatch loop (finding 2) — measured, then REVERTED

The `fill_arenas` dispatch loop bounds-checked every arena/pool access; the indices come
from the compiled `prog` stream, correct by construction — `Program::build` draws every
result slot (`loc[id]` and every `OperandRef` index) from `0..arena_sizes[class]` by
liveness allocation, `ensure_sizes` grows each arena to exactly `arena_sizes`, and
`resolve_moms` fills `moms` to `mom_table.len()` (the sole source of every momentum id).
`validate_arenas` re-proves this under debug/`extended-validation`.

**Outcome: NO-GO, reverted.** This was note 17 §7(b)'s escalation option — that memo
cancelled the A3c bounds-check work under a *safe-only* charter (its §9 recorded "the
evaluator stays 100% safe Rust") but left "(b) re-scope to a narrowly-audited `unsafe`
core … behind the full `validate_helas_mg` bit-for-bit gate" open. It was implemented and
measured here (numbers below), then reverted: the shipped delta is ~2–3% on the scalar
path and neutral on the SIMD paths, which does not justify retiring the 100%-safe-Rust
invariant. The evaluator stays entirely safe. The mechanism and bench are recorded so the
question note 17 left open — *what does the `unsafe` core actually buy on x86?* — is now
answered with data rather than the arm64 probe's ceiling, and a future decision can weigh
the real number, not a projection. What follows is the prototype's findings, not shipped
code.

**Prototype shape.** One getter/setter pair per arena class on `ScratchSpace`
(`scalar`/`set_scalar`, `vector`/`set_vector`, …) plus `real`/`mom`, each a one-line
`get_unchecked{,_mut}` under a single shared `// SAFETY:` doc; the dispatch loop body is
one `unsafe` block that carries none of its own. `unsafe` is confined to `run.rs` (11
accessors + 1 block) — the rest of the library stays safe. **Critical subtlety the bench
caught:** the composite reads (`vector`/`fin_at`/`fout_at`/`mom`) must return a *reference*,
matching the original `&arena[i]` borrow. A first cut returned them *by value*; for scalar
`f64` that copy is free, but for `NumericArray<f64,8>` a `ComplexVector` is 512 B, so every
operand read became a memcpy → a uniform **+60% lanes8 regression** (scalar unaffected).
Returning `&T` restored the zero-copy borrow and the regression vanished.

**Mechanism — note 17's symbol-count confirmation does *not* reproduce here.** On rustc
1.97 / x86 (`--profile profiling`), LLVM already *outlines the panic calls*: the checked
`fill_arenas<f64>` shows **0** `panic_bounds_check` symbols even at baseline (note 17's
arm64 build had 57). But the inline bounds *compare+branch* pairs are still there, and
`get_unchecked` removes those — direct evidence from the lanes2 monomorphization body:

| `fill_arenas` (lanes2, profiling) | insns | cond. branches |
|---|--:|--:|
| PRE (checked) | 3677 | 157 |
| POST (unchecked) | 2867 | 52 |

−810 instructions, −105 conditional branches (−22% body size). So the transform is real;
it just isn't visible by counting panic trampolines on this toolchain.

**Bench (before = FMA HEAD `3dab3a1` / baseline `pre_unchecked`, after = +unchecked;
median Δ%, 56 cells):**

| strategy | mean | median | min | max |
|---|---|---|---|---|
| forward (scalar) | **−2.84%** | −2.80% | −7.94% | +0.60% |
| lanes2 | −0.83% | −1.40% | −4.37% | +6.36% |
| lanes4 | **−2.98%** | −3.53% | −10.21% | +2.11% |
| lanes8 | −0.15% | −0.15% | −4.80% | +5.93% |
| **ALL** | **−1.70%** | −1.81% | −10.21% | +6.36% |

Modest and consistent — well below note 17's arm64 +7–11% *ceiling*, for two structural
reasons: (a) that ceiling was a *coupled* reads+writes effect and its write half was the
`Vec::push` capacity-check family, which the current pre-sized direct-index writes had
already eliminated (only the read checks remained to remove here); (b) the removed branches
were near-perfectly predicted (never-taken), so deleting them buys I-cache/scheduling, not
mispredict recovery. The prototype was provably value-preserving throughout — MG gate 14/14
bit-exact (identical `max_rel_diff` to baseline) and `eval_m2_lanes_bit_identical_to_scalar`
exact — so the decision is purely cost/benefit, not correctness. **The ~2–3% is not worth
an `unsafe` block in the amplitude core**: the scalar recovery it offers can be revisited
if the scalar path ever becomes hot enough to matter (event generation's single-helicity
regime, `mg-single-helicity-bench`), and the FMA change above already fixed the SIMD paths
that were the study's actual motivation. Reverted; the evaluator remains 100% safe Rust.

## Cumulative outcome (whole study, `x86_avx2_perf` branch)

Two shipped changes vs the pre-study `x86_avx2_perf` base (`90fe612`, the 14-process bench
extension): **inlining tune** (`1b47771`) → **FMA/`mul_add`** (`3dab3a1`). The third
(`get_unchecked`) was measured and reverted (above), so it contributes nothing to the
shipped state. The arithmetic (FMA) is the whole story:

- **lanes2 / lanes4 / lanes8**: FMA's **−24% … −35%** median is the headline — the SoA lane
  paths, which the `hadronic-xsec` H4 session had written off as a SIMD negative result,
  are now ~¼–⅓ faster than the pre-study base on this AVX2 host.
- **forward (scalar)**: FMA cost it ~+3.5% (it traded the packed `vmulpd`+`vaddsubpd`
  complex idiom for more scalar FMAs). Left as-is — `forward` is the least-used path, and
  the `get_unchecked` recovery that would have offset it was reverted with the rest.

Net shipped: the SIMD lane paths are substantially faster than the pre-study base, the
scalar path is ~3.5% slower, and the evaluator stays entirely safe Rust. All behind 14/14
bit-exact `validate_helas_mg` + exact lane-identity throughout.

## Reproduce

```
RUSTFLAGS="-C target-cpu=native" cargo bench -p vibegraph-lib --bench eval_strategies -- --save-baseline <name>
RUSTFLAGS="-C target-cpu=native" cargo test -p vibegraph-lib --test validate_helas_mg --features extended-validation --profile profiling
RUSTFLAGS="-C target-cpu=native" cargo test -p vibegraph-lib --lib --profile profiling eval_m2_lanes_bit_identical_to_scalar
```
