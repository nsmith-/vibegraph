---
type: Backlog Item
title: generate ignores the graceful-stop key
description: "Only integrate_channels polls the stop signal, so under vibegraph generate the first q or ^C does nothing visible; whether early stop truncates or refuses is undecided."
area: feature
state: needs-user
priority: low
closes_when: "The early-stop semantics for generate are decided (truncate the sample, or refuse), and generate's accept/reject loop polls the stop signal accordingly."
blocked_by: []
opened: 2026-08-06
tags: [cli, tui, generate, logging-tui]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: note33, resource: "../../history/notes/33-logging-tui-plan.md", title: "Note 33 §9.5, filed to the backlog (never carried into TODO.md)"}
---
The two-stage `q`/`^C` stop from the logging/TUI work is wired only into
`integrate_channels` ([note 33 §9.5](../../history/notes/33-logging-tui-plan.md)). Under
`generate` the first press does nothing visible. Wiring it is easy;
deciding what an early stop means is not: either write the events accepted so
far (a truncated sample whose normalisation must still be right) or refuse to
write anything.
