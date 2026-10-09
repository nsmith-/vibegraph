---
type: Design
title: "Four-fermion vertices: pairings, Fermi sign and the cyclic tensor structure"
description: "Per-structure fermion pairing split into feyngraph flow groups, MadGraph's permutation sign already inside our Fermi sign, and the Fierz cut that evaluates cyclic tensor×tensor contacts."
status: draft
tags: [four-fermion, smeft, fermion-flow, fierz, tensor]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n35-14, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L175-L214", title: "Note 35 §1.4: reference conventions read from the pinned MadGraph source"}
  - {id: n35-f1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L475-L548", title: "Note 35 F1: four-fermion vertices"}
  - {id: n35-r4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L549-L601", title: "Note 35 R4: the tensor slot and the cyclic four-fermion structures"}
  - {id: n38-s1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L362-L432", title: "Note 38 S1: the tensor-path line rule (2d99872)"}
  - {id: n39-1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/39-vector-vertex-signs.md#L40-L60", title: "Note 39 §1: the u u~ > t t~ g (vg_c4q) defect was the triple-gluon source sign"}
  - {id: mg-signflow, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/import_ufo.py#L1878", title: "MadGraph import_ufo.py: UFOMG5Converter.get_sign_flow"}
  - {id: mg-fermionflow, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/aloha/aloha_fct.py#L26", title: "MadGraph aloha_fct.py: get_fermion_flow"}
  - {id: code-topo, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/ufo/topo.rs#L200-L380", title: "ufo/topo.rs: fermion_flow, permutation_sign, flow_groups"}
  - {id: code-current-line, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/root_diagram.rs#L654-L680", title: "root_diagram.rs: fermion_current_line_sign"}
---

# Four-fermion vertices

## MadGraph's convention

MadGraph reads each four-fermion Lorentz structure's spinor pairing
`{I1: O1, I2: O2}` (`aloha_fct.py::get_fermion_flow`).[^mg-fermionflow] If it
differs from the reference `(1,2)(3,4)`, MadGraph builds the amplitude on the
reference pairing anyway and prefixes the coupling with the parity of the induced
permutation of particle positions (`import_ufo.py::get_sign_flow`):[^mg-signflow]

```python
if not flow or nb_fermion < 4:
    return ''
expected = {}
for i in range(nb_fermion//2):
    expected[i+1] = i+2
if flow == expected:
    return ''
```

For four fermions this `expected` evaluates to `{1: 2, 2: 3}`, which no real
pairing equals, so the early return never fires; the canonical `{1: 2, 3: 4}`
gets `+1` from the permutation-parity loop that follows (`import_ufo.py:1893-1918`).

Majorana fermions in four-fermion vertices are an `InvalidModel` in MadGraph, so
they stay out by construction.[^n35-14]

## This engine: one feyngraph vertex per pairing

Pairing is decided **per Lorentz structure**, not per vertex. A port of
`get_fermion_flow` (`ufo/topo.rs::fermion_flow`) reads each structure's oriented
pairing; `flow_groups` partitions a vertex's referenced structures by pairing;
`build_feyngraph_model` emits one feyngraph vertex per group (an `@k` suffix);
`Diagram::from_view` records `Vertex::flow_group`; and `VertexInfo::from_ufo` sums
that group's structures.[^code-topo] The UFO interaction set itself stays exactly
MadGraph's (interaction splitting by coupling-order tuple is described in
[model/coupling-orders](../model/coupling-orders.md)).

MadGraph's "canonical pairing × coupling sign" is not an option here: the pairing
decides how each shared external wavefunction is typed (crossed versus mixed line,
see [fermion-flow-and-crossing](fermion-flow-and-crossing.md)), so a diagram must
be built on the structure's own lines.[^n35-f1]

**The permutation sign is never multiplied in.** Building each diagram on the
structure's own lines puts the same parity inside the diagram's Fermi sign
(`Diagram::fermion_pairing_sign`). This is measured: on `ee_to_mumu_4f`, `V_729`'s
`(1,4)(2,3)` structures (`FFFF14+16`, `FFFF15`) and its `(1,2)(3,4)` structure
(`FFFF4`) interfere with eight γ/Z diagrams inside every helicity amplitude, and
MadGraph's `matrix1_orig.f` carries `−GC_54`, `−GC_29` on exactly the `(1,4)(2,3)`
calls; vibegraph matches per diagram without applying any coupling sign.
`topo::permutation_sign` exists as the statement of MadGraph's convention, checked
structure by structure against the SMEFTsim model.

**Census.** Of SMEFTsim's 21 four-fermion structures, 15 pair `(1,2)(3,4)` and 6
pair `(1,4)(2,3)` (`FFFF13` and `FFFF16` write their chains crossed). After
interaction splitting, 80 of 1985 interactions mix pairings, every one
same-flavour (`X̄ X X̄ X`). The flow split is load-bearing only there and is not
observable in any gated cell; it is pinned structurally. For a same-flavour
process (`e+ e- > e+ e- NP<=1`) this engine emits one diagram per pairing where
MadGraph draws one — the counting class of `gg_to_gg_cg` — and no banked row
checks it ([same-flavour-four-fermion-diagram-count-ungated](../backlog/validation/same-flavour-four-fermion-diagram-count-ungated.md)).
See [model/smeftsim-topu3l](../model/smeftsim-topu3l.md) for the model.

Sinks that close two lines at once, and the rooting-invariance falsifier, are in
[fermion-flow-and-crossing](fermion-flow-and-crossing.md).

## Cyclic tensor × tensor structures

A structure such as `(l̄σ^{μν}e)(q̄σ_{μν}u)` written as `γγ ⊗ γγ` has a cycle in its
index graph and cannot be rooted as a tree. A pre-pass over the term
(`cyclic_tensor_term`) recognises the shape; the walk the tree-shaped structures
use is untouched. The cycle is cut at the fermion line that does not contain the
output leg:[^n35-r4]

- the cut line is evaluated as a rank-2 Clifford element. A two-gamma chain lives
  in grades 0 and 2 only (`γ^αγ^β = g^{αβ} − iσ^{αβ}`), so it is fixed by its
  `ψ̄ψ` and `ψ̄σ^{μν}ψ` bilinears, and its contraction against the other line's two
  gammas is `4s − σ_{αβ}t^{αβ}` (`FierzOut`, `4s − 2·½t^{μν}σ_{μν}`; `FierzOutRev`
  with `+` when the two lines read the shared indices in opposite orders — the
  grade-2 sign is the whole difference);
- the element is applied to the continuing spinor (`MultivectorIout`/`Oout`,
  following the input's adjoint, momentum `p_bra − p_ket`) or paired into the
  amplitude (`FierzPair`, the grade-diagonal pairing). The value lives in a sixth
  register type, `WaveformSlot::Multivector`
  ([multivector-clifford-representation](multivector-clifford-representation.md)).

Conventions measured on this path: a line the vertex reads against its own arrow
takes `C Γᵀ C⁻¹`, which for two gammas transposes them and moves the projectors
with their slots without conjugating chirality; at the amplitude sink the choice of
which line to cut is free (a control mutation confirms it); the path reads each
line's bound adjoint at its row slot directly, never through the summed-index
chain walk. A literal `Sigma` per line is accepted in place of two adjacent gammas
(`SigmaOut`: the cut emits `∓2t`, the grade-0 term absent).

**Blind spot.** The gated row (`tata_to_ttx_tensor4f`) cannot see the grade-0
weight of the reconstruction: SMEFTsim writes the tensor operator with its aligned
structures at exactly `−2×` the reversed ones (`c₁₀₅₂ = −2 c₁₀₄₉` under
`vg_cleQt3`), so the scalar part cancels identically. That weight rests on the
hermetic 4×4 Weyl-matrix pin alone.

## The tensor-path line sign

A tree-rooted four-fermion current takes every line sign the diagram carries. A
line that closes at a *fermion-output tensor-path* current at the anchor rooting
takes none: `fermion_current_line_sign` divides those line signs back out of
`fermi_sign`. It is a convention of the tensor kernel, which cuts the index cycle
at one line, and was adjudicated against MadGraph standalone on processes where an
emission leaves the contact by one of its lines (`MG/ours` against the 1/4
helicity average):[^n38-s1][^code-current-line]

| process | contact | with the rule | without |
|---|---|---|---|
| `ta+ ta- > t t~ a` | tensor (`O_leQt3`) | 0.25000 ± 3e-6 | off by up to 20% |
| `lt~ lt > qt qt~ vt` (toy) | tensor | 0.25 exactly | off |
| `e+ e- > mu+ mu- a NP<=1` | vector (`vg_c4l`) | 0.21–0.49 | 0.25 exactly |

so the rule is confined to tensor-path vertices. `u u~ > t t~ g NP<=1` under
`vg_c4q` once missed MadGraph under either line rule; the four-quark contacts were
innocent — the cause was the triple-gluon source sign beside a quark-line anchor
([vector-vertex-signs](vector-vertex-signs.md)), and the process now agrees per
flow.[^n39-1] The full sign inventory is
[convention-sign-inventory](convention-sign-inventory.md).

[^mg-fermionflow]: `aloha/aloha_fct.py`, `get_fermion_flow`, at mg5amcnlo `b7687064`.
[^mg-signflow]: `models/import_ufo.py`, `get_sign_flow`, at mg5amcnlo `b7687064`.
[^n35-14]: Note 35 §1.4.
[^n35-f1]: Note 35 §F1, including its corrections to the §1.2 census (15/6, not 14/7; 80 of 1985 post-split, not "70 of 200").
[^n35-r4]: Note 35 §R4.
[^n38-s1]: Note 38 §S1 (commit 2d99872, landed in 1539abc).
[^n39-1]: Note 39 §1.
[^code-topo]: `vibegraph-lib/src/ufo/topo.rs`.
[^code-current-line]: `vibegraph-lib/src/helas/eval/root_diagram.rs`, `fermion_current_line_sign`.
