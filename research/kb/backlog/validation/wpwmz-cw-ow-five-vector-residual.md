---
type: Backlog Item
title: wpwm_to_wpwmz_cw disagrees with MadGraph in O_W's five-vector structures
description: w+ w- > w+ w- z under O_W disagrees with MadGraph at |M|² 2.79e1 while its Standard-Model part agrees to 3.7e-12; the residual is O_W's.
area: validation
state: open
priority: high
closes_when: wpwm_to_wpwmz_cw's amplitudes cell agrees with MadGraph inside the amplitude gate's tolerance and is flipped from info to gate.
blocked_by: []
opened: 2026-09-07
tags: [non-sm-ufo, smeftsim, five-vector, amplitudes, info-cell]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L400-L412", title: "TODO.md entry T028"}
  - {id: todo-t015, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L303-L304", title: "TODO.md entry T015"}
---
`wpwm_to_wpwmz_cw` is the SMEFTsim row with `O_W` reached through a five-vector
vertex inside a diagram. Its `amplitudes` cell is `info`. The current
measurement against MadGraph:

| quantity | value |
|---|---|
| \|M\|² `max_rel` | 2.79e1 (2.20e3 before the vector-vertex sign fix) |
| JAMP2 | 2.55e1 |
| \|G\| − 1 | 1.7e-2 |

What is already established:
- The Standard-Model `w+ w- > w+ w- z` agrees with MadGraph standalone to
  3.7e-12, so what remains belongs to `O_W`'s five-vector and momentum-bearing
  contact structures ([note 39 §5](../../history/notes/39-vector-vertex-signs.md)).
- The enumeration matches MadGraph's 222 diagrams, and that `diagrams` cell is
  enforced.
- Restricting the Yang–Mills source sign to three-leg vertices moved the row
  from 2.17e3 without resolving it.

There is no per-diagram MadGraph table at 222 graphs, so the residual cannot be
localised per diagram from banked data. The bit-exact-oracle route
([note 12](../../history/notes/12-helas-continuum-bugfix-journey.md)) needs per-diagram
`AMP()` dumps of the `O_W` diagrams, or a smaller process that isolates one
five-vector structure.

The configuration-partition comparison for this row belongs with this work;
see [wpwmz-cw-config-partition-uncompared](wpwmz-cw-config-partition-uncompared.md).
