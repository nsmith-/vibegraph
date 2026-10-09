---
type: Backlog Item
title: pp_to_jj's scale differs across flavour groups at 5e-7
description: p p > j j's scale spread over all (group, configuration) pairs is 4.999999e-7 while every within-group spread is exactly zero; too large to be rounding.
area: validation
state: open
priority: low
closes_when: The source of the across-group difference is located and either removed or explained (with the explanation pinned by a test), so a group-axis change cannot expose it unannounced.
blocked_by: []
opened: 2026-08-03
tags: [scales, flavour-groups, pp-to-jj, chain-b]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L385-L390", title: "TODO.md entry T026"}
---
Chain B-0's census measured the scale of `p p > j j`:
- The spread within each of the eight flavour groups is exactly `0.0`.
- The spread over every `(group, configuration)` pair is `4.999999e-7`.

The groups therefore agree on the scale to seven digits and not beyond. That is
far too large to be rounding on a 2 → 2 whose scale ought to be
group-independent.

Nothing in production reads the group axis for scales, so this moves no cell
today. It is, however, the size of effect that a change making scales
group-dependent (note 29 §B.4) would expose.

The first step is to find where the group enters the scale computation. It is
worth first ruling out that the probe itself introduces the difference, for
example through a printed round-trip, since the value sits just under `5e-7`.

Detail: [note 29 §B-0 and Chain B results, open items](../../history/notes/29-v01-validation-sprint-plan.md).
