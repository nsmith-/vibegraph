---
type: Physics Convention
title: Colour tensors under crossing; epsilon and sextet atoms
description: "Undoing feyngraph's all-incoming crossing transposes each T per tensor and swaps Epsilon↔EpsilonBar, K6↔K6Bar; how epsilon and sextet atoms flow through grammar and basis; what stays refused."
status: draft
tags: [colour, crossing, sextet, epsilon, toy-ufo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: code-convert, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/color/colorize.rs#L140-L340", title: "colorize.rs: convert_expr, slot_indices, check_slot_reps"}
  - {id: code-tensor, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/color/tensor.rs#L50-L90", title: "tensor.rs: TensorKind order and ColorTensor atoms"}
  - {id: n35-c1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L432-L474", title: "Note 35 §3.5: the four-quark contact's crossing and gg_to_gg_cg's exact colour"}
  - {id: n35-t3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1042-L1112", title: "Note 35 T3: d, Epsilon and sextet colour"}
  - {id: n35-z, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1281-L1358", title: "Note 35 §10.1: conventions pinned by the close-out"}
  - {id: fact-line-sign, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/facts/fermion-line-sign-ignores-vertex-content.md#L12-L35", title: "Fact: the d-colour row's sign was the SSS1/SSSS1 scalar-sink −1"}
---

# Colour tensors under crossing; epsilon and sextet atoms

## The rule

feyngraph presents every external leg in the **all-incoming** crossing. A UFO
colour string `T(a…, i, j)` puts the interaction's `3` field in slot `i` and its
`3̄` in slot `j`; MadGraph indexes a `T`'s fundamental slot by the leg that carries
a `3` index in the **all-outgoing** crossing — the opposite arrow. So the leg
standing in a `3` slot under feyngraph's crossing is exactly the one MadGraph puts
in the antifundamental position. `convert_expr` (`helas/color/colorize.rs`)
undoes this **per tensor**, as it substitutes slot indices:[^code-convert]

| UFO atom | becomes |
|---|---|
| `T(a…, i, j)` | `T(a…, j, i)` — the fundamental pair transposed; adjoint indices untouched |
| `T6(i, j)` (sextet delta) | `T6(j, i)` |
| `Epsilon(a, b, c)` | `EpsilonBar(a, b, c)` |
| `EpsilonBar(a, b, c)` | `Epsilon(a, b, c)` |
| `K6(m, i, j)` | `K6Bar(m, i, j)` |
| `K6Bar(m, i, j)` | `K6(m, i, j)` |
| `Tr`, `f`, `d` | unchanged (adjoint indices are self-conjugate) |

An `Epsilon` stands on three `3` slots and an `EpsilonBar` on three `3̄`; a `K6`
on a `6` with two `3̄` and a `K6Bar` on a `6̄` with two `3`. Under the crossing every
one of those legs is the opposite kind to the one MadGraph reads, so the whole
tensor becomes its conjugate. `check_slot_reps` enforces the premise: an atom
standing on a summed index or on a slot of the wrong rep means the vertex is
written under a convention this engine does not read, and it is rejected rather
than crossed into a wrong answer.

## Why each piece is needed

- **Transposing `T`.** Reading the pair straight through complex-conjugates the
  colour string. That is invisible to the CF matrix and to purely-rational
  T-chain terms, but it flips the imaginary `f → trace` coefficients relative to
  T-chain ones (`g g > t t~`, pinned by
  `tests/color_cf.rs::gg_to_ttx_flow_structures_untransposed`), and it splits a
  basis that mixes both readings into a structure and its conjugate.
- **Per tensor, not per slot.** A positional swap of a vertex's single `3` slot
  with its single `3̄` slot has no answer at a vertex with two of each. On
  `u u~ > t t~` with a four-quark contact the contact was left uncorrected and the
  basis held a structure and its conjugate (the colour matrix met
  `δ₁₂δ₄₃δ₁₂δ₄₃`, which MadGraph's own `color_algebra.py` also refuses to reduce).
  Per tensor, `uux_to_ttx_4f` has `NCOLOR = 2`, `CF = [[9,3],[3,9]]` and basis keys
  literally MadGraph's `T(2,1)T(3,4)` / `T(2,4)T(3,1)`.[^n35-c1]
- **Swapping `Epsilon ↔ EpsilonBar` and `K6 ↔ K6Bar`.** The ε–ε̄ contraction
  produces `T` products. Taking them unconjugated while every `T` in the same basis
  is conjugated splits the basis into a structure and its transpose; without the
  swap the `p3 r3 > p3 r3` rows' colour matrix stops reducing to a scalar — the
  same `set_Nc` failure the toy model's docstring records for MadGraph — and that
  is a standing gated failure.[^n35-t3]

## Epsilon and sextet atoms end to end

The toy colour UFO (`vibegraph_toy_color_UFO`, see
[validation/toy-ufo-models](../validation/toy-ufo-models.md)) carries them through
every layer: the colour grammar ([model/ufo-string-grammars](../model/ufo-string-grammars.md)),
`ColorTensor`, the ported `color_algebra.py` reduction rules, colorize, and
`ColorRep::Sextet`/`AntiSextet` with `Identity(6,6̄) → T6` (sextet slot first).

`TensorKind` is ordered by MadGraph's class-name sort, `ColorOne < Epsilon <
EpsilonBar < K6 < K6Bar < T < T6 < Tr < d < f`, so sorted basis keys come out in
MadGraph's JAMP order.[^code-tensor]

Gated rows (all `CF max_rel = 0` and `JAMP max_rel = 0` against MadGraph):

| row | what it separates |
|---|---|
| `p3r3_to_p3r3_toy_epsilon` | basis `T(3,1)T(4,2)` / `T(3,2)T(4,1)`, JAMP columns `+1/−1` (antisymmetric triplet); `G = −i` |
| `p3r3_to_p3r3_toy_sextet` | the same basis, JAMP columns `½/½` (symmetric sextet) |
| `qqx_to_o8o8_toy_dcolor` | the only basis built from `d(1,2,3)`: coefficients `−2·Nc⁻¹, +1, +1` on `T(2,1)Tr(3,4)` / `T(3,4,2,1)` / `T(4,3,2,1)`, matching MadGraph's `get_color_amplitudes()` |

The two `p3r3` rows separate the two representations by the colour atom alone.
Both keep the diquark internal, so their LHEF colour tags are ordinary triplet
lines.

**The `d`-colour row's disagreement was never colour.** Its coefficients were
MadGraph's from the start. The amplitude-level sign was a missing scalar-sink
`−1` on the operator-free all-scalar vertices `SSS1`/`SSSS1`; see
[fermion-line-sign](fermion-line-sign.md).[^fact-line-sign] Likewise
`gg_to_gg_cg`'s colour is exact — a per-graph check of every `JAMP(i) = … AMP(j)`
line, `TMP_JAMP` expansion included, passes at `max_rel = 0` with no rephasing on
all 27 columns — and its residual was the four-gluon contact's build sign
([vector-vertex-signs](vector-vertex-signs.md)).[^n35-c1]

Two oracle details that came out of chasing those rows (`tests/color_cf_oracle.rs`):
one normalising unit per *graph*, shared across that graph's colour structures
(`normalise_group`), and `graph_unit_flips`, which requires every graph's unit to
be real relative to the subprocess's modal fourth root of unity. Together they
make a per-structure sign on a multi-structure vertex (the four-gluon contact's
three `AMP()` from one graph) visible. `color_flow_tags_oracle` compares
colour-line labels and never normalises by a phase.

## Still refused, on purpose

- A `T6` carrying adjoint indices: the sextet generator's expansion into
  `K6 T K6Bar` draws fresh summed indices from a module-global MadGraph counter
  ([sextet-generator-with-adjoint-indices-refused](../backlog/feature/sextet-generator-with-adjoint-indices-refused.md)).
- Any basis key in which a baryonic or sextet tensor survives reduction, and any
  external sextet: three colour indices tied at a point, or two colour lines on
  one leg, which no Les Houches `ICOLUP` pair can write. MadGraph's
  `order_summation` is not ported (a no-op while `K6`/`K6Bar` reduce away)
  ([external-sextet-and-surviving-baryonic-tensor-refused](../backlog/feature/external-sextet-and-surviving-baryonic-tensor-refused.md)).

**Not pinned by any gate:** the `T6` crossing (no banked row carries a sextet
`Identity`; the algebra is unit-tested, e.g. `δ6(i,i) = ½Nc(Nc+1) = 6`).

Writing a toy UFO that MadGraph reads with the same conventions:
[model/ufo-authoring-for-madgraph](../model/ufo-authoring-for-madgraph.md). The
basis and CF construction these atoms feed:
[madgraph-colour-factorization](madgraph-colour-factorization.md),
[colour-flow-evaluator](colour-flow-evaluator.md). Rule 4 of the conventions note
35's close-out pinned is this crossing swap.[^n35-z]

[^code-convert]: `vibegraph-lib/src/helas/color/colorize.rs`, `convert_expr` and `check_slot_reps`.
[^code-tensor]: `vibegraph-lib/src/helas/color/tensor.rs`, `TensorKind`.
[^n35-c1]: Note 35 §3.5 (C1 and E2).
[^n35-t3]: Note 35 §T3, items 2–3.
[^n35-z]: Note 35 §10.1. Its cell counts are a dated measurement, and its rule 5 (the Dirac-content line exemption) is reverted.
[^fact-line-sign]: The fermion-line-sign fact, replaced by [fermion-line-sign](fermion-line-sign.md).
