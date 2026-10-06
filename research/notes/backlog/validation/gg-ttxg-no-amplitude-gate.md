---
type: Backlog Item
title: g g > t t~ g has no amplitude-level gate
description: Which colour structure multiplies which Lorentz structure is pinned per flow on gg_to_gg, uux_to_ggg and gg_to_ggg, but not on g g > t t~ g.
area: validation
state: open
priority: medium
closes_when: A standalone_jamps table for g g > t t~ g is banked and a per-flow amplitude comparison against it passes.
blocked_by: []
opened: 2026-10-01
tags: [mlm, color, amplitudes, ttx, standalone-jamps]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L270-L276", title: "TODO.md entry T011"}
---
`color_cf_oracle` (`vibegraph-lib/tests/color_cf_oracle.rs`, ~L894 and ~L999)
compares a graph's colour structures as a set, since their order is each
side's slot labelling. MadGraph puts a contact vertex's off-shell leg first, and
on `g g > t t~ g` the two orders reverse the four-gluon contact's three
structures. The CF-matrix oracle is blind to that assignment, so on this
subprocess nothing checks which colour structure multiplies which Lorentz
structure.

The assignment is pinned per flow on `gg_to_gg`, `uux_to_ggg` and
`gg_to_ggg` (banked tables in `validation/madgraph/standalone/`), not on this
subprocess. Adding a `g g > t t~ g` row to
`validation/madgraph/gen_standalone_jamps.py` and banking its table would pin it.
The subprocess feeds `pp_to_ttx_0j1j_mlm`'s `@1` (see
[ttx-mlm-at1-sigma-high](ttx-mlm-at1-sigma-high.md)).

Detail: [note 41 §4 Z1](../../41-mlm-feature-sprint-plan.md) ("Z1 close-out record").
