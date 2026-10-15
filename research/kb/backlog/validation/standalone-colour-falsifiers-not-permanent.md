---
type: Backlog Item
title: The f^abc orientation and T^aT^b order falsifiers of the t t~ + jet gate are not permanent
description: "The per-flow gate fails under a flipped triple-gluon orientation and a reversed T chain, but only by temporary mutations; only the contact permutation is a committed test."
area: validation
state: open
priority: low
closes_when: "A committed test fails the t t~ + jet rows under a flipped f^abc orientation and a reversed T^aT^b chain order, through a crate-internal hook, so the gate's sensitivity to both cannot lapse silently."
blocked_by: []
opened: 2026-10-15
tags: [standalone-jamps, colour, falsifier, non-vacuity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: g1, resource: "../../sprints/amplitude-signs/sessions/G1-report.md", title: "G1 report, the falsifier table"}
  - {id: gh, resource: "../../sprints/amplitude-signs/sessions/G-H-report.md", title: "G-H report, Found 3"}
---
`gg_to_ttxg_sees_the_contact_colour_assignment` keeps the four-gluon contact's
falsifier under test through the public `Vertex::couplings`. The other two
falsifiers, G1's `f^{abc}` flip (worst flow 0.57–2.1) and `T^a T^b` reversal in
`color/tensor.rs::t_t_pair` (1.0–2.1), need crate-internal access, so they were
run once and reverted.
