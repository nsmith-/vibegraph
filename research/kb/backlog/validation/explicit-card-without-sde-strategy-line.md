---
type: Backlog Item
title: "A hand-written run card with no sde_strategy line gets 2 from MadGraph at launch on single-colour processes"
description: "check_card_consistency sets SDE_strategy from single_color whenever the card does not set it; vibegraph keeps the template's 1, changing configuration draws and cluster scales."
area: validation
state: open
priority: medium
closes_when: "Parsing a card without an sde_strategy line applies MadGraph's launch-time rule (common_run_interface.py:6574-6580), pinned against a MadGraph launch of such a card."
blocked_by: []
opened: 2026-11-09
tags: [run-card, sde-strategy, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-09}
sources:
  - {id: ir1, resource: "../../sprints/v02-last-refusals/sessions/I-R1-report.md", title: "I-R1 report"}
---
Found by I-R1. The absent-card default now follows
`create_default_for_process`. An explicit card that omits the line is
different: MadGraph's launch fills it in from `single_color`. This affects
hand-written cards such as `p p > e+ e-`.
