---
type: Backlog Item
title: "The 'already exists (pass --force)' refusal is hand-written five times"
description: "integrate, inspect, generate, enumerate and parameters each write their own overwrite refusal."
area: hygiene
state: open
priority: low
closes_when: "One helper in the CLI's error.rs builds the refusal for all five commands."
blocked_by: []
opened: 2026-11-24
tags: [cli, refusal, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z5-report.md", title: "L-Z5 report: Found 5"}
---
Found by sprint L's hygiene pass L-Z5 (Found 5); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z5-report.md).

Sites: `integrate.rs:971`, `inspect.rs:76`, `generate.rs:421`, `enumerate.rs:519`, `parameters.rs:143`.
