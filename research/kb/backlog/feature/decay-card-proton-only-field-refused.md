---
type: Backlog Item
title: A decay card with a proton-beam-only field off default is refused at parse
description: A decay run card with a proton-beam-only physics field off its default is refused at parse, although a decay ignores the field and MadGraph runs it.
area: feature
state: open
priority: low
closes_when: A decay card carrying a proton-beam-only field off its default parses and runs, the field ignored as MadEvent's setcuts.f decay branch ignores it.
blocked_by: []
opened: 2026-09-25
tags: [process-grammar, decay, run-card, madgraph-parity, refusal]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L337-L339", title: "TODO.md entry T021"}
---
The run-card parser refuses a physics field that the beam configuration would
ignore (`classes::refuse_ignored_physics`, called at
`vibegraph-lib/src/runcard.rs:437` on the card's own `lpp1`/`lpp2`). A decay
card is read before `RunCard::for_decay` turns it into fixed beams at M/2, so a
proton-beam-only field off its default is refused at parse, although a decay
never uses it. MadGraph runs the card.

The refusal exists so that a field the run would silently ignore is never
accepted. For a decay, the replacement should keep that intent: accept and
ignore as MadEvent does, or refuse with a message naming the decay. Detail:
[note 38 §4 D1](../../history/notes/38-process-grammar-sprint-plan.md).
