---
type: Backlog Item
title: A vertex mixing Dirac-matrix and matrix-free bilinears has no defined sign
description: An interaction whose Lorentz terms mix a γ-matrix bilinear with an Identity/Gamma5/projector one may carry mixed per-term rooting signs, which a uniformity assertion turns into a panic.
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

Uniformity assertions in `vibegraph-lib/src/helas/eval/diagram_eval.rs:131-169`
and `:265` enforce this. They were checked only across the MadGraph-validated
set. An `Identity` term takes the scalar-sink −1 and a `Gamma` term does not, so
a mixed vertex would most likely panic on the first assertion. Nobody has run
this, and no test confirms it.

To resolve, either:

- add such a vertex to `vibegraph_toy_UFO`, bank MadGraph's per-diagram `AMP()`,
  and derive how the signs factorise per term
  ([note 12](../../history/notes/12-helas-continuum-bugfix-journey.md) method); or
- refuse it with a typed error instead of a panic.

History of the line-sign rule: [note 35 §T3](../../history/notes/35-ufo-lorentz-sprint-plan.md).
