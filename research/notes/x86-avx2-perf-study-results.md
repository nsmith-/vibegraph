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

### Commit `cf3f35b` — inlining tune (finding 3)

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

**Bench (before = FMA HEAD `be76771` / baseline `pre_unchecked`, after = +unchecked;
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

Two shipped changes vs the pre-study `x86_avx2_perf` base (`b99bb40`, the 14-process bench
extension): **inlining tune** (`cf3f35b`) → **FMA/`mul_add`** (`be76771`). The third
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

## ARM (M3 Max) results

**Status: the two shipped x86 changes re-measured on arm64, plus the transpose
isolation the x86 study left undone.** Host: Apple M3 Max (aarch64, NEON, no SVE),
macOS 24.6. Builds `RUSTFLAGS="-C target-cpu=native"`; the criterion bench profile
inherits `release` (fat LTO), the gates run `--profile release-debug` (thin LTO,
`debug = 1`) — `profiling` no longer exists as a profile, `release-debug` replaced it.

Same figure of merit as above: `eval_strategies`, 14 processes ×
{`forward`, `lanes2`, `lanes4`, `lanes8`}, criterion medians.

**One measurement correction to the x86 sections.** The bench does *not* time
`lanes{N}` per batch of `N` events. `bench_lanes` iterates `pts.chunks_exact(N)`
over all 16 points, so every strategy performs exactly **16 events per criterion
iteration** and the bars are already directly comparable per event — no
normalisation, and `lanes{N}` ÷ `forward` is the per-event throughput ratio as it
stands.

### Δ% of the two shipped changes (inlining tune + FMA), 56 cells

| process | fwd | lanes2 | lanes4 | lanes8 |
|---|--:|--:|--:|--:|
| `bbx_to_ccx_emmm_qcd0` | -9.7% | +22.0% | -25.1% | -23.0% |
| `ee_to_ee` | -0.1% | +19.1% | -17.4% | -15.0% |
| `ee_to_mumu` | +15.4% | +16.9% | -17.5% | -14.4% |
| `ee_to_mumu_tata_qcd0` | -14.9% | +25.1% | -22.6% | -20.0% |
| `ee_to_mumua` | -4.1% | +24.1% | -19.9% | -17.3% |
| `ee_to_tatah` | +0.3% | +19.5% | -18.7% | -16.5% |
| `ee_to_ttx` | +1.1% | +18.6% | -17.5% | -14.1% |
| `ee_to_wpwm` | +3.5% | +16.1% | -13.3% | -9.9% |
| `ee_to_zh` | +36.6% | +10.5% | -11.1% | -7.2% |
| `gg_to_gg` | +3.7% | +10.5% | -16.6% | -11.9% |
| `gg_to_ttx` | -1.5% | +21.0% | -22.9% | -19.1% |
| `pp_to_ll_qcd0` | +16.4% | +16.5% | -17.4% | -14.1% |
| `uux_to_ccx_emmm_qcd0` | -10.4% | +24.4% | -22.4% | -21.4% |
| `uux_to_uux` | +23.0% | +16.3% | -19.4% | -15.8% |

| strategy | mean | median | range | suite total |
|---|---|---|---|---|
| forward (scalar) | +4.23% | +0.65% | −14.9% … +36.6% | **−9.72%** |
| lanes2 | **+18.61%** | +18.83% | +10.5% … +25.1% | **+22.85%** |
| lanes4 | −18.70% | −18.12% | −25.1% … −11.1% | −23.86% |
| lanes8 | −15.70% | −15.43% | −23.0% … −7.2% | −22.17% |
| `set_alpha_s` | −0.50% | −0.54% | −1.0% … +0.6% | — |

"suite total" is Σ(medians) before vs after, i.e. the process-cost-weighted delta.
It matters here: the unweighted mean spans processes from 4 µs to 3.7 ms per
iteration, so it is dominated by the cheapest cells. Weighted by actual cost the
scalar path **improves 9.7%** on ARM — the three most expensive processes
(`bbx_to_ccx` −9.7%, `uux_to_ccx` −10.4%, `ee_to_mumu_tata` −14.9%) all win, and
every cell above +15% is a sub-10 µs process.

**`lanes2` is a real, uniform ARM regression**: +18.6% mean, all 14 processes
between +10.5% and +25.1%. It is the one place ARM inverts the x86 result, where
lanes2 was the biggest single winner (−34%). `lanes4`/`lanes8` reproduce the x86
direction at roughly half the magnitude.

Run-to-run noise floor for this bench, measured from two consecutive runs of
byte-identical code paths (`forward` is untouched by the prepacked bench work, yet
moved): **≈1–1.5%**. Everything above is well outside it; the transpose numbers
below are at or under it.

### `lanes{N}` ÷ `forward` per-event throughput — lanes lose on ARM, badly

Per-iteration medians are 16 events on every bar, so these ratios are per-event.

| baseline | lanes2 | lanes4 | lanes8 |
|---|--:|--:|--:|
| pre (main), mean | 2.47× | 8.02× | 5.77× |
| pre (main), median | 2.42× | 7.72× | 5.64× |
| post (merged), mean | 2.88× | 6.39× | 4.75× |
| post (merged), median | 2.79× | 6.23× | 4.69× |

Greater than 1 is *slower than scalar*. ARM shows the same signature the x86 study
reported — lanes do not beat scalar per event — but not marginally: the best lane
width is 2.4–2.9× slower than `forward`, and `lanes4` is worse than `lanes8` on
both baselines. The FMA work moves the ratios in the right direction for widths 4
and 8 and the wrong direction for width 2, and never comes close to 1.

### The AoS→SoA transpose, isolated (this is new)

`eval_m2_lanes` is now `pack_lane_points` (the transpose, allocation included)
followed by `eval_m2_lanes_packed`; `bench_lanes_prepacked` hoists the first out of
the timed region and calls the second, so `lanes{N}` − `lanes{N}_prepacked` prices
the transpose directly. The existing `lanes{N}` cells are unchanged.

| process | lanes2 | lanes4 | lanes8 |
|---|--:|--:|--:|
| `bbx_to_ccx_emmm_qcd0` | +0.36% | +0.19% | -0.20% |
| `ee_to_ee` | +0.34% | +0.18% | +0.06% |
| `ee_to_mumu` | +0.85% | +0.39% | +0.50% |
| `ee_to_mumu_tata_qcd0` | -0.09% | -0.08% | -1.18% |
| `ee_to_mumua` | +0.41% | +0.22% | -0.01% |
| `ee_to_tatah` | -0.55% | +0.56% | +1.19% |
| `ee_to_ttx` | +2.38% | +0.49% | +0.06% |
| `ee_to_wpwm` | +0.24% | +0.08% | -0.19% |
| `ee_to_zh` | +1.81% | +1.07% | -0.43% |
| `gg_to_gg` | +0.72% | +0.22% | -0.03% |
| `gg_to_ttx` | +0.27% | +0.04% | +0.05% |
| `pp_to_ll_qcd0` | +1.27% | +0.71% | +0.16% |
| `uux_to_ccx_emmm_qcd0` | -0.01% | -0.21% | -0.35% |
| `uux_to_uux` | +1.09% | +0.97% | -0.40% |
| **mean** | **+0.65%** | **+0.35%** | **−0.06%** |

**The transpose is small — confirmed, not assumed.** Mean share is ≤0.65% at every
width, several cells sit below zero, and the whole table is inside the ~1–1.5%
noise floor. In absolute terms it is a width-independent per-event cost: on
`ee_to_mumu` (4 legs) the gap is 7.0 / 7.2 / 6.9 ns per event at N = 2 / 4 / 8, i.e.
~14 / 29 / 55 ns per chunk — one `Vec` allocation plus `n_ext × 4` lane packs. It
cannot account for a 2.4–8× per-event deficit, so **the transpose is exonerated and
the lane stall has to be found elsewhere.**

### Where the time actually is (samply)

`samply record` on the fat-LTO bench binary, `--profile-time 20`, one benchmark each,
self-time per symbol (`--main-thread-only`, presymbolicated):

`forward/uux_to_ccx_emmm_qcd0` — 21 220 samples:

| self | symbol |
|--:|---|
| 93.44% | `helas::eval::run::fill_arenas` |
| 2.65% | `__psynch_cvwait` (criterion's own thread) |
| 0.72% | `hashbrown … reserve_rehash` |

`lanes8/uux_to_ccx_emmm_qcd0` — 21 109 samples:

| self | symbol |
|--:|---|
| 41.70% | `helas::eval::run::fill_arenas` |
| 19.67% | `<num_complex::Complex<T> as core::ops::arith::Mul>::mul` |
| 12.23% | `numeric_array::…<impl Float for NumericArray<T,N>>::mul_add` |
| 7.47% | `<num_complex::Complex<T> as core::ops::arith::Neg>::neg` |
| 7.39% | closure body reached through `FnMut::call_mut` |
| 4.77% | `_platform_memmove` |
| 2.59% | `__psynch_cvwait` |

Read across the two: the **scalar** monomorphisation is one flat loop — every
`*_bare` kernel and every leaf arithmetic primitive is inlined into `fill_arenas`,
which owns 93% of wall time and nothing else registers. The **lane**
monomorphisation is not: LLVM leaves `Complex::mul`, `NumericArray::mul_add` and
`Complex::neg` as **out-of-line calls**, and ~39% of lanes8 wall time is spent in
them, with another 4.8% in `memmove` shuffling the 128-byte
`NumericArray<f64,8>`-based temporaries into and out of those calls. Neither
`pack_lane_points` nor `transpose_points` appears anywhere in the profile, matching
the ≤0.65% measured above.

So on ARM the lane deficit is an **inlining failure in the lane monomorphisation**,
not the transpose and not the typed-instruction dispatch. Two concrete follow-ups
the profile hands to any future evaluator session:

- `Complex::mul` at 19.7% is *instruction-level*, not leaf-level: the FMA work
  converted the `lorentz.rs` primitives, but `fill_arenas`'s own
  `Instr::MulScalarC` / `Instr::Scale*` arms still multiply through
  `num_complex`'s operator impls. Those are the remaining un-fused, un-inlined
  complex multiplies, and they are the single largest non-`fill_arenas` bucket.
- The lane path pays a call plus two memory round-trips per complex multiply and
  per FMA. Until those inline, no lane width can approach the scalar path, and
  comparing lane widths to each other measures call overhead rather than SIMD.

Profiles: `s2_arm_forward_uux.json.gz`, `s2_arm_lanes8_uux.json.gz` (+ `.syms.json`
sidecars), recorded with
`samply record --save-only --main-thread-only --unstable-presymbolicate -o <out> -- \
 target/release/deps/eval_strategies-<hash> --bench --profile-time 20 <filter>`.

### Correctness

Post-merge, on ARM, with the `pack_lane_points` / `eval_m2_lanes_packed` split in
place:

- `amplitude_oracle` (`--features extended-validation --profile release-debug`):
  **20 passed, 0 failed** — the MG amplitude net, now 20 enforced processes rather
  than the 14 the x86 sections cite. Worst residual is
  `uux_to_ccx_emmm_qcd0` per-event `1.78e-11`, which the test's own ULP criterion
  attributes to the point's conditioning (4.5–6.3× the one-ulp momentum
  perturbation).
- `validate_helas` (HELAS kernels vs the Fortran reference): 1 passed.
- `eval_m2_lanes_bit_identical_to_scalar`: passed, exact.

The FMA change is **reassociating**, not order-preserving — `mul_add` collapses two
roundings into one, so results move at the last ulp by construction; it gates at the
MG net's tolerances, never bit-for-bit against the pre-change output. The
`pack_lane_points` / `eval_m2_lanes_packed` split is **order-preserving** (the same
two operations, same order, one call boundary moved) and the lane-identity test
holds exactly.
