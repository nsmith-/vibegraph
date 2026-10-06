---
type: Backlog Item
title: CKKW-L merging is refused
description: Run cards asking for CKKW-L merging (ktdurham, ptlund, a <clustering> record) are hard errors; only MLM matching is implemented.
area: feature
state: open
priority: low
closes_when: A card with ktdurham or ptlund generates events with MadGraph's CKKW-L merging cut and a <clustering> record, matching MadEvent's sigma and per-event record on a banked row.
blocked_by: []
opened: 2026-09-28
tags: [mlm, ckkw-l, merging, runcard]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L594-L600", title: "TODO.md entry T052"}
---
CKKW-L is a different merging scheme that shares MLM's kT clustering. The
user decided to keep it refused with its own backlog entry
([note 41-mlm §5 (b)](../../41-mlm-feature-sprint-plan.md)). Today `ptlund`
is a `Consumed(R_UNIMPL)` hard error (`vibegraph-lib/src/runcard/classes.rs:389`)
and `ktdurham` is among the unimplemented cuts (`vibegraph-lib/src/cuts.rs:392`).

The work: the `ktdurham`/`ptlund` merging cuts (MadGraph's `cuts.f:565`
has an `.and.`/`.or.` precedence defect to decide on, note 41-mlm §1.5) and
the `<clustering>` event record a shower needs to run CKKW-L. The clustering
itself (`coupling/cluster`) is already gated event by event against MadEvent.
