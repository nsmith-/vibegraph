---
type: Backlog Item
title: Four-momenta and cross sections carry no typed physical units
description: Momenta, energies and cross sections are bare floats; no study of typed-unit crates (uom, dimensioned, units) has been done.
area: feature
state: open
priority: low
closes_when: A written study compares uom, dimensioned and units for typed four-momenta and cross sections, ending in a decision to adopt one (with a scope) or to stay on bare floats.
blocked_by: []
opened: 2026-08-06
tags: [typed-units, type-system, research]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L833-L834", title: "TODO.md entry T069"}
---
Four-momenta (`[E, px, py, pz]` in GeV, natural units) and cross sections (pb) are
untyped floats throughout `vibegraph-lib`. No units crate is a dependency today.

Research `uom`, `dimensioned` and `units` for typed four-momenta and cross
sections. Weigh these points:

- whether the crates compose with the scalar-generic `F` the Lorentz layer is
  written over;
- the cost in the hot evaluation loops (zero-cost or not);
- how GeV⁻² ↔ pb conversion would be expressed.

The deliverable is a decision, not a migration.
