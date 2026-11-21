---
type: Backlog Item
title: "The classification table's value-refusal list is hand-kept"
description: "VALUE_REFUSALS in runcard/report.rs lists the consumed fields that refuse some values by hand, so a new value refusal fails no test."
area: hygiene
state: open
priority: low
closes_when: "The refusal text comes from the field class or the RunCardError variant, so the table reads it, and a test fails when a value refusal has no table entry."
blocked_by: []
opened: 2026-11-19
tags: [run-card, classification-table, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Found by K-C1 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-C1-report.md), Found).

**K-H3 (PR #159), Found 2:** a new `RunCardError` refusal for a consumed
parameter gets no `VALUE_REFUSALS` row unless one is written by hand, and no
test notices. A `parameter()` accessor on `RunCardError` plus a test that
every variant naming a parameter has a row would close it.
