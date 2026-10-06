---
type: Backlog Item
title: Majorana fermions and charge conjugation are unsupported
description: Fermion-flow handling assumes Dirac-continuous lines and the UFO `C` operator is refused, so no Majorana or fermion-number-violating model can be evaluated.
area: feature
state: open
priority: low
closes_when: A process with an internal or external Majorana fermion (e.g. a neutralino) and one with a fermion-number-violating vertex match MadGraph per diagram and per helicity on banked rows.
blocked_by: []
opened: 2026-09-05
tags: [non-sm-ufo, descoped-v1, majorana, fermion-flow]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L905-L912", title: "TODO.md entry T076"}
  - {id: todo-t068, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L827-L832", title: "TODO.md entry T068"}
---
Fermion-flow handling assumes every fermion line is Dirac-continuous end to end
and has no flow-flip machinery. Rooting refuses the UFO charge-conjugation
operator `C` with "Charge conjugation is deferred to future work"
(`vibegraph-lib/src/helas/eval/root_lorentz.rs:580`). Descoped by user decision
D2 ([note 35 §7](../../35-ufo-lorentz-sprint-plan.md)). MadGraph itself refuses
Majorana fermions in four-fermion vertices, so those can stay refused.

This is why `vibegraph_toy_color_UFO` is all-scalar. Two fermions in the same
representation reach a diquark only through a fermion-number-violating vertex.

Expect sign trouble. Flow conventions are fragile even with Dirac fermions only:
see the explicit fermion-flow slot swap in
[note 16](../../16-color-flow-design.md). Also, the reversal-sign rule in
[note 35 §10.1](../../35-ufo-lorentz-sprint-plan.md) was revised later (fact
[fermion-line-sign-ignores-vertex-content](../../facts/fermion-line-sign-ignores-vertex-content.md)).
Validate per diagram and per helicity against MadGraph's `AMP()`, following
[note 12](../../12-helas-continuum-bugfix-journey.md), before trusting any |M|².
