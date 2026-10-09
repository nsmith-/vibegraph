---
type: Design
title: "Multivector: the graded 1+4+6+4+1 Clifford representation"
description: "Multivector<F> stores a Cl(1,3)⊗C element as 16 graded coefficients; Clifford product, Fierz pairing, fierz_coefficients, and the completeness relations that pin them."
status: draft
tags: [clifford-algebra, fierz, multivector, four-fermion, representations]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n35-r1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L242-L333", title: "Note 35 R1 (the graded Clifford-basis tensor representation and the completeness relations)"}
  - {id: n35-r4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L549-L601", title: "Note 35 R4 (the tensor slot and the cyclic four-fermion structures)"}
  - {id: n35-d2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1191-L1218", title: "Note 35 §7, decision D2"}
  - {id: code-lorentz, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/repr/lorentz.rs", title: "Multivector, AsymRank2Tensor, SpinorRepr::fierz_coefficients and their tests"}
  - {id: code-op, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/op.rs", title: "FierzOut, MultivectorIout/Oout, FierzPair"}
---

# Multivector: the graded 1+4+6+4+1 Clifford representation

`Multivector<F>` (`helas/repr/lorentz.rs`) is an element of the complexified
spacetime Clifford algebra Cl(1,3) ⊗ ℂ: any 4×4 operator on Dirac spinors. It
is what a fermion line hands to another when two lines are joined by more
than one Lorentz index, and it is how the evaluator composes γ-chains of any
length without a type per chain.

## The decision: a graded basis, not sixteen `(μ, ν)` components

A fermion line can produce a general rank-2 object, `f̄ γ^μ γ^ν Γ f`, and by
extension any γ-chain with Γ ∈ Cl(1,3) ⊗ ℂ. It is stored in the **graded Dirac
basis `1 + 4 + 6 + 4 + 1`** (scalar, vector, bivector, axial vector,
pseudoscalar), not as a sixteen-component `(μ, ν)` array[^n35-d2]. The reasons:

- `γ^μ γ^ν = g^{μν} − i σ^{μν}` puts a two-gamma chain in grades 0 and 2 only.
- A chiral projector moves weight between the even grades (`1 ↔ γ⁵`,
  `σ ↔ σγ⁵ ∝ ε·σ`) and between the odd ones (`γ ↔ γγ⁵`).
- The antisymmetric rank-2 tensor is the grade-2 slice, not something extracted.
- Fierz orthogonality makes the tensor ⊗ tensor contraction of two lines a
  grade-diagonal pairing of their coefficient vectors.
- γ-chains compose by the Clifford product.

Not built: a symmetric rank-2 Lorentz tensor for spin-2 wavefunctions. That is
a different object (a Lorentz tensor, not a Clifford element) and would get its
own type.

## Storage and basis

`Multivector<F>` stores `[C<F>; 16]`, with grade accessors. One contiguous
array is what `ArrayBacked` (and with it the vector-space macros) needs[^code-lorentz]:

| grade | basis element | coefficient | slots |
|---|---|---|---|
| 0 | `1` | `s` | 0 |
| 1 | `γ_μ` | `v^μ` | 1–4 |
| 2 | `σ_{μν}` (`μ < ν`) | `T^{μν}` | 5–10 |
| 3 | `γ⁵ γ_μ` | `a^μ` | 11–14 |
| 4 | `γ⁵` | `p` | 15 |

so `M = s·1 + v^μ γ_μ + ½ T^{μν} σ_{μν} + a^μ γ⁵ γ_μ + p γ⁵`. Coefficients carry
the index variance opposite to their basis element, so every grade's
contraction is the plain Minkowski one.

**The grade-3 element is `γ⁵ γ^μ`, not `γ^μ γ⁵`.** The two orderings differ by a
sign. With this one, for `X = ψψ̄` the coefficient of `γ⁵γ_ν` is
`¼ Tr[X γ^ν γ⁵] = ¼ (ψ̄ γ^ν γ⁵ ψ)`, so `fierz_coefficients` equals the five existing
bilinears with no sign fixups. The sign has to live somewhere, and it sits in
the pairing: the grade-3 term of `fierz_pairing` is `−a_M · a_N`, because
`¼ Tr[γ⁵ γ_μ γ⁵ γ_ν] = −g_{μν}`.

The grade-2 slice is `AsymRank2Tensor<F>`: six contravariant components
`T^{μν}`, `μ < ν`, in the order `(0,1) (0,2) (0,3) (1,2) (1,3) (2,3)`. Read as a
Clifford element it is `Σ_{μ<ν} T^{μν} σ_{μν} = ½ T^{μν} σ_{μν}`. Its `dualize`
lowers both indices (only the three `(0,i)` slots change sign); `hodge_dual` is
`(⋆T)^{μν} = ½ ε^{μνρσ} T_{ρσ}` at `ε^{0123} = −1`. Writing `k^i = T^{0i}`
(boost-like) and `m^i = ½ ε^{ijk} T^{jk}` (rotation-like), the dual is
`(k, m) ↦ (−m, k)`, so `⋆⋆ = −1`. On the Weyl blocks it acts as `−i` on the
left-chiral block and `+i` on the right-chiral one, the (anti-)self-dual split;
flipping the ε convention would swap the eigenvalues. The ε convention is
[levi-civita-and-sigma-conventions](../amplitudes/levi-civita-and-sigma-conventions.md).

**Grade parity is chirality structure.** In the Weyl basis the even grades
(0, 2, 4) are block-diagonal and the odd grades (1, 3) block off-diagonal, so
even grades preserve a spinor's chiral blocks and odd grades swap them. The
explicit Weyl matrix (`to_weyl_matrix`), blocked `[[A, B], [C, D]]`:

```
A = (s − p) I₂ + (m⃗ + i k⃗)·σ⃗      B = (v⁰ − a⁰) I₂ − (v⃗ − a⃗)·σ⃗
C = (v⁰ + a⁰) I₂ + (v⃗ + a⃗)·σ⃗      D = (s + p) I₂ + (m⃗ − i k⃗)·σ⃗
```

## Operations

| method | what it is |
|---|---|
| `Multivector::clifford_product` | `self · rhs`, through the faithful 4×4 Weyl matrix rather than 256 structure constants; the basis products are pinned against explicitly built gamma matrices |
| `Multivector::fierz_pairing` | `⟨M, N⟩ = ¼ Tr[M N]` on coefficients; grade-diagonal, grade 3 with a minus sign |
| `Multivector::from_gamma(v)` | `v̸ = v^μ γ_μ` |
| `Multivector::from_gamma_pair(a, b)` | `a̸ b̸ = (a·b) − i σ^{μν} a_μ b_ν`, the closed coefficient form, checked against the Clifford product |
| `Multivector::from_projector(χ)` | `P_L = (1 − γ⁵)/2`, `P_R = (1 + γ⁵)/2`, or `1` for `Chirality::Both` |
| `SpinorRepr::apply(&M)` | `M ψ` on a ket, `ψ̄ M` on a bra |
| `SpinorRepr::fierz_coefficients(fi)` | all sixteen bilinears `f̄ Γ_A f` of a line at once |
| `SpinorRepr::tensor_bilinear(fi, χ)` | `f̄ σ^{μν} Γ f`, the grade-2 slice with the projector folded in (`σ^{μν}` commutes with γ⁵, so it may be read on either side) |

**`fierz_coefficients` returns the raw bilinears; the ¼ appears only in the
reconstruction.** Two identities fix the normalisation:

- the outer product: `ψ φ̄ = ¼ Σ_A (φ̄ Γ_A ψ) Γ^A`, a quarter of the multivector
  read as an operator;
- the pairing: `ψ̄ M ψ = ⟨fierz_coefficients(ψ̄, ψ), M⟩` for every Clifford
  element `M`.

The scalar, pseudoscalar, vector and axial bilinears that existed before equal
the corresponding grades of `fierz_coefficients` exactly, and
`f̄ a̸ b̸ f = (a·b) f̄f − i a_μ b_ν f̄σ^{μν}f`[^n35-r1].

`Multivector<F>` is generic over the scalar field like the rest of the repr
layer, so it runs wherever the evaluator does, including over the prime
field of [validation/finite-field-evaluator](../validation/finite-field-evaluator.md).

## How it is pinned, and each test's blind spot

| test (`repr/lorentz.rs`) | pins | cannot see |
|---|---|---|
| `test_completeness_relations` | `Σ_h u ū = p̸ + m`, `Σ_h v v̄ = p̸ − m`, projected through the Fierz bilinears | anything antisymmetric in helicity (it sums helicities first); eleven of the sixteen basis directions (it projects onto five) |
| `test_fierz_reconstruction`, diagonal form `ψψ̄` | the sixteen components of the outer product against the sixteen bilinears, one helicity at a time | the three boost-like `σ^{0i}` slots: `ψ̄σ^{0i}ψ ∝ (p⃗ × s⃗)^i` vanishes for a helicity eigenstate |
| `test_fierz_reconstruction`, off-diagonal form `ψφ̄` with an unrelated `φ̄` | all sixteen directions | — |
| `test_fierz_pairing_is_quarter_trace` | `⟨M, N⟩ = ¼ Tr[MN]` | — |
| `test_clifford_product`, `test_multivector_weyl_matrix` | products and the Weyl matrix against explicit gamma matrices | — |
| `test_sigma_gamma5_epsilon_identity` | `σ^{μν}γ⁵ = −(i/2) ε^{μνρσ} σ_{ρσ}` under `ε^{0123} = −1` | — |

The per-helicity reconstruction is the finest oracle in the module: it
passes only if every bilinear's normalisation, sign and index order is
mutually consistent. A mutation sweep of five was caught by these tests; one
of the mutations exposed the diagonal form's `σ^{0i}` blindness, which is why
the off-diagonal form exists[^n35-r1].

## Where the evaluator uses it

The cyclic four-fermion structures are the consumer: two fermion lines joined
by two summed Lorentz indices close a cycle in the index graph, so no rooted
tree can contract them one index at a time. The rooting cuts the cycle at one
line and evaluates that line as a `Multivector`[^n35-r4][^code-op]:

- `WaveformSlot::Multivector(MultivectorWf<F>)`, the line's sixteen coefficients
  plus momentum, is a sixth result arena. A `Multivector` never leaves its
  vertex; propagating one panics.
- `FierzOut` turns the cut line into `4s − 2·(½ t^{μν} σ_{μν})` (from its grade-0
  and grade-2 bilinears `s`, `t`), using `γ_α γ_β g^{αβ} = 4` and
  `γ_α γ_β t^{αβ} = −i σ_{αβ} t^{αβ}`. `FierzOutRev` is the same with `+`: the two
  lines reading the shared indices in opposite orders differ in the grade-2 sign
  and nothing else.
- `MultivectorIout` / `MultivectorOout` apply the element to the other line's
  continuing spinor (following its adjoint; momentum `p_bra − p_ket`).
  `FierzPair` closes it into the amplitude as the grade-diagonal pairing.
- The literal-`Sigma` contact (`SigmaOut`/`SigmaOutRev`) is the same cut with the
  two gammas already contracted, so the grade-0 term is gone. `SigmaMv` turns a
  doubly contracted `Sigma` into a pure grade-2 element.

Pairing conventions, the Fermi sign and the cut itself are
[four-fermion-vertices](../amplitudes/four-fermion-vertices.md).

**Caveat on what the gated row sees.** `tata_to_ttx_tensor4f` (SMEFTsim
`O_leQt3`) gates the tensor ⊗ tensor path per diagram, but SMEFTsim writes the
operator with its aligned structures at exactly −2× the reversed ones
(`c₁₀₅₂ = −2 c₁₀₄₉` under `vg_cleQt3`), so the scalar part of the
reconstruction cancels identically. That is the γγ⊗γγ decomposition of
`(l̄σ^{μν}e)(q̄σ_{μν}u)` doing its job, and it means the **grade-0 weight rests on
the hermetic 4×4 Weyl-matrix pin alone**. The toy model's `Sigma ⊗ Sigma` row
reproduces both spellings of the operator per diagram, which is the
process-level check of the γγ path against the literal `Sigma`
([validation/toy-ufo-models](../validation/toy-ufo-models.md))[^n35-r4].

[^n35-r1]: Note 35 R1 and its landed deviations: `γ⁵γ^μ` for grade 3; raw `fierz_coefficients` with the ¼ only in reconstruction; the Clifford product through `to_weyl_matrix`; `[C<F>; 16]` storage with grade accessors (five named fields cannot give one contiguous array); `AsymRank2Tensor`'s slot order; the mutation sweep and the off-diagonal reconstruction.
[^n35-r4]: Note 35 R4: the tensor slot, `FierzOut`/`FierzOutRev`, `MultivectorIout/Oout`, `FierzPair`, and the grade-0 blind spot of the SMEFTsim row.
[^n35-d2]: Note 35 §7, decision D2: the tensor representation lives in the graded Dirac basis; spin-2 externals deferred; spin-3/2 and Majorana out.
[^code-lorentz]: `vibegraph-lib/src/helas/repr/lorentz.rs`: `Multivector` and its doc table, `AsymRank2Tensor`, `SpinorRepr::fierz_coefficients`, the tests named above.
[^code-op]: `vibegraph-lib/src/helas/eval/op.rs`: the tensor-tensor and literal-`Sigma` primitives.
