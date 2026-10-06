---
type: Backlog Item
title: The Pythia consumption gate cannot see a corrupted momentum
description: consume.py compares only the multiset of outgoing PDG codes Pythia reconstructs, and its negative control mutates one colour field on one event.
area: validation
state: open
priority: medium
closes_when: The gate compares Pythia's reconstructed process four-momenta against the written record, and has negative controls that a permuted or corrupted momentum (and more than one ICOLUP mutation) make fail.
blocked_by: []
opened: 2026-08-06
tags: [pythia, lhef, negative-control, oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L496-L505", title: "TODO.md entry T037"}
---
`validation/pythia/consume.py` (`pixi run -e pythia validate-pythia`) reads both
emitted samples n/n. Its reconstruction check compares only
`sorted(p.id for status-1 particles)` against the record. A permuted or
corrupted momentum is therefore consumed silently.

To close that blind spot, compare Pythia's `process` four-momenta against the
record's `PUP`, with a tolerance set by the precision the record is written at.

The negative control has a narrow reach too. It rewrites `ICOLUP(1)` of one
final-state parton on one event with a dangling index. Widening it should cover:
- an anticolour mutation;
- a momentum mutation, which proves the new check is not vacuous.
