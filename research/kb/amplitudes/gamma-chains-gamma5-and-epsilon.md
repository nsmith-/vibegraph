---
type: Design
title: Gamma chains, Gamma5 and Levi-Civita nodes in the rooted evaluator
description: "Adjoint inference along summed spinor indices, Gamma5/Gamma5Amp, EpsilonVout/EpsilonAmp with explicit argument order, P**2 and P(1,2)P(2,1) forms, PMomOut on fermion pairs, the fusion guard."
status: draft
tags: [lorentz-structures, smeft, gamma5, levi-civita, rooting]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n35-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L334-L431", title: "Note 35 E1: tree-shaped structures (Epsilon, γ-chains, Gamma5, momentum algebra)"}
  - {id: code-chain, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/root_lorentz.rs#L985-L1040", title: "root_lorentz.rs: chain_adjoint"}
  - {id: code-eps, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/root_lorentz.rs#L268-L288", title: "root_lorentz.rs: epsilon_out_order"}
  - {id: code-pmomout, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/kernel.rs#L512-L536", title: "kernel.rs: pmom_out, p_bra − p_ket for a fermion pair"}
  - {id: code-fuse, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/lower.rs#L340-L400", title: "lower.rs: chiral_gamma_site, the fusion guard"}
  - {id: mg-epsilon, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/aloha/aloha_object.py#L938-L948", title: "ALOHA aloha_object.py: L_Epsilon.give_parity"}
---

# Gamma chains, Gamma5 and Levi-Civita nodes

Everything here is a Lorentz structure whose index graph is still a **tree**, so
it roots at any leg with the ordinary walk (`root_lorentz.rs::build_at_leg`). The
cyclic four-fermion structures are the exception, handled in
[four-fermion-vertices](four-fermion-vertices.md).[^n35-e1]

## γ-chains: the adjoint of a node on a summed index

A fermion-output `Gamma` reached through a *summed* spinor index has no external
leg of its own to read an adjoint from. Its adjoint is that of the **external
fermion the chain leads to**: every operation along a fermion line — slash, chiral
projector, `γ⁵` — preserves the adjoint, so `chain_adjoint` walks spinor indices
from the node until it reaches a plain leg and returns that leg's adjoint.[^code-chain]
The walk is bounded by the term's operator count; a cyclic index graph returns
`None` rather than looping.

This is what makes these structures rootable at any leg, including a leg that is
not on the chain (the vector leg of an FFVV, where the rooted output carries no
adjoint at all):

- `Gamma(3,2,-2)*Gamma(4,-2,-1)*ProjM(-1,1)` (FFVV);
- the dipole chains `P(-1,3)*Gamma(-1,2,-3)*Gamma(3,-3,-2)*ProjM(-2,1)`;
- every structure with a `Gamma5` inside a chain.

A momentum slash is a `GammaIout`/`GammaOout` whose vector child is a `P` node.

## Gamma5

`LorentzOp::Gamma5` lowers to `Op::Gamma5` (`γ⁵` on a continuing fermion current;
it preserves the input's adjoint, being diagonal in the Weyl basis) and
`Op::Gamma5Amp` (the pseudoscalar bilinear `ψ̄γ⁵ψ` at a sink). The kernels reuse
`pseudoscalar_bilinear`. `C γ⁵ᵀ C⁻¹ = γ⁵`, so a standalone `γ⁵` over a crossed
pair takes the same reversed-reading `−1` a standalone chiral projector does, and a
`γ⁵` reached through a summed index needs no chirality conjugation of its own
([fermion-flow-and-crossing](fermion-flow-and-crossing.md)). The SM UFO writes `γ⁵`
as `ProjP − ProjM`, so only SMEFTsim and toy rows reach these ops.

## Levi-Civita

`Op::EpsilonVout` takes three vectors to the off-shell vector
`E^σ = ε^{μνρσ} a_μ b_ν c_ρ`, the free index **last**; `Op::EpsilonAmp` takes four
vectors to the scalar, operands in ε argument order. Antisymmetry makes argument
order a sign, so the rooting carries it explicitly: with the output at slot `k`,
`ε(x₀,x₁,x₂,x₃) = (−1)^{3−k} ε(remaining…, out)`, and `epsilon_out_order` absorbs
a `−1` by swapping the first two remaining operands, so the node never carries a
sign of its own.[^code-eps] An ε can also sit in a term as a disconnected scalar
factor.

The ε normalisation is ALOHA's: `L_Epsilon.give_parity` returns
`−sign(perm)`, i.e. `ε^{0123} = −1` (`ε_{0123} = +1`) on ALOHA's upper-index
representation, metric applied at contraction.[^mg-epsilon]

```python
return -self.sign * ((i2-i1) * (i3-i1) *(i4-i1) * (i3-i2) * (i4-i2) *(i4-i3))/12
```

That convention is pinned against MadGraph by `gg_to_h_cpodd`: the CP-even and
CP-odd effective vertices interfere inside each per-helicity JAMP, so a flipped ε
cannot be absorbed by the global phase. `EpsilonVout` (ε rooted at a vector leg)
is gated through `gg_to_gg_cg`'s amplitudes cell; it also rests on the hermetic
identity `EpsilonVout · d = EpsilonAmp`. ALOHA's `Sigma` normalisation and the
remaining Levi-Civita conventions: [levi-civita-and-sigma-conventions](levi-civita-and-sigma-conventions.md).

## Momentum algebra

- `P(-1,a)**2`: an object with a summed index raised to a power multiplies copies,
  so `P(-1,2)**2 = p₂·p₂`, parsed by the Lorentz grammar
  ([model/ufo-string-grammars](../model/ufo-string-grammars.md)) and rooted as a
  scalar `Metric(P, P)`. A power above 2 on an indexed object is refused (an index
  may appear at most twice in a term).
- `P(1,2)*P(2,1)` outer forms in VVS (`cHG`'s
  `P(1,2)*P(2,1) − P(-1,1)*P(-1,2)*Metric(1,2)`) and the VVV/VVVV structures with
  three momenta and one metric root through the same nodes.
- A `P` carrying the output Lorentz index emits a bare momentum vector wrapped in
  the vector-output transform, so the mixed `Metric`/`P`-rooted terms of one
  structure stay coherent.

**`PMomOut` on fermion pairs.** The output leg's momentum is minus the sum of the
inputs in the all-incoming convention. A boson current stores the momentum flowing
into the vertex; a fermion current stores the momentum along its line, so a pair
enters as `p_bra − p_ket`, the same combination its vector current is routed
with.[^code-pmomout] Summing the pair with two plus signs reads the wrong momentum
into every `P` naming the output leg of an FFV vertex — invisible in the SM, which
never puts a `P` there, and exposed by the SMEFTsim dipole row. Falsifier:
`momentum_slashed_chain_is_rooting_invariant` (`rooting_soundness.rs`).

Five- and six-leg vertices need no special handling: `build_at_leg`, scalar `Mul`
roots and the diagram walk are arity-agnostic.

## The fusion guard

Lowering fuses a chiral `Gamma·ProjM` / `Gamma·ProjP` pair into one `Ffv*` kernel
([intertwiner-basis-and-peephole](intertwiner-basis-and-peephole.md)). The fused
kernels fold the projector into one gamma node, so `chiral_gamma_site` refuses a
tree with a second `Gamma*` (a γ-chain, in particular a momentum-slashed dipole) or
a `Gamma5`: either puts another chirality-sensitive factor between projector and
current.[^code-fuse] A process-level fused-vs-generic check on SMEFTsim is vacuous
(its restricted vertices keep one chirality, so no chiral pair survives to fuse);
the equivalence evidence is the hermetic `ffv_vout_matches_generic_chiral_pair` /
`ffv_fermion_out_matches_generic_chiral_pair` pair in `kernel.rs`.

## Rows that exercise these structures

Gated: `gg_to_h_cpeven`, `gg_to_h_cpodd`, `ee_to_ttx_dipole`, `gg_to_gg_cg` (whose
contact sign is a vector-vertex matter: [vector-vertex-signs](vector-vertex-signs.md)).
Informational, each with its cause recorded in `validation/manifest.toml`:
`ee_to_wpwm_cw` (linear level exact; one of 48 `|M|²` points at 2.078e-12 against
a 1e-12 budget, 336× its own one-ulp sensitivity), `ee_to_zh_smeft` (MadGraph's
Fortran writes the `GC_303` literal `11/24` to seven digits; the reference is the
side that rounds) and `wpwm_to_wpwmz_cw`
([wpwmz-cw-ow-five-vector-residual](../backlog/validation/wpwmz-cw-ow-five-vector-residual.md)).
The SMEFTsim rows are described in [validation/non-sm-rows](../validation/non-sm-rows.md).

[^n35-e1]: Note 35 §E1, including its landed record.
[^code-chain]: `vibegraph-lib/src/helas/eval/root_lorentz.rs`, `chain_adjoint`.
[^code-eps]: `root_lorentz.rs`, `epsilon_out_order`.
[^code-pmomout]: `vibegraph-lib/src/helas/eval/kernel.rs`, `pmom_out`.
[^code-fuse]: `vibegraph-lib/src/helas/eval/lower.rs`, `chiral_gamma_site`.
[^mg-epsilon]: `aloha/aloha_object.py`, `L_Epsilon.give_parity`, at mg5amcnlo `b7687064`.
