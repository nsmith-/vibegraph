---
type: Design
title: "Float reassociation in the Lorentz kernels"
description: "Serial accumulation chains in dot/dot4/dot_lorentz/contract* are split into two-chain or tree forms by hand; f64::algebraic_* was rejected because results then depend on the inliner."
status: draft
tags: [performance, floating-point, kernels, latency, benchmarks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
measured:
  host: "Intel Xeon Emerald Rapids (family 6 model 207), 4-vCPU Firecracker VM, one pinned core"
  command: "cargo bench -p vibegraph-lib --bench lorentz_kernels (A/B/A/B, native and default targets); eval_strategies three interleaved rounds per variant"
sources:
  - {id: alg, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L700-L748", title: "x86 study: algebraic float arithmetic (rustc 1.98), not adopted"}
  - {id: kb, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L749-L834", title: "x86 study: kernel microbenchmarks (benches/lorentz_kernels.rs)"}
---

# Float reassociation in the Lorentz kernels

The leaf kernels in `vibegraph-lib/src/helas/repr/lorentz.rs` write their accumulations
so that independent partial sums can run in parallel, rather than as one serial chain
from the first term to the last. The reassociation is done by hand, in the source, with
each multiply-add through [`Real::mul_add_fast`](mul-add-fast.md). Letting the compiler
reassociate (`f64::algebraic_*`) was measured and rejected.

## The kernel shapes

```rust
/// Two independent accumulation chains, joined by one add, keep the dependent path
/// to about five operations instead of the eight of a single chain.
fn dot4<F: Real>(a: [C<F>; 4], b: [C<F>; 4]) -> C<F> {
    let even = cmul_add(a[2], b[2], cmul(a[0], b[0]));
    let odd = cmul_add(a[3], b[3], cmul(a[1], b[1]));
    even + odd
}
```

| kernel | form |
|---|---|
| `ComplexVector::dot`, `dot4` | two chains (even and odd components) joined by one add; `dot4` also serves the Weyl-matrix rows of `apply` and `clifford_product` and `epsilon4`'s final contraction |
| `dot_lorentz` | each real sum (re and im) split into two partials joined by one subtraction, metric signs inline |
| `scalar_bilinear(Both)` | `dot4` instead of four chained multiply-adds |
| `AsymRank2Tensor::contract` | one independent partial per spatial axis |
| `contract_vector` | written out over each row's three non-zero entries; a loop over all four columns included a multiply-add by the zero diagonal, which the compiler cannot drop |
| `contract_vectors` | `a · (T b)` on `contract_vector`, not a six-step fold |
| `fierz_pairing` | four grade terms added as a tree |

Already flat and unchanged: the chiral currents, `tensor_bilinear`, the `epsilon_cofactors`
minors, `to_weyl_matrix` and `from_weyl_matrix`.

## How the change was judged

A leaf-kernel change is usually below the resolution of the 8-row `eval_strategies` bench
(code layout alone moves it by up to ~10% on the Emerald Rapids VM), so it is judged on the
kernel itself with `benches/lorentz_kernels.rs`.[^kb] That bench times the production
kernels through their public API at `f64` and `LaneField<4>` in two shapes:

- `throughput`: 1024 independent calls, the shape the dispatch loop mostly presents;
- `chain`: each result feeds the next call's input, so the time is critical-path latency.
  Each chain is built to stay O(1), and the bench asserts it never reaches overflow or
  subnormals. A `chain` win is a kernel property, not a promised end-to-end speedup,
  because the dispatch loop hides latency behind independent work.

Protocol: A/B/A/B over three rounds on one pinned core, native (AVX-512) and default
(SSE2, no FMA) targets. `slash` is unchanged code between the builds and moves −8% to +6%:
that is the noise floor.

**`dot` as two chains** (per-round change, B vs A): chain −18% to −21% at `f64` on both
targets and at lanes4 native; throughput −4% to −6% native `f64` (−2% to −4% native lanes4),
−13% to −18% on the default target `f64`; default-target lanes4 within noise. Adopted on these kernel merits although the
8-process bench showed a null.

**The full rewrite** (range of per-round change, throughput / chain, %):

| kernel | native f64 | native lanes4 | default f64 | default lanes4 |
|---|---|---|---|---|
| `dot_lorentz` | −1…+1 / −10…−7 | −2…+1 / −9…−7 | −2…+3 / −9…−6 | −5…+3 / −2…−0 |
| `scalar_bilinear` | −4…−1 / −17…−14 | −5…+5 / −20…−16 | −19…−14 / **+14…+23** | −2…+3 / +1…+9 |
| `tensor_contract_vectors` | −22…−18 / −28…−5 | −21…−17 / −24…−19 | −23…−21 / −24…−22 | −32…−25 / −17…−16 |
| `tensor_contract_vector` | −40…−34 / −21…−20 | −25…−23 / −20…−16 | −29…−29 / −43…−39 | −50…−46 / −49…−45 |
| `fierz_pairing` | −4…−3 / −36…−33 | −15…+7 / −3…+3 | −2…+2 / −6…+3 | −7…+8 / −5…+6 |
| `apply` | −2…+4 / −3…+2 | +1…+10 / −5…+6 | −5…−3 / +0…+7 | −1…+7 / −4…+5 |
| `slash` (control) | −1…+4 / −6…−2 | −5…+5 / −3…+4 | −4…+3 / −1…+2 | −8…+6 / +1…+6 |

- **Clear wins**: `contract_vector` and `contract_vectors` do less work as well as having
  shorter paths; `dot_lorentz` gains 7–10% in chain except default lanes4.
- **Outliers re-run**: round 3 on native had three disturbed cells (`epsilon4` chain +60%,
  `tensor_contract` chain +36%). A native-only A/B/C rerun, C reverting only `contract`,
  showed `tensor_contract` chain −6% (`f64`) and −12% (lanes4) consistently, throughput
  unchanged, and `fierz_pairing`'s `f64` chain 45 → 30 µs with the new `contract` (its
  critical path runs through the bivector term).
- **The one regression**: `scalar_bilinear`'s default-target `f64` chain is 14–23% slower,
  while that target's throughput gains 14–19% and native's chain gains 14–20%. Without
  hardware FMA, `mul_add_fast` is a multiply and an add, so a serial chain's dependency from
  the accumulator is already one add per term and the split buys less.
- **Neutral**: `apply` (its rows were already independent) and `epsilon4`.

All rewrites are kept: the throughput shape, which is what the dispatch loop mostly
presents, improves or holds in every cell except native-lanes4 `apply` and
`tensor_contract`, both within noise. The amplitude oracle's residuals moved only in their
last printed digits, and the lane-vs-scalar tests stayed exact (the same reassociation runs
on both fields).

## Compiler reassociation (`f64::algebraic_*`) was rejected

rustc 1.98's algebraic float operations let LLVM reassociate and contract freely. A
feature-gated study hook (explicit `Real` impls for `f32`, `f64` and `LaneField` in place of
the blanket one, so `f64` could override kernel arithmetic) tried three variants:
v1 multiply-adds only; v2 every op in `cmul`/`cmul_add`/dots/`slash_bispinor`/`m2`; vw v2
plus the vector-space macros and the dispatch loop's `AddScalar`/`MulScalar*`.[^alg]

Median `forward` effect over the 8 rows (positive = slower; control drift 1–9% per row):

| target | v1 | v2 | vw |
|---|--:|--:|--:|
| native (AVX-512) | +9.8 … +17.7% on 6 of 8 rows | +3.6 … +14.2% | −2.2 … +9.0%, mostly ≈ 0 |
| `x86-64-v3` | −3.5 … +7.3% | −6.8 … +2.5% | −1.7 … +5.6% |
| default | −4.8 … +2.3% | −2.1 … +5.2% | −4.2 … +1.3% |

- **Why v1 is slower**: given algebraic mul and add, the SLP vectoriser packs each complex
  product's re/im pair into `vmulpd` + `vaddsubpd` + shuffles before FMA formation sees it.
  In `fill_arenas` on native, scalar FMAs go 534 → 172, `vaddsubpd` 35 → 98, `vshufpd`
  82 → 162; on this core the packed-complex form is slower than the FMAs it replaced.
  v2 and vw bring the FMAs back and trim 3–5% of static instructions, but no time.
- **What it broke** (under vw): `eval_m2_lanes_match_scalar` and `lanes4_lanes8_match_scalar`
  failed on the default and v3 targets (69–73 of 118 lane comparisons differ, worst 1 265
  ulp, 1.8e-13 relative), and `test_fierz_reconstruction` failed on v3: the same bilinear
  through `fierz_coefficients().scalar()` and `scalar_bilinear()` differed (`im` 2.6e-18 vs
  0) because two inlining contexts contracted differently. The amplitude oracle stayed green
  (42/42) at the control's residual scale, and the helicity-expansion, alternative-schedule,
  batched-VEGAS and fixed-seed bit-identity tests passed: only the lane-vs-scalar and
  two-path comparisons see the drift.
- **Noise**: lane code was census-identical between control and vw yet moved −2.8% to +9.7%
  between builds, so single-digit effects in this bench are not resolvable.

Verdict: no speedup to pay for results that depend on the inliner. Hand reassociation keeps
every rounding decision in the source, identical for scalar and lanes. This follows
`AGENTS.md`'s rule that tolerances sit at the algorithm's own error scale: the gain was
judged against measured noise, and the cost that decided it was loss of determinism across
inlining contexts, not ulp drift. The bench methodology is in
[benchmarking the evaluator](microbenchmark-protocol.md).

[^kb]: x86 study, "Kernel microbenchmarks", Emerald Rapids.
[^alg]: x86 study, "Algebraic float arithmetic", rustc 1.98, Emerald Rapids.
