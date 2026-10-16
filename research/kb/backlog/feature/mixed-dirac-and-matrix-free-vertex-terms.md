---
type: Backlog Item
title: A vertex mixing Dirac-matrix and matrix-free bilinears has no defined sign
description: "An interaction mixing a γ-matrix bilinear with an Identity/Gamma5/projector one may carry mixed per-term rooting signs; it is refused at some rootings and compiles, unchecked, at others."
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
FFS interaction. No model in the tree has such a vertex, and no oracle in the
suite tests it.

The `carries_dirac_matrix` assertion named in TODO.md is gone: `7f523ad`
removed it when the line sign went back to one −1 per propagator (see the fact
[fermion-line-sign-ignores-vertex-content](../../amplitudes/fermion-line-sign.md)).

The open question is now a different one. The per-vertex rooting signs are
lifted from per-term values that must agree:

- `build_sign` (scalar-sink, pure-metric and contact −1s);
- `reversed_sign` (reversed-bilinear parity);
- `tensor`.

The sign uniformity is enforced in `vibegraph-lib/src/helas/eval/diagram_eval.rs`
(`uniform_sign`, ~:300), which since PR #32 returns a typed refusal
(`RootLorentzError::UnsupportedVertex`) instead of panicking. The refusal is
pinned by `toy_models.rs::mixed_sign_vertex_is_refused_at_compile`, on a toy
vertex with `Identity(2,1)` beside `P(-1,3)*Gamma(-1,2,1)` under one order. The
`tensor()` uniformity asserts (~:149, ~:280) still panic
([tensor-uniformity-asserts-panic](../validation/tensor-uniformity-asserts-panic.md)).

The refusal depends on rooting: the hygiene review of PR #32 found that this
vertex is refused for `lt~ lt > st`, `lt~ lt > st st`, `st lt > st lt`,
`lt~ lt > st vt`, `qt qt~ > st st` and `lt~ lt > qt qt~ st`, but compiles for
`st > lt lt~`, with a sign no oracle has checked.

To resolve, either:

- add such a vertex to `vibegraph_toy_UFO`, bank MadGraph's per-diagram `AMP()`,
  and derive how the signs factorise per term
  ([note 12](../../history/notes/12-helas-continuum-bugfix-journey.md) method); or
- refuse it consistently, at every rooting, with a typed error.

History of the line-sign rule: [note 35 §T3](../../history/notes/35-ufo-lorentz-sprint-plan.md).
