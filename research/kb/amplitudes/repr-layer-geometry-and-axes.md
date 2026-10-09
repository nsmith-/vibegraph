---
type: Design
title: "The repr layer: bundle picture, variance, Dirac adjoint and flow"
description: "Wavefunctions as sections of Spin(1,3)×gauge bundles, vertices as intertwiners; variance, bra/ket adjoint and in/out flow as three orthogonal, form-induced axes."
status: draft
tags: [representations, intertwiners, variance, dirac-adjoint, type-design]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n08-picture, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/08-repr-geometry.md#L10-L60", title: "Note 08 (geometric picture; the bundle picture)"}
  - {id: n08-traits, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/08-repr-geometry.md#L63-L123", title: "Note 08 §1–§4 (representation-trait strategy)"}
  - {id: n11-idea, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/11-variance-flow-duality.md#L10-L84", title: "Note 11 (form-induced dualities; the one real difference)"}
  - {id: n11-cautions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/11-variance-flow-duality.md#L111-L121", title: "Note 11 cautions"}
  - {id: n13-axes, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/13-typed-repr-conventions-design.md#L244-L271", title: "Note 13 §5 (form/adjoint discipline, three-axis terminology)"}
  - {id: n13-impl, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/13-typed-repr-conventions-design.md#L289-L354", title: "Note 13 §7 (implementation: rename, typed seam, contravariant-only vectors)"}
  - {id: code-lorentz, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/repr/lorentz.rs", title: "Variance, DiracAdjoint, VectorRepr, SpinorRepr, Bispinor"}
  - {id: code-intertwiner, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/repr/intertwiner.rs", title: "Intertwiner2Leg/3Leg/4Leg"}
  - {id: code-wavefn, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/wavefn.rs", title: "DiracWf, VectorWf, ScalarWf and the momentum-flow signs"}
  - {id: code-rootl, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/root_lorentz.rs", title: "Adjoint, LegAdjoint (runtime adjoint and the crossed bit)"}
---

# The repr layer: bundle picture, variance, Dirac adjoint and flow

`vibegraph-lib/src/helas/repr/` is the typed vocabulary under the amplitude
evaluator: Lorentz and colour representations over a generic real field `F`,
the vertex factors written as maps between them, and the markers that keep
index position and spinor side apart at compile time. This concept gives the
geometric picture it encodes and the three orthogonal "side" axes that every
wavefunction carries.

## The bundle picture

The structure group is Spin(1,3) × G (Spin^c in the SM, because of
hypercharge). Each wavefunction is a section of an associated vector bundle
over momentum space[^n08-picture]:

| Field | Bundle | Lorentz rep | Colour rep |
|---|---|---|---|
| left-handed fermion | S_L ⊗ V_q | (½,0) | fundamental or singlet |
| right-handed fermion | S_R ⊗ V_q | (0,½) | fundamental or singlet |
| gauge boson | T*M ⊗ ad(P_G) | (½,½) | adjoint |
| scalar | trivial ⊗ V_q | (0,0) | fundamental or singlet |

Vertex factors are not sections but **intertwiners**: Spin(1,3)-equivariant
linear maps between the fibres at each leg. `γ^μ : S_L → T*M ⊗ S_R` exists
because (½,0) ⊗ (½,½) ⊃ (0,½). A UFO vertex is then an element of
`Hom(R₁ ⊗ R₂ ⊗ R₃, ℂ)`, a colour tensor times a Lorentz tensor times a coupling,
which is also MadGraph's internal decomposition. The module doc of
`helas/repr/mod.rs` carries this table.

The Weyl (chiral) basis is the natural one here: S = S_L ⊕ S_R is manifest
(components 0,1 left-chiral, 2,3 right-chiral; `γ⁵ = diag(−1, 1)`), so the
chiral projectors are block selections. Basis independence is kept at the
trait level (`SpinorRepr`), but only the Weyl numerics exist; there is no
`WeylBasis` type any more, the name survives in doc comments only.

## Which layer uses which form

The picture is realised differently in two layers. Do not read note 08's
blanket intertwiner trait as the evaluator's design[^n08-traits].

- **`helas/repr` (the representation layer).** Lorentz and colour are separate
  traits: `LorentzRepr`, `VectorRepr<F, V: Variance>`,
  `SpinorRepr<F, Adj: DiracAdjoint>` in `repr/lorentz.rs`; `ColorRepr` with
  `SU3Fundamental`, `SU3Adjoint`, `ColorSinglet` in `repr/color.rs`. The vertex
  factors are methods on the representation types (`left_current`,
  `right_current`, `vector_bilinear`, `tensor_bilinear`,
  `fierz_coefficients`, `apply`, `epsilon_vector`, `epsilon4`). The leg-count
  traits `Intertwiner2Leg`/`3Leg`/`4Leg` in `repr/intertwiner.rs` name an
  orientation per implementor, but **nothing implements them**. The same goes
  for `Vertex3` and `GaugeVertex` in `repr/coupling.rs`: they are defined and
  have no users outside that file[^code-intertwiner].
- **`helas/eval` (the evaluator).** A flat, byte-sized `Op` set with one kernel
  per op; see [flat-op-ir](../amplitudes/flat-op-ir.md) and
  [intertwiner-basis-and-peephole](../amplitudes/intertwiner-basis-and-peephole.md).
  Colour does not appear in it at all. The runtime carries no colour vector,
  and `ColorRepr`'s numeric `Color` fibre is used only by hand-built
  wavefunction objects; colour is factored symbolically (see
  [madgraph-colour-factorization](../amplitudes/madgraph-colour-factorization.md)).

## Three axes, one gadget for two of them

"Flow" once named three different things. They are kept apart as three
orthogonal axes[^n13-axes]:

| Axis | Applies to | Form | Iso | Where |
|---|---|---|---|---|
| **Variance** (index up/down) | vectors, tensors | symmetric bilinear (metric `g`) | ℂ-linear `♭/♯`; `♭∘♭ = +id` | `Variance`: `Contravariant`/`Covariant` (`repr/lorentz.rs`) |
| **Dirac adjoint** (ket/bra) | spinors only | Hermitian sesquilinear (`ψ̄φ = ψ†γ⁰φ`) | conjugate-linear `bar()`; `♭∘♭ = +id` | `DiracAdjoint`: `Ket`/`Bra` (`repr/lorentz.rs`); runtime `Adjoint`/`LegAdjoint` (`eval/root_lorentz.rs`) |
| **Flow** (in/out) | every wavefunction | none: not a musical iso | — | HELAS `nsf`/`nsv`/`nss`, the sign on the stored momentum (`wavefn.rs`) |

Variance and the Dirac adjoint are two instances of one structure: a space
with a nondegenerate form, which gives a musical isomorphism V ≅ V*. The two
"sides" are V and V*, the form is the pairing, and index raising or the Dirac
adjoint is the iso[^n11-idea]. The payoff is that a node's output side is
derived (the adjoint of the map with respect to the forms), not hand-set;
contraction type-checks only between dual sides; and a propagator cannot
apply or drop `g` twice.

**The one real difference** is the kind of form[^n11-idea]:

- Variance uses a symmetric **bilinear** form. `dualize()` on a
  `ComplexVector` negates the spatial components and does not conjugate; `dot`
  is ℂ-bilinear.
- The Dirac adjoint uses a Hermitian **sesquilinear** form. `Bispinor::bar()`
  conjugates: `Bispinor::dualize`, in either direction, swaps the chiral blocks
  and conjugates each component (`ψ†γ⁰` in the Weyl basis), and `ψ̄Γψ` is
  sesquilinear.
- A third kind, **alternating** (symplectic), is not present. It would arrive
  with genuine two-component Weyl spinors, where the index is raised by
  `ε_{αβ}` and `♭∘♭ = −id`. A unified trait would have to allow that sign.

Only the invariants are adopted. There is no unified `Paired`/`Side`/`FormKind`
trait in the code; `Variance` and `DiracAdjoint` are separate sealed marker
traits with the same shape (an involutive `Dual` associated type and a
`const` orientation bit: `COVARIANT`, `KET`)[^code-lorentz].

## Cautions that still bind

- **Altitude.** `Variance` is a pure `repr`-layer marker. The wavefunction
  types (`DiracWf`, `VectorWf`) are carriers that also hold momentum; unify
  the marker, not the whole types[^n11-cautions].
- **Three different Z/2 dualities on spinors.** The Dirac adjoint (bra ↔ ket)
  is neither the holomorphic dual (½,0) ↔ (0,½) nor charge conjugation
  (particle ↔ antiparticle, the `Charge` enum). Keep them distinct.
- **Bra/ket is derived, not a free axis.** For spinors the adjoint follows
  from Flow ⊕ Charge through the fermion arrow the rooting chooses. The runtime
  `LegAdjoint` carries the adjoint plus a `crossed` bit, because diagram
  enumeration presents outgoing legs all-incoming: an outgoing μ⁺ is a bra at
  the `mu-` slot, and crossing inverts slot identity and adjoint together, so
  inspecting adjoint against slot cannot see it. A crossed pair evaluates
  `ū₁Γv₂` where the vertex is defined as `ū₂Γv₁`; by `ū₁Γv₂ = −ū₂(CΓᵀC⁻¹)v₁` that
  is exact for vector structures and needs `P_χ → P_χ̄` for gamma-chained chiral
  projectors[^code-rootl]. The signs this produces are in
  [fermion-flow-and-crossing](../amplitudes/fermion-flow-and-crossing.md).
- **Type aliases.** `InDiracWf<F>` = `DiracWf<F, Ket>` and
  `OutDiracWf<F>` = `DiracWf<F, Bra>`. Their names say "in/out", but the type
  parameter is the adjoint side, not the flow.

## Flow in detail

Every external constructor takes a flow flag and stores flag × momentum, the
HELAS convention that lets off-shell routines add and subtract leg momenta to
get an internal line's momentum[^code-wavefn]:

| constructor | flag | +1 | −1 | stored |
|---|---|---|---|---|
| `DiracWf::from_momentum` | `nsf: Charge` | `Particle` (u spinor) | `Antiparticle` (v spinor) | `nsf · p` |
| `VectorWf::vxxxxx` | `nsv: i32` | outgoing leg | incoming leg | `nsv · p` |
| `ScalarWf::sxxxxx` | `nss: i32` | outgoing leg | incoming leg | `nss · p` |

`nsv` and `nss` panic on anything but ±1. The spinor flag is the charge, not
the in/out direction; `DiracWf::charge` reads it back off the sign of the
stored energy. The wavefunctions themselves are in
[wavefunctions-and-propagators](../amplitudes/wavefunctions-and-propagators.md).

## Variance on the register

In the evaluator, variance and adjoint live on the produced slot, not on a
node tag[^n13-impl]:

- Every vector slot (`WaveformSlot::Vector`) holds the physical contravariant
  current `ε^μ`: external legs, momenta, vertex producers and propagated
  currents alike. There is no covariant vector slot. A kernel that needs
  covariant components lowers explicitly; `DiracAdjoint::slash_bispinor`, for
  example, takes covariant `v_μ` and leaves the lowering to its caller
  (`SpinorRepr::slash`).
- Fermion slots are `FermionIn` (ket) or `FermionOut` (bra). The bake step
  resolves each leg's `Adjoint` structurally, and an off-shell current and the
  propagator on it inherit the adjoint of their continuing fermion input.
- `VectorRepr` and `VectorWf` stay generic over `V: Variance` (default
  `Contravariant`), so covariant objects remain expressible in the repr
  layer; the evaluator does not produce them.

The sign each vector producer carries, and why, is in
[vector-vertex-signs](../amplitudes/vector-vertex-signs.md).

[^n08-picture]: Note 08, the bundle table and the intertwiner reading of `γ^μ`.
[^n08-traits]: Note 08 §1–§4 proposed separate Lorentz/gauge traits, an `Intertwiner<In, Out>` trait, `Vertex3<R1,R2,R3>` and a `Propagator<R>` trait. The repr layer kept the first and defines the leg-count intertwiner and `Vertex3` types without using them; the evaluator took none of them (note 13 §1b).
[^n11-idea]: Note 11, the form-induced-duality thesis and the form-kind taxonomy. Its "Flow" means the Dirac adjoint throughout.
[^n11-cautions]: Note 11, cautions 1 and 2.
[^n13-axes]: Note 13 §5, the three-axis terminology.
[^n13-impl]: Note 13 §7: the rename `SpinorFlow`/`FlowIn`/`FlowOut` → `DiracAdjoint`/`Ket`/`Bra` and runtime `Flow`/`LegFlow` → `Adjoint`/`LegAdjoint`; the typed propagator seam; the single contravariant vector convention.
[^code-lorentz]: `vibegraph-lib/src/helas/repr/lorentz.rs`: `Variance`, `DiracAdjoint`, `Bispinor::bar`, `ComplexVector::dualize`.
[^code-intertwiner]: `vibegraph-lib/src/helas/repr/intertwiner.rs` module doc: "nothing implements them yet". `Vertex3`/`GaugeVertex` in `repr/coupling.rs`.
[^code-wavefn]: `vibegraph-lib/src/helas/wavefn.rs` module doc, "Momentum-flow signs".
[^code-rootl]: `vibegraph-lib/src/helas/eval/root_lorentz.rs`, `Adjoint` and `LegAdjoint`.
