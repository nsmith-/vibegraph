---
type: Design Decision
title: "Kernel multiply-adds go through Real::mul_add_fast"
description: "Scalar and lane fields share one complex multiply-add on Real::mul_add_fast (hardware FMA or product+sum), enforced by clippy; restoring the packed complex idiom is x86-only and lost 8–9% on ARM."
status: draft
tags: [performance, fma, floating-point, simd, clippy]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
measured:
  - {commit: be76771, host: "x86-64 with AVX2 + FMA, no AVX-512", command: "RUSTFLAGS='-C target-cpu=native' cargo bench --bench eval_strategies (14 processes)"}
  - {host: "Apple M3 Max, macOS", command: "eval_strategies, the inlining tune + FMA against main; note 32 S9 A/B of the packed-complex trait"}
  - {host: "Intel Xeon Emerald Rapids (family 6 model 207), 4-vCPU Firecracker VM", command: "baseline x86-64 target, scalar forward, before (rustc 1.94, software FMA) vs after (rustc 1.98, mul_add_fast)"}
sources:
  - {id: x86-intro, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L14-L116", title: "x86 study: findings, FMA / mul_add, the latent test-fixture NaN"}
  - {id: x86-cum, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L187-L204", title: "x86 study: cumulative outcome"}
  - {id: x86-arm, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L231-L274", title: "x86 study, ARM: Δ% of the two shipped changes"}
  - {id: x86-armc, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L378-L398", title: "x86 study, ARM: correctness (reassociating, not order-preserving)"}
  - {id: x86-builds, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L651-L699", title: "x86 study: the two release builds; relaxed scalar multiply-add"}
  - {id: x86-alg, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/x86-avx2-perf-study-results.md#L700-L748", title: "x86 study: algebraic float arithmetic (SLP packs complex products)"}
  - {id: n32-s9, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L300-L353", title: "Note 32 §2 S9, the packed-complex workaround design"}
  - {id: n32-out, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L575-L597", title: "Note 32 §5.1, S9 outcome (8–9% loss on ARM)"}
---

# Kernel multiply-adds go through `Real::mul_add_fast`

**Decision.** Every multiply-add in the evaluator's kernels is written as
`a.mul_add_fast(b, c)`, a method on `Real` (`vibegraph-lib/src/helas/repr/mod.rs`):

```rust
fn mul_add_fast(self, a: Self, b: Self) -> Self {
    if HARDWARE_FMA { self.mul_add(a, b) } else { self * a + b }
}
pub const HARDWARE_FMA: bool = cfg!(any(
    target_feature = "fma",
    all(target_arch = "aarch64", target_feature = "neon")
));
```

One hardware FMA (single rounding) on x86 with `fma` (`x86-64-v3` and up) and on aarch64;
a product and a sum (two roundings) otherwise. Complex products go through two helpers in
`vibegraph-lib/src/helas/repr/lorentz.rs`, `cmul(a, b)` and `cmul_add(a, b, c)`, built
from it, and the dots, slashes, currents and bilinears build on those. The scalar `f64`
field and the lane field [`LaneField<N>`](lane-field-over-wide.md) run the same code.

`clippy.toml` bans the alternatives everywhere else (`disallowed-methods`): `f64::mul_add`,
`f32::mul_add` and `num_traits::float::Float::mul_add`, each with the reason "use
Real::mul_add_fast; without hardware FMA this is a software FMA call". The one sanctioned
call is inside `mul_add_fast`, under `#[allow(clippy::disallowed_methods)]`. A probe with
one planted call of each kind confirmed clippy catches them.[^x86-builds]

## Why this form

- **LLVM never contracts `a*b + c` by itself** (FP contraction is off by default), and
  `num_complex`'s operator impls hide the real arithmetic from it anyway. Before this,
  every complex multiply was `vmulpd` + `vaddsubpd` and every dot a multiply/add chain,
  with no FMA emitted on an FMA host.[^x86-intro]
- **One code path for every `F: Real`.** `Complex::mul_add` needs `num_traits::MulAdd` on
  the component type. The lane type had only the `Float::mul_add` method, and adding
  `MulAdd` to `Real` was an orphan-rule dead end (both foreign). Routing complex
  arithmetic through the real multiply-add fuses on every field, and sharing it is what
  keeps each lane bit-identical to scalar.
- **Why not plain `mul_add`.** `Float::mul_add` always rounds once, which on a target
  without FMA hardware is a software FMA call per use. On the baseline x86-64 release
  target that made scalar `forward` 2.3–3.8× slower than necessary:

| process (baseline x86-64, µs / 16 events) | software FMA | `mul_add_fast` | speedup | ÷ v3 scalar |
|---|--:|--:|--:|--:|
| `ee_to_mumu` | 19.7 | 8.7 | 2.28× | 1.04× |
| `gg_to_gg` | 115.6 | 48.7 | 2.37× | 1.20× |
| `ee_to_mumu_tata_qcd0` | 671.2 | 179.0 | 3.75× | 1.15× |
| `uux_to_ccx_emmm_qcd0` | 14 066 | 3 713 | 3.79× | 1.18× |

  (Emerald Rapids VM; "before" rustc 1.94 on an idle machine, "after" rustc 1.98 pinned to
  one core with another job sharing the VM, so a few percent of contention against a 2–4×
  effect. Full 8 rows in the source.) The amplitude oracle's residuals against MadGraph on
  that target stayed at the 1e-16 to 6e-11 scale; the largest move was an improvement
  (`ee_to_mumu_tata_qcd0` worst per-event 6.0e-12 → 2.8e-14).

## What it costs, and the packed-complex idiom

Without `mul_add`, plain `Complex<f64>` `*`/`+` compiles on x86 to the packed two-double
complex idiom (`vmulpd` + `vaddsubpd`). Decomposing into real FMAs trades that 2-op form
for more scalar FMAs.

- **x86 AVX2 host (`be76771`)**: scalar `forward` +3.55% mean (+2.86% median, −10% to
  +14%).[^x86-intro]
- **M3 Max**: the same change (with the inlining tune) was −9.7% on the cost-weighted scalar
  suite total; its +4.2% unweighted mean came from sub-10 µs processes.[^x86-arm]
- **Forcing the packed idiom back on ARM cost 8–9%.** The designed workaround, an in-house
  complex multiply-add trait whose default body is the shared real-FMA construction and
  whose `f64` override defers to `Complex<f64>`'s `num_traits::MulAdd` path, was built and
  A/B-measured on the M3 Max against a pre-registered kill criterion (≥ 2% forward geomean
  win); the idiom is x86-specific codegen, it lost, and nothing merged.[^n32-out] The design
  stays the resume point for an x86 host.[^n32-s9]
- **On Emerald Rapids the packed form also lost**: when algebraic float ops let the SLP
  vectoriser pack each complex product into `vmulpd` + `vaddsubpd` + shuffles before FMA
  formation, `fill_arenas` went from 534 to 172 scalar FMAs and ran 10–18% slower on 6 of
  8 rows.[^x86-alg] See [float reassociation](kernel-float-reassociation.md).

Whether `Complex::mul_add` on a lane field that implements `MulAdd` beats the shared path
is unmeasured. `LaneField` is local, so the orphan rule does not block it
([backlog](../backlog/performance/lane-field-complex-muladd-unmeasured.md)); any replacement
must keep lane-vs-scalar bit identity.

The lane-path gains recorded when FMA first went in (−24% to −35% median on the AVX2 host,
`vpermpd` 71 → 20) were measured under the `NumericArray` field, whose arithmetic ran as
out-of-line calls. Fewer, fused operations meant fewer calls, so those deltas measured call
overhead, not SIMD, and are superseded by [lane throughput](lane-throughput.md).[^x86-cum]

## Numerical contract

- **It reassociates.** One rounding replaces two, so results move at the last ulp by
  construction. Changes of this kind gate at the MadGraph amplitude oracle's tolerances
  (`vibegraph-lib/tests/amplitude_oracle.rs`), never bit-for-bit against the previous
  output.[^x86-armc]
- **Bit identity across fields, not across targets.** Scalar and lanes perform the same
  operations under the same `HARDWARE_FMA`, so lane tests are exact on every target. Two
  builds that differ in `HARDWARE_FMA` agree to rounding only.
- **A worked case of FMA exposing a latent bug.** FMA makes `m2()` more accurate, so
  hand-built "massless" test momenta rounded to m² ≈ −1e-16, `m()` returned `√(negative)` =
  NaN, and NaN comparisons silently took the wrong branch in three unit tests. The fix was
  in the fixtures: compare `m2()` (NaN-safe, the real intent) rather than `m()`. Production
  calls only `.m2()` on such momenta.

[^x86-intro]: x86 study, findings 1–3, "FMA / `mul_add`" and the correctness note, at `be76771`.
[^x86-arm]: x86 study, ARM results, Δ% table (56 cells).
[^n32-out]: Note 32 §5.1, S9 outcome.
[^n32-s9]: Note 32 §2, S9 design.
[^x86-alg]: x86 study, algebraic float arithmetic, variant v1.
[^x86-cum]: x86 study, cumulative outcome; the corrections in its AVX-512 section.
[^x86-armc]: x86 study, ARM correctness.
[^x86-builds]: x86 study, relaxed scalar multiply-add; `clippy.toml`.
