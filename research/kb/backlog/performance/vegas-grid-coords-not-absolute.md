---
type: Backlog Item
title: VEGAS grid coordinates are relative to each point's window
description: Invariants are binned on window-relative coordinates, so a cut edge is not a fixed grid location as it is under MadEvent's sample_get_x.
area: performance
state: open
priority: medium
closes_when: Absolute grid coordinates ship behind --map-grid-coords (with an auto setting), with the eager path bit-identical, and are measured on pp_to_llj and pp_to_jj at ≥5 seeds against the relative grid.
blocked_by: []
opened: 2026-09-07
tags: [vegas, phase-space-map, cut-edge, madevent-parity, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1004-L1012", title: "TODO.md entry T094"}
---
The user asked for this as an option. Bin each invariant's VEGAS coordinate on
the absolute `s/s_tot` or `−t/s_tot` scale, with the draw restricted to the
point's window, as MadEvent's `sample_get_x` does. A cut edge then sits at a
fixed grid location. Today the coordinate is relative to the window, which moves
from point to point. No `--map-grid-coords` option exists yet (checked
2026-10-06).

The design needs two changes before the flag:
- invert the VEGAS↔channel contract: the channel drives the grid one coordinate
  at a time, and the eager path stays bit-identical;
- make every analytic map a fixed transform with the window inverted through it,
  so the mixture density stays grid-free.

Then add `--map-grid-coords` and its `auto`. Look for the payoff on the cut-edge
rows, `pp_to_llj` and `pp_to_jj`.

Detail: [note 37 §5.2](../../history/notes/37-madevent-map-survey-and-soft-angle.md).
