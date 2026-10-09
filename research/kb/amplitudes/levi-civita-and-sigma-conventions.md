---
type: Physics Convention
title: ALOHA Levi-Civita and Sigma conventions
description: "ALOHA's Epsilon is ε^{0123}=−1; Sigma is half of (i/2)[γ^μ,γ^ν]; a chiral projector beside Sigma keeps its chirality; both pinned against MadGraph via CP-even/odd interference."
status: draft
tags: [levi-civita, sigma, aloha, sign-conventions, gamma5]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n35-ref, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L175-L214", title: "Note 35 §1.4 (reference conventions read from the pinned MadGraph)"}
  - {id: n35-r1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L242-L333", title: "Note 35 R1 (graded Clifford basis; Levi-Civita primitives; the σγ⁵ identity)"}
  - {id: n35-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L334-L431", title: "Note 35 E1 (Epsilon, Gamma5, γ-chains against MadGraph)"}
  - {id: n35-t1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L905-L991", title: "Note 35 T1 (the toy UFO; Sigma's half measured)"}
  - {id: n35-t2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L992-L1041", title: "Note 35 T2 (literal Sigma primitives)"}
  - {id: n35-closeout, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1281-L1358", title: "Note 35 §10.1 (what the sprint leaves gated; pinned conventions)"}
  - {id: n35-pin, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1398-L1415", title: "Note 35 §10.3 (the sigma_chained mutation pin)"}
  - {id: aloha-eps, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/aloha/aloha_object.py#L938-L983", title: "ALOHA aloha_object.py, L_Epsilon.give_parity"}
  - {id: aloha-sigma, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/aloha/aloha_object.py#L728-L790", title: "ALOHA aloha_object.py, L_Sigma.sigma"}
  - {id: code-lorentz, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/repr/lorentz.rs", title: "epsilon4, epsilon_vector, AsymRank2Tensor::hodge_dual, test_sigma_gamma5_epsilon_identity"}
  - {id: code-kernel, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/kernel.rs", title: "sigma_half and the Sigma kernels"}
  - {id: code-rootl, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/root_lorentz.rs", title: "sigma_chained, epsilon_out_order"}
---

# ALOHA Levi-Civita and Sigma conventions

UFO Lorentz structures name `Epsilon` and `Sigma`, but the UFO format does
not fix their sign or normalisation; ALOHA's definitions do, and MadGraph's
numbers follow ALOHA. Vibegraph adopts ALOHA's conventions and pins each one
with a test that fails if it is false. The kernels that use them are described
in [gamma-chains-gamma5-and-epsilon](../amplitudes/gamma-chains-gamma5-and-epsilon.md)
and [multivector-clifford-representation](../amplitudes/multivector-clifford-representation.md).

Basis facts these rest on: metric `(+,−,−,−)`; Weyl basis with
`γ⁵ = diag(−1, −1, +1, +1)` (components 0,1 left-chiral, 2,3 right-chiral).

## Levi-Civita: `ε^{0123} = −1`

ALOHA's `L_Epsilon.give_parity` stores the component `(l1,l2,l3,l4)` of the
upper-index object as `−sign(perm)` and applies the metric at
contraction[^aloha-eps]:

```python
def give_parity(self, perm):
    i1 , i2, i3, i4 = perm
    return -self.sign * ((i2-i1) * (i3-i1) *(i4-i1) * (i3-i2) * (i4-i2) *(i4-i3))/12
```

with `self.sign = 1`, so `(0,1,2,3)` gives −1[^n35-ref]. That is **`ε^{0123} = −1`,
equivalently `ε_{0123} = +1`**.

Vibegraph's primitives (`helas/repr/lorentz.rs`)[^code-lorentz]:

- **`epsilon4(a, b, c, d)`** takes four contravariant vectors and returns the
  all-lower contraction `ε_{μνρσ} a^μ b^ν c^ρ d^σ`. It is `+1` on
  `(e₀, e₁, e₂, e₃)`: the determinant of the matrix whose rows are the
  arguments in `[E, px, py, pz]` layout.
- **`epsilon_vector(a, b, c)`** returns the contravariant `E^σ = ε^{μνρσ} a_μ b_ν c_ρ`,
  characterised by `E·d = epsilon4(a, b, c, d)` for every `d`.

**Trap.** ALOHA's *stored* upper-index component is the negative of what
`epsilon4` returns on the basis vectors. Comparing a single component
against ALOHA's table without accounting for index position reads as a sign
error that is not there.

Everything in the module that involves ε derives from this one value (the
Hodge dual and the σγ⁵ identity below included), so a flip propagates
consistently and is caught by the identity test.

### Argument order is a sign

`EpsilonVout` evaluates `ε(remaining…, out)`, with the free (output) index
last. A UFO `Epsilon` with its output at slot `k` equals `(−1)^{3−k}` times
that form. `epsilon_out_order` (`root_lorentz.rs`) absorbs a `−1` by swapping
the first two remaining slots, so the node never carries a sign of its
own[^code-rootl]. `EpsilonAmp` takes its four operands in the structure's
argument order.

### How the ε sign is pinned

- **Inside one representation, before MadGraph.**
  `test_sigma_gamma5_epsilon_identity` pins
  `σ^{μν} γ⁵ = (i/2) · s · ε^{μνρσ} σ_{ρσ}` with **`s = −1`**, so
  `σ^{μν}γ⁵ = −(i/2) ε^{μνρσ} σ_{ρσ}`, under `ε^{0123} = −1` and the Weyl `γ⁵`
  above. `s` is the ε sign re-expressed; the test fails under the flipped
  convention. It ties the ε sign to the γ⁵ sign[^n35-r1].
- **Against MadGraph.** `gg_to_h_cpodd` (gated, 8.47e-16 at the time it was
  pinned): the CP-odd `ε` structure interferes with the CP-even one inside each
  per-helicity JAMP. A flipped ε changes the relative sign of the two, which the
  per-process global phase `G` cannot absorb. A CP-odd structure alone could
  not see the sign; |M|² of a pure CP-odd process is blind to it[^n35-e1].
- **Reach.** `EpsilonVout` (ε rooted at a vector leg) is exercised by the
  SMEFTsim rows `gg_to_gg_cg` and `ee_to_wpwm_cw`. Where no gated row reaches
  it, it rests on the hermetic identity `EpsilonVout · d = EpsilonAmp`.

## Sigma: half the textbook `σ^{μν}`

**ALOHA's `Sigma` is `½ σ^{μν}`, with `σ^{μν} = (i/2)[γ^μ, γ^ν]`.**
`L_Sigma.sigma` carries ±½ and ±½i where the textbook matrix carries ±1 and
±i[^aloha-sigma]. The kernels apply it through `kernel::sigma_half`, once per
`Sigma`[^code-kernel].

Measured, not read: on the toy model's `ll_to_qqx_toy_tensor`, the same
four-fermion operator written with two literal `Sigma`s (`FFFFT`) and with its
γγ expansion (`FFFFG`, a separate coupling on the same vertex) gives
`AMP(FFFFG)/AMP(FFFFT) = 4 × ggam/gtens` to 4.7e-14 over every helicity of every
banked point. A kernel at the textbook normalisation is 2× too large on a dipole
and 4× on a tensor⊗tensor contact; it reads |M|² `max_rel` 2.97 on
`ll_to_qqx_toy_dipole` and 0.569 on `ll_to_qqx_toy_tensor`[^n35-t1][^n35-t2].

**What pins Sigma's sign.** |M|² is blind to the global sign of `Sigma` in the
tensor row (it enters squared there). The dipole row pins it: there the
literal-`Sigma` structure `Sigma(3,-1,2,-2)*P(-1,3)*ProjM(-2,1)` interferes
linearly with a plain gauge coupling through one propagator. Negating
`SigmaVout` flips the fitted `G` to `−i` at per-diagram 3.0e-1; flipping the
`Sigma` cut of the contact reads 7.55e-1.

**`Sigma ⊗ Sigma` is the γγ expansion at the process level.** Both spellings
of the tensor operator are reproduced per diagram and per helicity in one
process (`ll_to_qqx_toy_tensor`). The γγ ⊗ γγ evaluation of
[four-fermion-vertices](../amplitudes/four-fermion-vertices.md) and the
literal `Sigma` path agree by the identity `γ^αγ^β = g^{αβ} − i σ^{αβ}`.

### Reversed lines

Reading a `Sigma` bilinear against the vertex's defined adjoint conjugates
the structure as `C σ^{μνT} C⁻¹ = −σ^{μν}`, so a reversed line takes a
relative −1, exactly as `C γ^{μT} C⁻¹ = −γ^μ` does for `GammaVout`. The
`SigmaVout` kernel's `negate` carries it[^code-kernel]. `SigmaVoutRev`
(free index on the second slot) is the negative of `SigmaVout`, since σ is
antisymmetric.

## A chiral projector beside a literal `Sigma` keeps its chirality

`σ^{μν}` commutes with `γ⁵`, so a projector next to `Sigma` reads the same on
either side of it. What conjugates the chirality is a single gamma:
`γ^μ P_χ = P_χ̄ γ^μ`. When the rooting reaches a projector through a summed
index, `sigma_chained` (`root_lorentz.rs`) asks whether the operator on that
side is a literal `Sigma`, and if so the chirality is not conjugated[^code-rootl].

Mutation-pinned: forcing `sigma_chained` to return `false` puts
`ll_to_qqx_toy_dipole`, and only that row (41 of 42 still pass), outside the
amplitude gate at per-diagram 9.985e-1. The gauge diagram keeps its constant
(`|g| = 1`, residual 2.68e-15) and the dipole diagram's own fitted constant
collapses to `|g| = 0.0756`, so the mutation lands where the rule
lives[^n35-pin].

The contrast with γγ chains matters. For a line read against its own arrow,
`C Γᵀ C⁻¹` on a **two**-gamma chain transposes the gammas and moves the
projectors with their slots **without** conjugating the chirality, unlike the
single-gamma case[^n35-closeout]. A `Gamma5` alone over a crossed pair keeps its
sign too (`C γ⁵ᵀ C⁻¹ = γ⁵`).

## `Gamma5`

The SM writes γ⁵ as `ProjP − ProjM` and has no Levi-Civita vertex, so the
`Gamma5`, `Gamma5Amp`, `EpsilonVout` and `EpsilonAmp` ops are listed in the
SM's `KNOWN_UNCOVERED` op census and are reached only by SMEFTsim and toy rows.
`Gamma5` preserves its input's adjoint: γ⁵ is diagonal in the Weyl basis, so the
same weighting acts on a ket from the left and on a bra from the right. The
fused chiral-FFV peephole refuses a structure containing `Gamma5` (it would
put a second chirality-sensitive factor between projector and current); see
[intertwiner-basis-and-peephole](../amplitudes/intertwiner-basis-and-peephole.md).

## Where these are exercised

| convention | hermetic pin | MadGraph row |
|---|---|---|
| `ε^{0123} = −1` | `test_sigma_gamma5_epsilon_identity`; `epsilon4` basis values | `gg_to_h_cpodd` (CP-even/odd interference) |
| `EpsilonVout` argument order | `EpsilonVout·d = EpsilonAmp` | SMEFTsim rows reaching ε at a vector leg |
| `Sigma = ½σ^{μν}` | `sigma_half` | `ll_to_qqx_toy_tensor` (ratio), `ll_to_qqx_toy_dipole` (sign) |
| projector beside `Sigma` | rooting tests | `ll_to_qqx_toy_dipole` (mutation-measured) |
| `Sigma ⊗ Sigma ≡ γγ` | the 4×4 Weyl-matrix reconstruction | `ll_to_qqx_toy_tensor` (both spellings per diagram) |

Reach gaps, measured: `SigmaVout` is reached by the dipole row and `SigmaOut`
(with `FierzOut`/`FierzOutRev`/`FierzPair`) by the tensor row. `SigmaMv` and
`MultivectorIout/Oout` need the structure on an internal line and have only
hermetic rooting tests. `SigmaVoutRev` and `SigmaOutRev` are reached by no
process in reach; the census note in `tests/smeftsim.rs` records that. The
toy models built for this are [validation/toy-ufo-models](../validation/toy-ufo-models.md).

One adjacent caveat belongs to the reference rather than to these
conventions: MadGraph's Fortran writer prints UFO literals to seven
significant digits, which made `ee_to_zh_smeft`'s `GC_303` differ by 1.2e-8
from MadGraph's own Python. That is a MadGraph defect, recorded in
[validation/madgraph-defects](../validation/madgraph-defects.md).

[^n35-ref]: Note 35 §1.4, the `Epsilon` convention read from the pinned source, recorded as a hypothesis until a MadGraph gate pinned it.
[^n35-r1]: Note 35 R1, landed deviations: `epsilon4` returns the all-lower symbol; `s = −1`.
[^n35-e1]: Note 35 E1: "the ALOHA ε sign is confirmed against MadGraph through the CP-even/CP-odd interference inside each per-helicity JAMP".
[^n35-t1]: Note 35 T1, finding 3 (ALOHA's `Sigma` is half the textbook σ).
[^n35-t2]: Note 35 T2: the textbook-normalisation numbers, the `SigmaVout` and cut mutations, and the reach of each `Sigma` op.
[^n35-closeout]: Note 35 §10.1, pinned conventions 1–3, and R4's record of `C Γᵀ C⁻¹` on two-gamma chains (note 35 lines 549–601).
[^n35-pin]: Note 35 §10.3.
[^aloha-eps]: `aloha/aloha_object.py`, `L_Epsilon.give_parity` and `__init__` (`self.sign=1`).
[^aloha-sigma]: `aloha/aloha_object.py`, `L_Sigma.sigma`, entries ±0.5 and ±0.5j.
[^code-lorentz]: `vibegraph-lib/src/helas/repr/lorentz.rs`: `epsilon4`, `epsilon_vector`, `test_sigma_gamma5_epsilon_identity`. The `epsilon4` doc comment still calls the sign "a hypothesis about MadGraph until an MG comparison exercises it"; `gg_to_h_cpodd` has since exercised it.
[^code-kernel]: `vibegraph-lib/src/helas/eval/kernel.rs`: `sigma_half`, `sigma_vout_bare`.
[^code-rootl]: `vibegraph-lib/src/helas/eval/root_lorentz.rs`: `sigma_chained`, `epsilon_out_order`.
