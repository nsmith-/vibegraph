---
type: Backlog Item
title: banked_outgoing_orderings computes η from the wrong momentum components
description: The dijet ordering probe computes pT as hypot(E, px) and η from py, so the η-ordering counts it prints and asserts on are wrong.
area: validation
state: open
priority: medium
closes_when: The probe `banked_outgoing_orderings` reads pT from px, py and η from pz under the [E, px, py, pz] layout, and the counts it prints are re-recorded.
blocked_by: []
opened: 2026-09-26
tags: [test-bug, four-momentum-layout, flavour-grouping, pp-to-jj]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L317-L332", title: "TODO.md entry T019"}
---
`banked_outgoing_orderings` (`vibegraph-lib/tests/validate_hadronic.rs:1937`)
reads MadGraph's banked `pp_to_jj` events and counts how often the first
outgoing leg is the more forward. Its η closure (`:1965`) computes
`pt = momentum[0].hypot(momentum[1])` and `asinh(momentum[2] / pt)`. With
`LheParticle::momentum` in `[E, px, py, pz]` order
(`vibegraph-lib/src/lhef/record.rs:176`), that is hypot(E, px) and py: not η.

The caller (`:2113`) asserts only that both orderings occur, which probably
still holds by symmetry. But the counts it prints are not what they claim, and
the assertion does not test the premise it states. Fix the closure
(`momentum[1].hypot(momentum[2])`, `momentum[3]`) and re-read the printed counts.
