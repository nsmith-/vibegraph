---
type: Backlog Item
title: Single-helicity timing against MadGraph has no consumer
description: A fair single-helicity comparison needs an MG single-config timing, but no code path evaluates one fixed helicity in a loop, so nothing needs it yet.
area: performance
state: blocked
priority: low
closes_when: A feature that evaluates a single fixed helicity in its hot loop exists and is timed against an MG single-config run, or the comparison is dropped.
blocked_by: []
opened: 2026-07-28
tags: [madgraph-comparison, helicity, bench]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1166-L1172", title: "TODO.md entry T108"}
---
Accept/reject selects the helicity off the `eval_hel_m2` diagonal: one
helicity-summed evaluation per accepted event. Single-helicity evaluation is
therefore not a hot path, and timing it against MadGraph would answer no
current question.

A fair comparison needs an MG single-config timing: editing the generated
Fortran driver and `gen_amplitude.py`, then regenerating reference data.
Re-sequence this under whatever first needs a single fixed helicity in a loop.

Detail: [note 23 §E2](../../history/notes/23-event-output-lhef-plan.md).
