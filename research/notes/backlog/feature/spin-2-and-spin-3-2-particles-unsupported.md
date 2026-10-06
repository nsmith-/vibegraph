---
type: Backlog Item
title: Spin-2 and spin-3/2 particles are unsupported
description: UFO spin codes 4 (spin-3/2) and 5 (spin-2) have no external wavefunction or propagator, so a model with a gravitino or graviton cannot be evaluated.
area: feature
state: open
priority: low
closes_when: A process with an external and an internal spin-2 particle, and one with a spin-3/2 particle, evaluates and matches MadGraph per helicity on a banked row.
blocked_by: []
opened: 2026-09-05
tags: [non-sm-ufo, descoped-v1, spin, helas]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L899-L904", title: "TODO.md entry T075"}
  - {id: todo-t068, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L827-L832", title: "TODO.md entry T068"}
---
`Particle::helicity_states` (`vibegraph-lib/src/ufo/particles.rs:91`) lists the
five spin-2 helicities for code 5. Nothing downstream builds a tensor external
wavefunction or a spin-2 propagator. Spin-3/2 (code 4) returns `None`, which
`compile.rs` turns into `EvalError::UnsupportedSpin`
(`vibegraph-lib/src/helas/eval/compile.rs:206`). Ghost codes (negative) do not
matter at LO.

Descoped by user decision D2 ([note 35 §7](../../35-ufo-lorentz-sprint-plan.md)).
The graded `1 + 4 + 6 + 4 + 1` Dirac-basis `Multivector` built for `ufo-lorentz`
holds the *antisymmetric* rank-2 tensor only. A spin-2 field is a symmetric
Lorentz tensor, so it needs its own representation, wavefunctions (`txxxxx`),
propagator and vertex rooting. Spin-3/2 needs a vector-spinor
(`irxxxx`/`orxxxx`) and has Majorana cases of its own. That links it to the
Majorana item, `majorana-fermions-unsupported.md`.

Adding a spin-2 or spin-3/2 field to the toy UFO would give an oracle row with
nothing else in the vertex.
