---
type: Backlog Item
title: w+ w- > e+ e- disagrees with MadGraph standalone on the neutrino exchange sign
description: With a W pair at the anchor and a final-state fermion line, the t-channel fermion exchange has the wrong sign relative to the γ/Z s-channel.
area: validation
state: open
priority: high
closes_when: The row `wpwm_to_epem` (and `w- w+ > e- e+`, `w+ w- > u u~`) match MadGraph standalone per diagram, and the known-disagreement banking in `standalone_jamps.rs` becomes an agreement check.
blocked_by: []
opened: 2026-09-25
tags: [amplitude, sign-convention, standalone, electroweak]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L298-L302", title: "TODO.md entry T014"}
---
`w+ w- > e+ e-`, `w- w+ > e- e+` and `w+ w- > u u~` disagree with MadGraph
standalone: the neutrino (down-quark) t-channel exchange carries the wrong sign
relative to the photon and Z s-channel diagrams. The crossings
`e+ e- > w+ w-` and `u u~ > w+ w-` agree, as do `w+ w- > w+ w-`,
`w+ w- > w+ w- z` and the QCD analogue `g g > u u~`. So the error needs a W
pair at the anchor *and* a final-state fermion line.

A constant sign on the γWW/ZWW vertex fixes it but breaks `w+ w- > w+ w- z`
([note 39 §4.4](../../history/notes/39-vector-vertex-signs.md)), so it is not the fix. The
next place to look is the final–final fermion line with chiral `FFV2`
vertices, per diagram against MadGraph's `AMP()` (per-diagram × per-helicity
complex dumps, [note 12](../../history/notes/12-helas-continuum-bugfix-journey.md)).

Banked as a known disagreement: `validation/madgraph/standalone/wpwm_to_epem.json`,
checked in `vibegraph-lib/tests/standalone_jamps.rs:269`. Detail:
[note 39 §5](../../history/notes/39-vector-vertex-signs.md).
