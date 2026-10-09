---
type: Backlog Item
title: Custom UFO propagators are parsed but not evaluated
description: A particle with a custom propagators.py form is refused when it propagates in a diagram; the HELAS compiler has no path for the form.
area: feature
state: open
priority: low
closes_when: A UFO model whose propagating particle carries a custom propagator form evaluates through the HELAS compiler and matches a MadGraph standalone |M|^2 row per diagram.
blocked_by: []
opened: 2026-08-02
tags: [descoped-v1, non-sm-ufo, ufo, helas]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L589-L590", title: "TODO.md entry T051"}
---
Descoped from the v1 release goal (user, 2026-08-02). `propagators.py` (UFO
2.0) is already parsed: `vibegraph-lib/src/ufo/propagators.rs` keeps each
form's numerator and denominator as verbatim strings, and a particle carrying
one is rejected when it propagates in a selected diagram (pinned by
`vibegraph-cli/tests/cli_hard_errors.rs`, fixture particle `Zx` with
`Prop.V1`).

What remains is threading the forms through the HELAS compiler: lower the
numerator (a Lorentz structure in the propagator's two indices) and the
denominator into propagator kernels, as ALOHA does for custom propagators,
and validate per diagram against MadGraph on a model that uses one.
