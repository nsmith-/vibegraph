---
type: Backlog Item
title: A sextet generator carrying adjoint indices is refused
description: "`T6(a,i,j)` with adjoint indices is refused in colorize: its expansion into `K6 T K6Bar` and the crossing rule for `T6` are not pinned by any row."
area: feature
state: open
priority: low
closes_when: A banked MadGraph row with a sextet–gluon vertex (a `T6` with an adjoint index) and one with a sextet `Identity` gate the colour matrix and per-flow JAMPs.
blocked_by: []
opened: 2026-09-07
tags: [non-sm-ufo, colour, sextet]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L913-L917", title: "TODO.md entry T077"}
---
Colorize reduces only the sextet delta `T6(i,j)`. A `T6` carrying adjoint
indices returns `ColorAlgebraError::Unsupported`
(`vibegraph-lib/src/helas/color/colorize.rs:219`). Any model with a coloured
sextet that couples to gluons hits this refusal.

MadGraph expands the sextet generator into `K6 T K6Bar` and takes the fresh
summed indices from a module-global counter. So the index numbering depends on
evaluation order, and a port has to reproduce or neutralise it. Unit tests cover
the algebra (`δ6(i,i) = 6`), but no banked row carries a sextet `Identity`, so
nothing pins the all-incoming crossing rule for `T6`. The crossing rules that
are pinned (`Epsilon ↔ EpsilonBar`, `K6 ↔ K6Bar`) are in
[note 35 §T3 and §10.1](../../history/notes/35-ufo-lorentz-sprint-plan.md).

To close this: extend `vibegraph_toy_color_UFO` with a sextet–gluon vertex, bank
its oracle, and port the expansion.
