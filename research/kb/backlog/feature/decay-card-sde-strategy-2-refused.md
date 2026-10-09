---
type: Backlog Item
title: A decay card with SDE_strategy = 2 is refused
description: A 1 → n decay card with sde_strategy = 2 is refused because channel forests are 2 → n only, while MadGraph runs the card.
area: feature
state: open
priority: low
closes_when: What MadEvent does with a user's sde_strategy = 2 on a decay is recorded, and this side matches it (runs, or overrides to 1) instead of refusing.
blocked_by: []
opened: 2026-09-25
tags: [process-grammar, decay, sde-strategy, madgraph-parity, refusal]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L337-L339", title: "TODO.md entry T021"}
---
A `1 → n` decay card with `sde_strategy = 2` is refused: the channel forests
that weight configurations at strategy 2 are built for 2 → n only. MadGraph runs
such a card.

Settle the target first. The note reads `banner.py:5045` as forcing
`sde_strategy = 1` on a decay's default card. If MadEvent also overrides a
user's 2, the fix is to apply the same override rather than build decay
forests. If it honours the 2, decay channel forests are needed. A one-card
MadEvent run answers it. Detail:
[note 38 §4 D1](../../history/notes/38-process-grammar-sprint-plan.md).
