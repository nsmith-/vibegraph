---
type: Backlog Item
title: A vertex mixing Dirac-matrix and matrix-free bilinears has no defined sign
description: "An interaction mixing a γ-matrix bilinear with an Identity/Gamma5/projector one can carry mixed reversed-bilinear parities; it is refused at some rootings and compiles, unchecked, at others."
area: feature
state: open
priority: low
closes_when: Either a toy-UFO row with a mixed vertex matches MadGraph per diagram and per helicity, or the case is refused with a typed error at load time.
blocked_by: []
opened: 2026-09-07
tags: [non-sm-ufo, fermion-flow, sign-convention, toy-ufo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L931-L934", title: "TODO.md entry T082"}
---
Example: a derivative-coupled `P·Gamma` term next to an `Identity` term in one
FFS interaction. No model in the tree has such a vertex.

Each vertex's reversed-bilinear parity (and tensor sign) is lifted from per-term
values that must agree (`uniform_sign` in `helas/eval/diagram_eval.rs`). Since
PR #32 a disagreement is a typed refusal (`RootLorentzError::UnsupportedVertex`),
pinned by `toy_models.rs::mixed_sign_vertex_is_refused_at_compile` on a toy
vertex with `Identity(2,1)` beside `P(-1,3)*Gamma(-1,2,1)`. The `tensor()`
asserts still panic
([tensor-uniformity-asserts-panic](../validation/tensor-uniformity-asserts-panic.md)).

**The refusal depends on rooting**, under the root-free convention too
(re-measured when PR #30 merged PR #32's hygiene commits). Where the fermion pair
sinks into the scalar (`lt~ lt > st`, and five more processes C-H listed), the
gamma term is a `GammaVout` sink and `Identity` is not, so the vertex is refused
with "mixed reversed-bilinear parities". With the scalar on leg 0
(`st > lt lt~`) neither term is a fermion→vector sink, and the vertex compiles
with a sign no oracle has checked. B′'s derived claim that the refusal no longer
depends on the rooting does not hold.

To resolve, either:

- add such a vertex to `vibegraph_toy_UFO`, bank MadGraph's per-diagram `AMP()`,
  and derive how the signs factorise per term
  ([note 12](../../history/notes/12-helas-continuum-bugfix-journey.md) method); or
- refuse it consistently, at every rooting, with a typed error.

History of the line-sign rule: [note 35 §T3](../../history/notes/35-ufo-lorentz-sprint-plan.md)
and [fermion-line-sign](../../amplitudes/fermion-line-sign.md).
