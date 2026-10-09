---
type: Backlog Item
title: A $ list that differs between process lines is refused
description: A card whose process lines carry different $ (forbidden on-shell s-channel) lists is refused, where MadGraph marks each line's own list.
area: feature
state: open
priority: medium
closes_when: A multi-line card with a different $ list per line runs, each subprocess vetoing only its own line's list, with a σ check against MadEvent on one such card.
blocked_by: []
opened: 2026-09-26
tags: [process-grammar, onshell-veto, madgraph-parity, refusal]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L309-L312", title: "TODO.md entry T017"}
---
`diagrams::forbidden_onshell_ids` (`vibegraph-lib/src/diagrams/mod.rs:310`)
resolves one `$` list for the whole card and refuses a card whose process
lines name different lists. MadGraph marks each process line's own list.

Lifting it needs the per-line marking carried per subprocess through
`onshell.rs`, which today marks, per subprocess, the s-channel lines whose
oriented id is in the single card-wide list. Flavour-group members must still
share their representative's marking (checked today). Detail:
[note 38 §4 S3](../../history/notes/38-process-grammar-sprint-plan.md).
