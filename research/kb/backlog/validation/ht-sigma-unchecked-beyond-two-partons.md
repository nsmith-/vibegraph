---
type: Backlog Item
title: "No σ-level check of an HT cut over three or more partons"
description: "With two partons HT = 2 pT, so the banked HT rows cannot see a vector sum or which jets enter the sum; a p p > j j j row with htjmin would be the first."
area: validation
state: open
priority: low
closes_when: "A MadGraph row such as p p > j j j with htjmin 400 is banked and its σ gates at a usable vibegraph cost."
blocked_by: [ht-cuts-give-channels-no-regulator]
opened: 2026-10-26
tags: [sigma, ht-cuts, madgraph-reference]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f1, resource: "../../sprints/ht-cuts/sessions/F1-report.md", title: "ht-cuts F1 report, Found 3"}
---
`pp_to_jj_ht` and `pp_to_bb_iht` each have two final partons, so HT is `2 pT`:
they see the edges, the `shat_min` floor and whether `iht` takes the b class,
but not a scalar against a vector sum, nor which jets a sum of three or more
takes. The per-event unit tests in `cuts.rs` pin those; no σ does.

F1 ran `p p > j j j` with `htjmin 400` once (177154.4 ± 605.2 pb, 720k
evaluations, 12 of 15 channels accepting under a quarter of their points)
([F1](../../sprints/ht-cuts/sessions/F1-report.md)). That cost is why the
row waits on the regulator item.
