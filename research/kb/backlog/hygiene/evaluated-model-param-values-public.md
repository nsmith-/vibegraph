---
type: Backlog Item
title: EvaluatedModel::param_values is a public field, so callers can break the evaluator's invariant
description: "A caller that removes or renames an entry of the pub param_values map can still reach expr::eval's unknown-parameter panic, which every other public path now guards."
area: hygiene
state: open
priority: low
closes_when: "param_values is private (read through an accessor), or every mutation goes through a method that keeps every expression's parameters defined, and expr::eval's invariant doc says so."
blocked_by: []
opened: 2026-10-20
tags: [ufo, api, visibility, panic]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: f2, resource: "../../sprints/ufo-conditional-params/sessions/F2-report.md", title: "ufo-conditional-params F2 report, Found 1"}
---
The [ufo-conditional-params sprint](../../sprints/ufo-conditional-params/sprint.md)
made `recompute`/`set_alpha_s` fallible and kept `expr::eval`'s panic as an
internal invariant whose doc lists each public path's guard. A direct write to
`EvaluatedModel::param_values` bypasses all of them. Readers outside the crate
(tests, the reweight engine, `rescale.rs`) index it; an accessor would keep them.
