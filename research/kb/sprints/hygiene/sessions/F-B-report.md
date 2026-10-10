---
type: Session Report
title: "F-B report: Lorentz, colour and wavefunction fixes"
description: "All 24 R-B findings plus three extra items fixed in three commits: oracles given teeth with 13 run mutations, the (j_L,j_R) table corrected, dead test-only abstractions deleted, and checked rational arithmetic from num-rational."
status: draft
tags: [hygiene, fix, helas, colour, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
sources:
  - {id: 7d33250, resource: "https://github.com/nsmith-/vibegraph/commit/7d33250", title: "give the vertex and wavefunction tests teeth"}
  - {id: bb64149, resource: "https://github.com/nsmith-/vibegraph/commit/bb64149", title: "correct the (j_L,j_R) table, drop dead representation scaffolding"}
  - {id: 2127cfe, resource: "https://github.com/nsmith-/vibegraph/commit/2127cfe", title: "checked arithmetic from num-rational, pinned conjugation, refused out-of-range diagrams"}
---
The dev agent's report, condensed by the manager. Everything except the
manager check is the agent's claim. The mutation logs and the driver that
applies, runs and reverts each mutation are in the scratchpad (`fb-mut1.log`,
`fb-mut2.log`, `fb_mutate.py`).

## Fixed

| finding | commit | change | mutation evidence |
|---|---|---|---|
| R-B.1 | 7d33250 | The e⁺e⁻→μ⁺μ⁻ tests use `jioxxx`; `j3xxxx` is deleted | Doubling the right current fails all 3 tests. Flipping the L/R relative sign passes (Found 2) |
| R-B.2 | 7d33250 | The Ward doc states what it pins and what it cannot see; bound \|M\| < 1e-12 | +1e-11 in `iovxxx` fails the new bound |
| R-B.3 | 7d33250 | The bra is checked against an explicit Weyl-basis ψ†γ⁰ | Dropping `.conj()` in `dualize` fails |
| R-B.4 | 7d33250 | ε_out = conj(ε_in) for nhel ∈ {−1,0,+1}, massive and massless, with a non-trivial-imaginary guard | Inert `nsv` fails |
| R-B.5 | 7d33250 | `jvvxxx`, `jsixxx` and the five `_basic` tests deleted (`run.rs` and the Ward tests cover the remaining ports) | — |
| R-B.6, .7, .11, .13, .24, .25 | 7d33250 | Doc corrections | — |
| R-B.12 | all | Plan, history and future-tense comments removed | — |
| R-B.14 | 7d33250, bb64149 | Empty impls and `DiracAdjoint::KET` removed | — |
| R-B.15 | 7d33250 | `VectorWf<F>` holds a contravariant `ComplexVector` | — |
| R-B.19 | 7d33250, bb64149 | Squared-norm asserts become linear (2 in `mod.rs`, 9 in `lorentz.rs`) | (1+1e-8) scalings fail three representative tests. No mutation was found for three sites |
| R-B.8 | bb64149 | Corrected table folded into `repr/mod.rs` with its reading stated; `intertwiner.rs` deleted; `clifford_product` → `cfg(test)` | — |
| R-B.22 | bb64149 | `VectorSpace` deleted; macro example and typo fixed | — |
| R-B.23 | bb64149 | `get`/`slot` → plain `cfg(test)` | — |
| R-B.9 | 2127cfe | `ColorRepr`/`SU3*`/`GroupScalar` deleted; Casimir literals kept | — |
| R-B.10 | 2127cfe | num-rational `CheckedMul`/`CheckedAdd`; tripwire matches the checked path's message; new add-overflow test | Unchecked `*` and `+` each fail. The old tripwire's `"overflow"` matched the debug panic too, so it was vacuous |
| R-B.17 | 2127cfe | Canonical-form test-only code and its two tests deleted | — |
| R-B.18 | 2127cfe | `Unsupported` doc lists the real refusals | — |
| R-B.20 | 2127cfe | One `is_permutation` | — |
| R-B.21 | 2127cfe | `coeff_conjugate_negates_the_imaginary_part` | An identity `conj` fails it (and passes the involution test) |
| R-B Found 1 | 2127cfe | `ColorTensor::conj` of `f` documented as MadGraph's default and unreachable | — |
| R-G1 Found 2 | 2127cfe | `LeadingColorFlows::of` asserts `diagram < n_diagrams`; a control and a `should_panic` test | The old silent filter fails the `should_panic` test |
| F-A Found 3 | 7d33250 | `GammaJout` → `GammaOout` | — |

**The corrected table's reading:** a bilinear is bra block ⊗ ket block → the
projected irrep, with ψ_L in (½,0) and χ̄ holding χ_R† in components 0,1.
Odd grades pair same-chirality blocks; even grades pair opposite ones.

## Gate (agent, at 2127cfe, `CARGO_INCREMENTAL=0`)

- **fmt** and **clippy, both configurations:** pass.
- **`cargo test --workspace`:** 1340 passed, 16 ignored. The baseline was
  1343; 7 tests were deleted and 4 added.
- **`cargo doc --document-private-items`:** 9 lib warnings, none under
  `helas/`.
- **Banked `color_cf_oracle`:** 97 passed.
- **Banked `color_flow_tags_oracle`:** 163 passed.

## Found

1. **The `run.rs:2723` comment says `fvixxx`'s `q = fi.p + v.p`.** It belongs
   to the evaluator cluster; a one-line fix.
2. **`jioxxx`'s relative L/R sign is pinned by no helas-root test.** It
   overlaps F-A Found 6 and needs a per-helicity mixed-chirality comparison.
3. **No CI step denies `cargo doc` warnings.** 9 remain.
4. **Kb close-out:**
   - `amplitudes/repr-layer-geometry-and-axes.md` (old chains, intertwiner
     links);
   - the `wavefn.rs:329-332` site of evaluator-doc-comments-stale is now
     resolved.
5. **R-B Found 3** (vector `nsv` end to end) stays open at the validation
   level.

## Brief corrections

- **R-B.1's fix moves the L/R sign blind spot rather than removing it.**
- **R-B.10's tripwire was vacuous under debug overflow checks.**
- **R-B.19's live sites were 11, not 13.**
- **The package is `vibegraph-lib`.**

## Method

- **Batching:** each group of edits was batched into an assert-checked patch
  script, then built once.
- **Mutations:** thirteen, at about 3 minutes of rebuild each, about 30
  minutes in total.
- **Test runtimes:** the full workspace test takes about 15 minutes on this
  host, and a `--lib -- helas::` run about 3.

## Manager check (2026-10-10)

- **Commits and trailers:** three commits, `Assisted-by` only. The diff is
  14 files, +363/−912.
- **Deletions:** no `j3xxxx`, `jvvxxx`, `jsixxx` or `GammaJout` remains in
  `src/`, and `intertwiner.rs` is gone.
- **Gate re-run:** fmt, both clippy configurations, the hermetic suite and
  `color_cf_oracle`. The results are in `log.md`.
