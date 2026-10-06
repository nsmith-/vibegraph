---
type: Backlog Item
title: Per-group reclustering dominates a matched wide-mixture point
description: A pp_to_ll_0j2j_mlm point costs 0.48 ms on 4 cores, dominated by per-group reclustering over every member and both setclscales calls.
area: performance
state: open
priority: medium
closes_when: The per-group reclustering no longer dominates a pp_to_ll_0j2j_mlm point's profile, with before/after per-point cost and seed time recorded against MadEvent's same-host figure.
blocked_by: []
opened: 2026-09-28
tags: [mlm, setclscales, reclustering, per-point-cost, madevent-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L961-L965", title: "TODO.md entry T087"}
---
After channel merging, the mixture sums 43 densities instead of 364, and a
`pp_to_ll_0j2j_mlm` point costs 0.48 ms of CPU on 4 cores. That figure was taken
under load: it is 1.4× below the quiet M6 base (0.69 ms). The per-group
reclustering is unchanged by the merge and now dominates. It runs for every
group member, through both `setclscales` calls.

End to end, MadEvent takes 101–125 s per 10000-event seed on two cores. This
side takes about seven minutes per integrated and generated seed.

The lever is sharing reclustering work across a group's members and across the
two `setclscales` calls, where the clustered history is common. Profile first to
confirm which part dominates.

Detail: [note 41 §4, M3 and "F-B Landed"](../../41-mlm-feature-sprint-plan.md).
