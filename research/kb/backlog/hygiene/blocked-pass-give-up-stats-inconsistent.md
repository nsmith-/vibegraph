---
type: Backlog Item
title: "BlockedPass's two give-up branches leave different statistics"
description: "A too-late acceptance leaves the last event's statistics and an exhausted block leaves every consumed block's; neither matches the doc, so a short-sample report depends on the branch."
area: hygiene
state: open
priority: low
closes_when: "One definition (trials consumed up to the give-up, at block granularity) is applied to both branches and pinned in a_pass_that_cannot_accept_gives_up."
blocked_by: []
opened: 2026-11-24
tags: [generate, unweighting, statistics]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z8-report.md", title: "L-Z8 report: F9"}
---
Found by sprint L's hygiene pass L-Z8 (F9); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z8-report.md).

Sites: `unweight/blocks.rs:183-186` and `:199-203`.
