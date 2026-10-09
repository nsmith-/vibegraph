---
type: Design Decision
title: Evaluator IR is a flat Op set; variance lives on the register
description: "Primitives are irreducible intertwiners named one-to-one with a flat, dataless Op enum and kernel fns; variance and adjoint ride on the waveform slot, not on a node tag."
status: draft
tags: [evaluator, ir, design-decision, kernels, representation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n13-1a, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/13-typed-repr-conventions-design.md#L45-L77", title: "Note 13 §1a: primitives are irreducible intertwiners, not HELAS routines"}
  - {id: n13-1b, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/13-typed-repr-conventions-design.md#L78-L112", title: "Note 13 §1b and its revision: a flat op set, not a two-level enum"}
  - {id: n13-7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/13-typed-repr-conventions-design.md#L289-L354", title: "Note 13 §7: kernel factor-out, 1-1 Op naming, typed propagator seam"}
  - {id: code-op, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/op.rs", title: "helas/eval/op.rs: the Op enum"}
  - {id: code-slot, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/waveform_slot.rs#L31-L52", title: "helas/eval/waveform_slot.rs: WaveformSlot"}
---

# Evaluator IR is a flat Op set; variance lives on the register

## Decision

The evaluator's node language is one flat, dataless enum, `Op`
(`helas/eval/op.rs`). A node is an `Op` tag plus a typed leaf payload; children
live in the `Ast` arena. Each Lorentz `Op` maps one-to-one to a kernel function
named for it in `helas/eval/kernel.rs` (`Op::GammaVout` → `kernel::gamma_vout`,
`Op::FierzOut` → `kernel::fierz_out`, …), so the interpreter's dispatch is one
`match` of `kernel::<op>(children)`.[^code-op] Where several ops share arithmetic
they are thin wrappers over a private helper (`proj_m`/`proj_p` →
`chiral_project`; `proj_m_amp`/`proj_p_amp`/`identity_amp` →
`scalar_bilinear_current`). Structural and algebraic ops (`External`, `Mul`, `Add`,
`Coupling`, `Flows`, `Hels`, `Configs`, …) are not kernels.

What a produced value *is* — vector or spinor, ket or bra — is carried by the
value, not by the node: `WaveformSlot` distinguishes `FermionIn` (ket) from
`FermionOut` (bra), and every `Vector` is the physical contravariant current.[^code-slot]
There is no "output type" level in the IR.

## Why

**Primitives are irreducible intertwiners, not HELAS routine names.**[^n13-1a]
In `S* ⊗ S → V` the vector has exactly two invariant intertwiners, the left
current `γ^μ P_L` and the right current `γ^μ P_R`. Every SM FFV structure is a
point in that 2-D span:

| UFO | structure | `[g_L, g_R]` |
|---|---|---|
| FFV1 | `γ^μ` | `[1, 1]` |
| FFV2 | `γ^μ P_L` | `[1, 0]` |
| FFV3 | `γ^μ P_L − 2 γ^μ P_R` | `[1, −2]` |
| FFV4 | `γ^μ P_L + 2 γ^μ P_R` | `[1, +2]` |
| FFV5 | `γ^μ P_L + 4 γ^μ P_R` | `[1, +4]` |

The `−2, +2, +4` are the SM's chiral charges, not Clebsch–Gordan coefficients;
the representation theory fixes only the dimension (2). So the IR names the
irreducible pieces (`GammaVout`, `ProjM`, `ProjP`, …) and coefficient vectors
rebuild the named structures. ALOHA is the oracle (does `{left, right}·coeff`
reproduce each `FFVn`?), not the interface. The fused `FfvVout`/`FfvIout`/`FfvOout`
kernels are this span in coordinates: one spinor pass, both chiral projections,
one linear combination — HELAS's own `iovxxx(fo, fi, v, [g_L, g_R])`. See
[intertwiner-basis-and-peephole](intertwiner-basis-and-peephole.md).

**No two-level enum.**[^n13-1b] A nested `Outer { VectorOut(V), SpinorOut(Adj), … }`
was considered to make the output type explicit. It was rejected:

1. *Layout.* A nested enum carries two discriminants and is larger than the flat,
   dataless `Op`, which fits in a byte. The arena is walked in the integrand's
   inner loop, so node size matters.
2. *Redundancy.* The variance and adjoint the outer level would carry already live
   on the register. A node tag would duplicate a fact the produced slot states.

The typing that prevents duality-boundary bugs — an off-shell fermion current
coerced between ket and bra, or a covariant vector fed to a contravariant
consumer — is the slot's own variant, enforced by the kernels' pattern matches.
The old "adjoint on demand" `.bar()`/`.unbar()` coercions corrupted the
propagator `(q̸+m)`, which does not commute with the adjoint; flow-typed fermion
slots replaced them (see [fermion-flow-and-crossing](fermion-flow-and-crossing.md)).

The full intertwiner/trait machinery of the repr layer (generic multi-leg
intertwiner traits, the Lorentz×gauge product bundle, the Weyl ε) stays out of the
eval layer: it buys compile-time proof at a churn cost the evaluator does not
need. See [repr-layer-geometry-and-axes](repr-layer-geometry-and-axes.md).

## Consequences

- Adding a primitive means one `Op` variant, one kernel, lowering, and the
  egglog constructor; the op ↔ s-expression bijection (`strum`-derived names) and
  the op-coverage census extend mechanically.
- Every vector current is contravariant, so the propagator has one vector branch
  per mass case and needs no lowered-storage opcode
  ([global-phase-i-counting](global-phase-i-counting.md)).
- The flat `Op` tree is the compile-time form. For execution it is lowered again
  into a typed instruction stream over per-type arenas
  ([performance/evaluator-program-layout](../performance/evaluator-program-layout.md)).
- Lorentz and fermion-sign coefficients are still `Coeff(f64)` leaves while colour
  coefficients are exact `CoeffRat`
  ([lorentz-coefficients-still-f64](../backlog/hygiene/lorentz-coefficients-still-f64.md)).

[^code-op]: `vibegraph-lib/src/helas/eval/op.rs` and `kernel.rs`; the dispatch is `run.rs::apply`.
[^code-slot]: `vibegraph-lib/src/helas/eval/waveform_slot.rs`.
[^n13-1a]: Note 13 §1a.
[^n13-1b]: Note 13 §1b, including its revision; §7 steps 2 and 4.
