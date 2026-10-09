---
type: Design
title: Intertwiner basis, general tier and peephole kernels
description: "The Lorentz evaluator is a general typed node tree; a non-exhaustive peephole layer picks fused kernels by intertwiner-basis coordinates; EFT structures fall through to the general tier."
status: draft
tags: [intertwiners, evaluator, peephole, eft, lorentz-structures]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n13-summary, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/13-typed-repr-conventions-design.md#L23-L39", title: "Note 13 §0 (one-paragraph summary)"}
  - {id: n13-primitives, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/13-typed-repr-conventions-design.md#L45-L77", title: "Note 13 §1a (primitives are irreducible intertwiners)"}
  - {id: n13-basis, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/13-typed-repr-conventions-design.md#L113-L171", title: "Note 13 §2, §2a, §2b (the basis from the leg reps; glossary; SM enumeration)"}
  - {id: n13-catalog, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/13-typed-repr-conventions-design.md#L172-L222", title: "Note 13 §3, §3a (rewrite catalog and mechanics)"}
  - {id: n13-scope, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/13-typed-repr-conventions-design.md#L272-L288", title: "Note 13 §6 (scope: the EFT extension point)"}
  - {id: code-lower, resource: "vibegraph-lib/src/helas/eval/lower.rs", title: "chiral_gamma_site / lower_vertex: the chiral-pair fusion"}
  - {id: code-kernel, resource: "vibegraph-lib/src/helas/eval/kernel.rs", title: "ffv_vout / ffv_iout / ffv_oout and their fused-vs-generic tests"}
  - {id: code-op, resource: "vibegraph-lib/src/helas/eval/op.rs", title: "The flat Op set"}
---

# Intertwiner basis, general tier and peephole kernels

The Lorentz evaluator has two tiers[^n13-summary]:

1. A **general typed node tree**, which is the semantics. Every UFO Lorentz
   structure lowers to granular primitives (`Gamma*`, `Proj*`, `Metric`,
   momentum reads, `Gamma5`, `Epsilon*`, `Sigma*`, the Fierz/multivector
   nodes). It is complete for any structure the rooting accepts, dim-6 EFT
   included, and carries the variance/adjoint typing that keeps the duality
   boundaries correct whether or not any optimisation fires.
2. A **peephole (instruction-selection) layer** that recognises common vertex
   patterns at compile time and emits a fused kernel parameterised by the
   pattern's coordinates in a small irreducible basis. It never has to be
   exhaustive: anything it does not match falls through to tier 1.

Type safety comes from typing the general nodes, so it needs no closed
primitive set. Fusion is an optimisation applied where a pattern is worth a
kernel. The two goals are independent[^n13-catalog].

The node representation is a flat, byte-sized `Op` enum with one kernel
function per op; variance and adjoint ride on the produced register, not on a
node tag. That decision has its own concept,
[flat-op-ir](../amplitudes/flat-op-ir.md).

## Primitives are irreducible intertwiners, not HELAS routines

In `S* ⊗ S → V` the vector appears in exactly two invariant intertwiners,
the left current `γ^μ P_L` and the right current `γ^μ P_R`. Every SM FFV
structure is a point in that two-dimensional span[^n13-primitives]:

| UFO | structure | `[g_L, g_R]` |
|---|---|---|
| FFV1 | `γ^μ` | `[1, 1]` |
| FFV2 | `γ^μ P_L` | `[1, 0]` |
| FFV3 | `γ^μ P_L − 2 γ^μ P_R` | `[1, −2]` |
| FFV4 | `γ^μ P_L + 2 γ^μ P_R` | `[1, +2]` |
| FFV5 | `γ^μ P_L + 4 γ^μ P_R` | `[1, +4]` |

The `−2, +2, +4` are the SM's phenomenological chiral charges (roughly
`T³ − Q sin²θ_W` combinations), not Clebsch–Gordan coefficients. What
representation theory fixes is the *dimension* of the space, two; the named
FFVn are lattice points in it. HELAS's `iovxxx(fo, fi, v, [g_L, g_R])` is the
same space in coordinates.

So the evaluator maps 1-to-1 onto irreducible intertwiners, not onto ALOHA
routine names (`FFV2_4_3`). The compiler keeps each UFO `LorentzTerm.coeff`
separately and lowers `Gamma·ProjM` / `Gamma·ProjP` to distinct nodes;
coefficient vectors reconstruct the named structures. ALOHA is the oracle that
`{left, right}·coeff` reproduces each `FFVn`, not the interface.

## The basis is (mostly) enumerable from the leg reps

The spinor/index part of the basis is fixed by the leg representations through
classical invariant theory. The number of momentum (derivative) insertions is
not: it is a graded tower that renormalizability (operator dimension ≤ 4)
truncates[^n13-basis].

Complete basis = {invariant index structures saturating the leg indices} ×
{momentum insertions up to the dimension bound}. The index structures come
from the only Lorentz-invariant tensors there are: `δ`, `g^{μν}`, the Clifford
generators `{1, γ⁵, γ^μ, γ^μγ⁵, σ^{μν}}`, `ε^{μνρσ}`, and the momenta `P^μ`.

Terms:

- **Grade.** The number of `P^μ` factors. Each is a derivative and raises the
  operator dimension by one; grades do not mix. The truncation is a physics
  input, not a representation-theory fact, which is why it is the EFT
  extension point.
- **Bose/Fermi reduction.** Exchange symmetry of identical legs and momentum
  conservation make naive structures dependent (VVVV's three metric pairings
  collapse to the one physical quartic under Bose symmetry plus colour).
- **Schouten and Fierz identities.** Four-dimensional linear relations
  (no antisymmetrisation of five indices; completeness of the 16 gamma
  covariants) that make a naive basis over-complete. They do not bite for the
  2- and 3-point SM vertices; they must be quotiented out at four points and
  wherever `ε` or tensors appear.
- **YM structure.** `g^{μν}(p₁−p₂)^ρ + g^{νρ}(p₂−p₃)^μ + g^{ρμ}(p₃−p₁)^ν`, the
  grade-1 VVV intertwiner.

Renormalizable SM enumeration, per output leg. The last column is how the
current code realises each family (`LorentzEvalNode` in
`helas/eval/root_lorentz.rs`, `Op` in `helas/eval/op.rs`):

| Family | spins | output | basis | dim | ∂ | realised by |
|---|---|---|---|---|---|---|
| FFV | ½,½,1 | V | `{γ^μP_L, γ^μP_R}` | 2 | 0 | `GammaVout`; fused `FfvVout` |
| | | F | `{γ̸P_L, γ̸P_R}` on ket | 2 | 0 | `GammaIout`/`GammaOout`; fused `FfvIout`/`FfvOout` |
| FFS | ½,½,0 | S | `{P_L, P_R}` | 2 | 0 | `ProjMAmp`/`ProjPAmp` (`IdentityAmp` for `δ`) |
| | | F | `{P_L, P_R}` on ket | 2 | 0 | `ProjM`/`ProjP` |
| VVS | 1,1,0 | S | `{g^{μν}}` | 1 | 0 | `Metric` |
| | | V | `{g^{μν}→V^μ}` | 1 | 0 | `MetricVout` |
| VVV | 1,1,1 | V | `{YM}` | 1 | 1 | `Metric`/`P`/`POut`/`Mul` composition |
| VVVV | 1,1,1,1 | V | three metric pairings | 3 | 0 | `Metric`/`MetricVout` composition |
| VSS | 1,0,0 | V | `{(p₁−p₂)^μ}` | 1 | 1 | `P`/`POut` read-offs |
| SSS/SSSS | 0,… | S | `{1}` | 1 | 0 | coupling × identity |

Two facts follow. VVV must carry a derivative: three vector indices cannot
form a scalar without a fourth index (`g` supplies two, `ε` needs four). And
FFV is one two-dimensional coefficient space whichever leg is the output: the
*basis* is keyed by the leg-rep multiset, the *realisation* by which leg is the
output (its variance or adjoint).

There is no dedicated VVV node. The grade-1 current is a composition of
`Metric`, `P` (an input leg's momentum) and `POut` (the output leg's momentum,
`−Σ` inputs, as ALOHA's `VVV1P0_1` writes `P1 = −(V2+V3)`). The sign
conventions it carries are in [vector-vertex-signs](../amplitudes/vector-vertex-signs.md).

## The enumeration is a rewrite catalog, not the whole design

A closed primitive set is closed only under the renormalizability truncation.
Dim-6 and dim-8 SMEFT bring higher grade, `ε` structures, `σ^{μν}p_ν`
dipoles and tensor currents, so making the enumerated intertwiners the *only*
representation would be a trap. The §2b table is the catalog of rewrite
targets for common cases, explicitly non-exhaustive; the truncation marks the
boundary between "has a peephole kernel" and "generic fallback"[^n13-catalog].

Mechanics[^n13-catalog]:

- Matching runs at compile time, once per vertex and output leg, never per
  phase-space point.
- Coordinate read-off expresses the vertex's terms in the kernel's basis. If
  they do not lie in it (an extra grade, an `ε` the kernel does not model), the
  read-off fails and the structure stays generic.
- Every fused kernel has a property test, fused == generic on random typed
  inputs. This replaces "add a node when a validation process breaks" with a
  local, mechanical definition of correct for each hand-coded kernel. The
  harness and the per-model op census are
  [validation/evaluator-kernel-coverage](../validation/evaluator-kernel-coverage.md).

## The peephole layer as built

One family of patterns is fused today: the **chiral-pair FFV** sites
(`helas/eval/lower.rs`, `chiral_gamma_site` and `lower_vertex`)[^code-lower].

- A site is a tree with exactly one `ProjM`/`ProjP` adjacent to exactly one
  `Gamma*` node, and no `Gamma5`. A second gamma (a γ-chain, a momentum-slashed
  dipole) or a `Gamma5` puts another chirality-sensitive factor between the
  projector and the current, so those stay generic.
- A projector on `GammaVout`'s first (`i`) operand stays generic: the fused
  kernels put the projected fermion at the second operand.
- An outer projector (`ProjX(GammaIout(..))`) is normalised to the flipped inner
  one, `ProjX ∘ slash = slash ∘ ProjX̄`; this identity is bit-exact because the
  Weyl slash maps chiral storage blocks crosswise.
- Terms are grouped across **all** of the vertex's coupling terms by the tree
  rendered with the projector replaced by a hole. A group fuses only when both
  chiralities are present. The fused node takes `g_L` and `g_R` as scalar
  sub-graphs `Σ coupling·coeff` over the group's left and right members, and the
  kernel forms `g_L·(left) + g_R·(right)` directly.
- FFV1 (plain `γ^μ`, no projector) and a lone chirality (FFV2 alone) do not
  form a pair and lower generically. The `[1, 1]` coordinate in the table above
  is conceptual for FFV1.

The fused kernels (`ffv_vout`, `ffv_iout`, `ffv_oout` in `helas/eval/kernel.rs`)
reorder floating-point operations relative to the generic composition, so
agreement is approximate (≲1e-15 per kernel). The tests
`ffv_vout_matches_generic_chiral_pair` (both operand orders, so the
reversed-line coupling swap is covered),
`ffv_fermion_out_matches_generic_chiral_pair` and
`outer_projector_equals_flipped_inner_bit_exactly` certify them[^code-kernel].

A second, separate peephole works one level down, on the compiled helicity
program (binary `Mul` into typed instruction variants); see
[performance/evaluator-program-layout](../performance/evaluator-program-layout.md).
The egglog stage in `helas/eval/egraph.rs` is a round-trip seam with no
rewrite rules and no production consumer
([performance/egglog-rewrite-stage](../performance/egglog-rewrite-stage.md)).

## EFT structures fall through to the general tier

The general tier is the extension point for non-renormalizable structures. It
now hosts the primitives SMEFT and the toy models need: `Gamma5` /
`Gamma5Amp`, `EpsilonVout` / `EpsilonAmp`, the literal-`Sigma` nodes
(`SigmaVout`, `SigmaVoutRev`, `SigmaMv`, `SigmaOut`, `SigmaOutRev`) and the
cyclic four-fermion path (`FierzOut`, `FierzOutRev`, `MultivectorIout`,
`MultivectorOout`, `FierzPair`)[^code-op][^n13-scope]. None of them has a fused kernel;
each runs generically, which is the design's intent for rare structures.

These are exercised by banked rows: SMEFTsim rows (`ee_to_zh_smeft`,
`ee_to_ttx_smeft`, …) and the toy-model rows (`ll_to_qqx_toy_dipole`,
`ll_to_qqx_toy_tensor`, …) in `validation/manifest.toml`; see
[validation/non-sm-rows](../validation/non-sm-rows.md) and
[validation/toy-ufo-models](../validation/toy-ufo-models.md). The conventions
the new nodes carry are in
[gamma-chains-gamma5-and-epsilon](../amplitudes/gamma-chains-gamma5-and-epsilon.md),
[levi-civita-and-sigma-conventions](../amplitudes/levi-civita-and-sigma-conventions.md)
and [four-fermion-vertices](../amplitudes/four-fermion-vertices.md).

What the general tier still refuses is the rooting's business, not this
design's: structures with charge conjugation or too many free indices fail
with `RootLorentzError::UnsupportedVertex`, and an unrecognised index cycle is
refused by name (`root_lorentz.rs`).

Adding a kernel for an EFT structure is a performance decision: add it when a
profile shows the generic path matters, with its own fused == generic test.

## Propagators

Propagators are not fused into vertex kernels; see
[propagator-separate-from-vertex](../amplitudes/propagator-separate-from-vertex.md).

[^n13-summary]: Note 13 §0, the two-tier summary.
[^n13-primitives]: Note 13 §1a, the FFV coordinate table and the ALOHA-as-oracle stance.
[^n13-basis]: Note 13 §2–§2b. The node names in the §2b table there (`LowerVout`, "P-node") predate the current node set; the table above is re-derived from `root_lorentz.rs` and `op.rs`.
[^n13-catalog]: Note 13 §3–§3a.
[^n13-scope]: Note 13 §6 recorded `Epsilon`, `Sigma` and arbitrary-grade momenta as a documented but unbuilt extension point; they are now implemented and gated, and the general-tier-as-extension-point decision stands.
[^code-lower]: `vibegraph-lib/src/helas/eval/lower.rs`, `ChiralSite`, `chiral_gamma_site`, `lower_vertex`.
[^code-kernel]: `vibegraph-lib/src/helas/eval/kernel.rs`, fused chiral FFV kernels and their tests.
[^code-op]: `vibegraph-lib/src/helas/eval/op.rs`, `enum Op`.
