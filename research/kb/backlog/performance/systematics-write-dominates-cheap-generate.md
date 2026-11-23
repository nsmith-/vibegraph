---
type: Backlog Item
title: "The systematics write is the largest phase of generate on p p > t t~"
description: "With 145 weights an event, the systematics write takes 7.5 s of 12.7 s at -j1 and 2.7 s of 4.5 s at -j4 on p p > t t~, after accept/reject and only partly parallel."
area: performance
state: open
priority: medium
closes_when: "SystematicsWriter is profiled per event, and either the PDF and scale re-evaluation moves into the blocked pass's workers (prepare) with the weights unchanged, or the measured cost is recorded with the reason it stays."
blocked_by: []
opened: 2026-11-23
tags: [performance, generate, systematics, parallelism]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: ls1, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-S1-report.md", title: "L-S1 report: Found 4"}
---
On `p p > t t~` with 145 systematics weights an event, the write takes 7.5 s
of 12.7 s at `-j 1` and 2.7 s of 4.5 s at `-j 4` (6.8 s CPU in 2.7 s wall).
It runs in the writer after the accept/reject pass.

Moving the weight computation into the blocked pass's workers needs the
systematics evaluation to take a prepared event in place of the writer's
record. Profile first: the cost may sit in the PDF re-evaluation alone.
