---
type: Backlog Item
title: generate ignores the graceful-stop key
description: "Only integrate_channels polls the stop signal, so under vibegraph generate the first q or ^C does nothing visible; the user decided early stop truncates where that is unbiased and refuses otherwise."
area: feature
state: open
priority: low
closes_when: "generate's accept/reject loop polls the stop signal; an early stop writes the events accepted so far, marked as truncated in the file header, wherever events are drawn i.i.d. from the full mixture, and refuses to write a file where truncation would bias it (fixed per-part event counts on mixed-multiplicity cards); both paths are tested."
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

**Decided (user, 2026-10-15):** truncate, with the header marking the sample
as truncated, when events are drawn i.i.d. from the full mixture, since the
file's normalisation then stays right. Refuse where truncation would bias the
sample: fixed per-part event counts on mixed-multiplicity cards.
