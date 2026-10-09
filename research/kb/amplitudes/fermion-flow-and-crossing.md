---
type: Physics Convention
title: Fermion flow, bra/ket dispatch and crossed legs
description: "Flow-typed fermion slots, bra/ket chosen by physical flow, the per-leg crossed bit, and how C Γᵀ C⁻¹ acts on vector, chiral, scalar and tensor bilinears read against their arrow."
status: draft
tags: [fermion-flow, spinors, crossing, helas, conventions]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n12-causes, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/12-helas-continuum-bugfix-journey.md#L37-L87", title: "Note 12: root causes 2, 3 and 5 (flow-typed slots, flow-driven dispatch, crossed-line conjugation)"}
  - {id: n35-f1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L475-L548", title: "Note 35 F1: sinks closing two fermion lines; per-pair bra/ket and crossed bookkeeping"}
  - {id: code-kernel, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/kernel.rs#L538-L600", title: "kernel.rs: resolve_bra_ket, off_shell_fermion_current"}
  - {id: code-adjoint, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/diagram_eval.rs#L38-L55", title: "diagram_eval.rs: ExtLegInfo::adjoint"}
  - {id: code-rootlorentz, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/root_lorentz.rs#L942-L1140", title: "root_lorentz.rs: standalone_projector_crossed, term_reversed_parity, pair_crossed, chiral_correction"}
  - {id: code-mixed, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/root_diagram.rs#L720-L850", title: "root_diagram.rs: collect_fermion_pairs, mixed_line_final_legs"}
---

# Fermion flow, bra/ket dispatch and crossed legs

## Fermion slots are typed by adjoint

A fermion value in the evaluator is either a ket (`WaveformSlot::FermionIn`, a
column spinor or flow-in current) or a bra (`FermionOut`, a row spinor or flow-out
current). Nothing converts one into the other on demand: the propagator numerator
`(q̸ + m)` does not commute with the Dirac adjoint, so an "adjoint when needed"
coercion corrupts any off-shell fermion current it touches.[^n12-causes] A
fermion line carries one adjoint from end to end, and every operation on it —
slash, chiral projector, `γ⁵`, propagator — preserves it.

**External legs.** A Dirac leg is a ket iff it is an incoming particle or an
outgoing antiparticle, `incoming == is_particle` — HELAS's `IXXXXX`/`OXXXXX`
choice (`ExtLegInfo::adjoint`).[^code-adjoint]

**Bilinears are resolved by the values, not the UFO indices.** At a vertex,
`resolve_bra_ket` reads which of the two fermion inputs is the bra and which the
ket from their runtime types; the UFO `i`/`j` positions only say which leg is
input and which output. When the slots arrive as `(ket, bra)` the line runs
against the vertex's defined adjoint and `reversed = true`; the caller applies its
structure's reversal sign (`C γ^{μT} C⁻¹ = −γ^μ` for a vector current).[^code-kernel]
With physically typed externals and adjoint-preserving currents, the two fermions
meeting at any vertex always have opposite adjoints.

**Off-shell fermion currents follow the input's adjoint**, with HELAS momentum
routing:

| input | current | momentum | HELAS |
|---|---|---|---|
| ket `ψ` | `ε̸ψ` | `q = p_f − p_v` | `fvixxx` |
| bra `ψ̄` | `ψ̄ε̸` | `q = p_f + p_v` | `fvoxxx` |
| bilinear → vector or scalar | `ψ̄Γψ` | `q = p_bra − p_ket` | `jioxxx` |

A flow-in current must subtract the boson momentum and a flow-out one add it; the
scalar (Higgs) current from an FFS vertex uses `p_bra − p_ket`, not the sum. Each
of these was once wrong and is pinned by the per-diagram oracles.

## The crossed bit

feyngraph binds outgoing legs in the all-incoming (crossed) convention. On a line
whose two ends are both final-state, the pair evaluates `ū₁Γv₂` where the vertex
is defined `ū₂Γv₁`, and `ū₁Γv₂ = −ū₂(CΓᵀC⁻¹)v₁`. Crossing inverts the slot identity
and the adjoint *together*, so no inspection of flow versus slot can detect it.
Each bound leg therefore carries an explicit bit, `LegAdjoint { adjoint, crossed }`,
set when its line's two externals are both final-state.[^n12-causes]

Lines with an initial-state end are handled differently. A final-state leg whose
line reaches an initial-state leg (a *mixed* line, e.g. Bhabha's t-channel
electron) is typed by its **physical** particle and adjoint, matching the
reference's external wavefunctions (`mixed_line_final_legs`). The crossed
representation C-conjugates the whole bilinear chain, which is an identity only
when both endpoints conjugate together — on a final–final line.[^code-mixed] The
resulting sign bookkeeping per line class is
[fermion-line-sign](fermion-line-sign.md).

## What reading a bilinear backwards does

`C Γᵀ C⁻¹` per structure, and where the evaluator applies it:[^code-rootlorentz]

| `Γ` | `C Γᵀ C⁻¹` | applied by |
|---|---|---|
| `γ^μ` | `−γ^μ` | reversed-bilinear parity at the vector sink (`term_reversed_parity`; runtime `reversed` flag) |
| `γ^μ P_χ` (gamma-chained projector) | `−γ^μ P_χ̄` | chirality flip per vertex (`chiral_correction`); the `−1` as above |
| standalone `P_χ`, `1`, `γ⁵` | unchanged | no flip; on a crossed line the reordering `−1` (`pair_crossed` at a scalar sink, all four; `standalone_projector_crossed` at a fermion output, `P_χ` and `γ⁵` only — a standalone `Identity` rooted at a fermion output on a crossed line takes no `−1`, and no gated row reaches that case) |
| `σ^{μν}` | `−σ^{μν}` | the same reversed-bilinear `−1` as `γ^μ`; a projector beside a literal `Sigma` keeps its chirality (`σ^{μν}` commutes with `γ⁵`) |
| `γ^αγ^β` (tensor-path line) | transposed | the two gammas swap; projectors move with their slots **without** conjugating chirality |

A gamma-chained projector conjugates on two disjoint occasions: an uncrossed line
traversing the vertex against its arrow (an initial-state annihilation pair), and a
crossed line. A crossed pair's two `−1`s cancel, so neither case carries an
explicit sign in `chiral_correction`. All the per-vertex signs that depend on the
rooting are lifted into the diagram's `fermi_sign`
([convention-sign-inventory](convention-sign-inventory.md)).

## Vertices that close two lines

A four-fermion vertex may close two fermion lines at once. The rooted tree gets
one spinor sink per pair, each with its own bra/ket resolution and `crossed`
bookkeeping: `collect_fermion_pairs` tags each open fermion end with its vertex
slot and closes them by the vertex's own pairing (its flow group), and
`OffShellCurrent`/`ContractAmplitude` nodes carry `fermion_pairs`.[^n35-f1] The
pairing decides how each shared external wavefunction is typed (crossed versus
mixed), which is why the pairing split lives at the diagram layer; see
[four-fermion-vertices](four-fermion-vertices.md). A current rooted on one line of
such a vertex must group fermion legs by the pairing, not consecutively; the
falsifier `four_fermion_currents_are_rooting_invariant` runs a line *through* the
contact by adding a photon, since every gated four-fermion diagram is a single
contact.

The spinor and variance types these slots carry are described in
[repr-layer-geometry-and-axes](repr-layer-geometry-and-axes.md). Majorana fermions
and the UFO `C` operator are refused
([majorana-fermions-unsupported](../backlog/feature/majorana-fermions-unsupported.md)).

[^n12-causes]: Note 12, root causes 2, 3 and 5.
[^n35-f1]: Note 35 §F1.
[^code-kernel]: `vibegraph-lib/src/helas/eval/kernel.rs`, `resolve_bra_ket`, `off_shell_fermion_current`.
[^code-adjoint]: `vibegraph-lib/src/helas/eval/diagram_eval.rs`, `ExtLegInfo::adjoint`.
[^code-rootlorentz]: `vibegraph-lib/src/helas/eval/root_lorentz.rs`.
[^code-mixed]: `vibegraph-lib/src/helas/eval/root_diagram.rs`, `mixed_line_final_legs`.
