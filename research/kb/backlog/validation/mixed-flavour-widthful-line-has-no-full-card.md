---
type: Backlog Item
title: "No full card exercises a widthful s-channel line whose flavour differs between group members"
description: "PR 15's per-member resonance codes are pinned by unit tests and a massless-line card (894); no released card writes a widthful line that differs by member, so that path has no end-to-end check."
area: validation
state: open
priority: low
closes_when: "A card with a widthful s-channel line differing in flavour between flavour-group members is found among the released cards or built, generated, and its intermediates and colours compared with MadEvent's events."
blocked_by: []
opened: 2026-11-23
tags: [generate, resonance, flavour-groups, lhef, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: lg2, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-G2-report.md", title: "L-G2 report: Found F3"}
---
The absent-line case (`sprop = 0`, warning counted) and the per-member codes
are pinned in unit tests on the SM. On card 894 the line that differs is
massless, so no intermediate is written for it. A widthful line of the same
shape would exercise the written-intermediate path with mixed flavours.
