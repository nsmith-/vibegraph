---
type: Backlog Item
title: "The 4-jet enumeration walks keep only half of a 4-thread pool busy"
description: "W+4j and Z(→νν)+4j enumerate at user/real 2.2 on 4 threads: each 4j line walks 15.9M candidates serially, allocating per candidate, between batches it hands to feyngraph."
area: performance
state: open
priority: medium
closes_when: "The 4j line is profiled; if the candidate walk dominates, it walks the alias product without per-candidate allocation or splits it across the pool in a fixed order; and W+0–4j enumerate wall is measured before and after with the artifact digest unchanged."
blocked_by: []
opened: 2026-11-23
tags: [performance, diagram-enumeration, multijet, parallelism]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: lt3, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-T3-report.md", title: "L-T3 report: job 2 and Found 5"}
---
Release build, `-j 4`: W+0–4j enumerates in 260.7 s wall, 561.6 s user;
Z(→νν)+4j in 180.8 s wall, 407.1 s user. Each 4j line walks 15 944 049
candidates (100–130 s per line), cloning a `String` per leg and sorting a key
per candidate, and hands feyngraph only 1 284 or 1 878 of them. The walk is
serial, so the pool idles between batches.

The artifact must stay byte-identical across thread counts, so a parallel
walk has to keep its output order.
