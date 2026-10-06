---
type: Backlog Item
title: No ping-pong t-channel ordering for ladders of three or more rungs
description: MadEvent alternates beams (tstrategy ping-pong) on ≥3-transfer ladders with massless ends; rungs here are always ordered from beam 0.
area: performance
state: open
priority: low
closes_when: Ping-pong rung ordering is measured at ≥20 seeds against one-side ordering on a ≥3-rung row (p p > j j j j class), and adopted for such ladders or rejected.
blocked_by: []
opened: 2026-09-07
tags: [phase-space-map, t-channel, madevent-parity, multijet]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1013-L1020", title: "TODO.md entry T095"}
---
MadEvent picks a `tstrategy` per configuration (`reorder_tchannels`,
`export_v4.py`). It uses ping-pong (alternate beams) when both outermost
exchanged lines are massless and the ladder has three or more transfers, and
one-side otherwise. Here rungs are always ordered from beam 0 (`spine_chain`).
`with_rung_order` exists to measure an alternative order.

No gated row has three rungs, so this is untested. llj and the 2→3 QCD rows have
at most two transfers. On two rungs, reversed order reads 1.01 ± 0.02 against
derived order (`u u~ > g g g`, twenty seeds), as MadEvent's own one-side choice
there predicts. A `p p > j j j j`-class row is needed to measure it.

Detail: [note 37 §1 and §6](../../37-madevent-map-survey-and-soft-angle.md).
