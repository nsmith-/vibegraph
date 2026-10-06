---
type: Backlog Item
title: Event generation has no lazy iterator API
description: Events are produced inside the CLI's generate loop; the library offers no lazy iterator that yields events on demand.
area: performance
state: open
priority: low
closes_when: The library exposes an iterator that yields unweighted events lazily, and the CLI generate path consumes it.
blocked_by: []
opened: 2026-07-17
tags: [api, event-generation, streaming, long-tail]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1372-L1373", title: "TODO.md entry T120"}
---
The `generate-stream` Part B item: a lazy `generate_*` event iterator. Event
generation runs in `vibegraph-cli/src/generate.rs` (`generate_sample`,
`generate_proton_sample`), not behind a library iterator. Diagram
enumeration (`generate_from_proc_card` → `Vec<DiagramSet>`) is a separate
question that [note 33](../../33-logging-tui-plan.md) §2 keeps apart from this one.

Long-tail; no consumer is blocked. Origin:
[note 20](../../20-eval-perf-2-plan.md).
