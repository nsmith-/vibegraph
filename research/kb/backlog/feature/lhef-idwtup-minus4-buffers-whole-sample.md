---
type: Backlog Item
title: IDWTUP = -4 output buffers the whole sample in memory
description: The weighted (IDWTUP = -4) LHEF strategy holds every event before writing; a streaming two-pass replay is designed but not built.
area: feature
state: open
priority: low
closes_when: "`generate --strategy buffer` (IDWTUP = -4) writes events in streaming fashion via a deterministic two-pass replay over `EventSource::restart`, with memory independent of event count."
blocked_by: []
opened: 2026-07-28
tags: [lhef, generate, streaming, memory]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L807-L809", title: "TODO.md entry T064"}
---
`Strategy::Buffer` (`vibegraph-cli/src/generate.rs:97`) holds the whole sample in
memory and writes the weights (`IDWTUP = -4`). The file's header needs σ before
any event is written, so streaming needs two passes. The first pass accumulates σ
and the second regenerates the identical events and writes them. The hook for that
is `EventSource::restart` (`vibegraph-lib/src/lhef/emit.rs:86`). Its contract
requires every stream reseeded and every accumulator reset. Both implementations
honour it (`generate.rs:601`, `:1670`), and the
`restarting_the_source_replays_the_same_events` test pins it.

This is not needed today: a 100k-event run buffers in about 42 MB. It becomes worth
doing when sample sizes make that footprint a constraint.
Design and close-out: [note 23](../../history/notes/23-event-output-lhef-plan.md).
