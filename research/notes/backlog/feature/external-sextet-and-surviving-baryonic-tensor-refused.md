---
type: Backlog Item
title: External sextets and colour bases with a surviving ε or sextet tensor are refused
description: A process with an external sextet, or whose colour basis keeps an Epsilon/K6 tensor, is refused; LHEF colour tags cannot express it and `order_summation` is unported.
area: feature
state: open
priority: low
closes_when: A process with an external sextet (or an external baryonic ε vertex) matches MadGraph's colour matrix and JAMPs, and its events carry a documented colour-tag representation.
blocked_by: []
opened: 2026-09-07
tags: [non-sm-ufo, colour, sextet, lhef]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L918-L923", title: "TODO.md entry T078"}
---
These processes are refused:

- an external sextet;
- any colour basis key in which a baryonic (`Epsilon`/`EpsilonBar`) or sextet
  (`K6`/`K6Bar`) tensor survives reduction.

In each case three colour indices meet at a point, or one leg carries two colour
lines. A Les Houches record's two `ICOLUP` slots cannot write either.
`flow_tags::slots_for` gives a sextet no slots, and the leg is rejected upstream
(`vibegraph-lib/src/helas/color/flow_tags.rs:76`).

The two gated rows (`p3r3_to_p3r3_toy_epsilon`, `p3r3_to_p3r3_toy_sextet`) keep
the diquark internal, so their flow tags are ordinary triplet lines.

MadGraph's `order_summation` is not ported. It does nothing while `K6`/`K6Bar`
reduce away, but an external sextet needs it.

Lifting this needs three things:

- the port of `order_summation`;
- a convention for writing an external sextet into LHEF colour tags, which is
  MadGraph's choice to read, not to invent;
- a banked row.

Detail: [note 35 §T3](../../35-ufo-lorentz-sprint-plan.md).
