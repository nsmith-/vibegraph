---
type: Backlog Item
title: "The serial LHE writer bounds multicore generate on cheap processes"
description: "Unmatched p p > t t~ at -j 4 spends about 4 s of 10.9 s writing 249k events after the last unweight record, so it scales 2.24× against 3.38× on an MLM card."
area: performance
state: open
priority: medium
closes_when: "The write phase is profiled per event (formatting, compression, I/O), and either the event formatting moves onto the pool with output order unchanged, or the bound is recorded with the measured -j 8 table on the released binary."
blocked_by: []
opened: 2026-11-23
tags: [performance, generate, lhef, parallelism]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: ls1, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-S1-report.md", title: "L-S1 report: four-core table"}
---
Sprint L made generate's accept/reject a deterministic blocked pass across
the pool. The LHE write after it is still one thread. On unmatched
`p p > t t~` (249k events, 225 MB) at `-j 4`, about 4 s of the 10.9 s wall
comes after the last unweight record. That card scales 2.24× on four cores,
against 3.38× on an MLM card whose per-event sampling cost is higher.

The more threads, the larger the writer's share, so a `-j 8` number on a
cheap process measures the writer, not sampling. The systematics write
([systematics-write-dominates-cheap-generate](systematics-write-dominates-cheap-generate.md))
is a separate cost in the same phase.
