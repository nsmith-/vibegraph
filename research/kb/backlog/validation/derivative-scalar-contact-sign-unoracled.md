---
type: Backlog Item
title: A derivative all-scalar vertex takes no scalar-sink sign and has no oracle
description: "The scalar-sink −1 applies to all-scalar contacts with no operator; a P-carrying all-scalar structure is left unsigned, and no row compares one against MadGraph."
area: validation
state: open
priority: medium
closes_when: "A row with a derivative all-scalar vertex (as current and as amplitude) is gated per diagram against MadGraph, and the vertex's sign follows what that row shows."
blocked_by: []
opened: 2026-10-09
tags: [amplitude, sign-convention, scalar, oracle-gap, non-sm-ufo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: code, resource: "../../../../vibegraph-lib/src/helas/eval/root_lorentz.rs", title: "root_lorentz.rs ~:645-656, the pure-scalar-contact −1 and its stated gap"}
---
`root_lorentz.rs` (~:645-656) gives an all-scalar contact with no operator
(`SSS1`, `SSSS1`) the same −1 against the `−i/D` scalar propagator as the
scalar-sink bilinears. It was pinned per diagram on `ta+ ta- > t t~ h` and
`ta+ ta- > t t~ h h`. The comment says a derivative all-scalar structure, one
that carries `P` operators, "is not covered by any oracle and is left as it
was". Such a structure reaches no arm, so it is unsigned. Whether that is right
is unknown.

Derivative scalar self-couplings occur in EFT and composite-Higgs UFOs, so a
user model can reach this today with no refusal. The work: a toy-UFO
derivative `SSS` (for example `P(-1,1)*P(-1,2)`), a MadGraph standalone row
rooting it as both current and amplitude, a per-diagram comparison, then the
sign. Found by Phase 2 drafter D8; the code comment confirms it.
