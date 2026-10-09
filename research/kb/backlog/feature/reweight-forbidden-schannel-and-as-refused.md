---
type: Backlog Item
title: Reweighting refuses $ processes and the strong coupling
description: Reweighting refuses processes with a forbidden on-shell s-channel ($) and any hypothesis that moves aS.
area: feature
state: open
priority: low
closes_when: A $ pattern process and an aS hypothesis each reweight to MadGraph's reweight-module weights event by event, or either refusal is recorded as a permanent scope decision.
blocked_by: []
opened: 2026-10-05
tags: [reweight, s-channel, alphas]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L668-L730", title: "TODO.md entry T059"}
---
Two refusals in `vibegraph::reweight`:

- `ReweightError::ForbiddenSChannel` (`vibegraph-lib/src/reweight/engine.rs:175`):
  a `$` pattern amplitude depends on where each event sits relative to the
  veto windows, so the reweighter would need the per-event window state.
- `ReweightError::StrongCoupling` (`vibegraph-lib/src/reweight/mod.rs:179`):
  a hypothesis moving `aS` or anything depending on it. The refusal message
  treats this as a scale variation rather than a parameter reweight; whether
  MadGraph's semantics for an `aS` launch should be matched is the open
  question.
