---
type: Backlog Item
title: Decay-chain stitching asserts that one graph is never reached with opposite signs
description: "chain.rs stitch_set panics through assert_eq! if one stitched graph is reached twice with opposite signs; no input reaches it today, but it should refuse."
area: validation
state: open
priority: low
closes_when: "The assert returns the decay-chain refusal the same block already returns for an ambiguous forced line, and a hermetic test pins the refusal (or a proof that the case is unreachable replaces the assert with a debug assertion)."
blocked_by: []
opened: 2026-11-19
tags: [diagrams, decay-chains, refusal, robustness]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
sources:
  - {id: k-h2, resource: "../../sprints/v022-smeft-and-refusals/sessions/K-H2-report.md", title: "K-H2 report, Found"}
---
`assert_eq!(*sign, d.sign, "one stitched graph reached twice with opposite
signs")` in `diagrams/chain.rs` `stitch_set` predates the identical-decay
stitching change. A panic is never an acceptable outcome for a user card; the
neighbouring ambiguous-forced-line case already returns a named refusal.
