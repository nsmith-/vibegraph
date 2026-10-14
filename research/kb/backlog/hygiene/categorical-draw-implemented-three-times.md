---
type: Backlog Item
title: "Three categorical draws with different rounding fallbacks"
description: "select_index, phasespace::select_channel and a test's hand-rolled draw implement the same categorical step with different fallbacks."
area: hygiene
state: open
priority: low
closes_when: "One categorical draw serves select.rs's selections and the channel draw, with its fallback rule stated and tested."
blocked_by: []
opened: 2026-10-10
tags: [sampling, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-d-report, resource: "../../sprints/hygiene/sessions/R-D-report.md", title: "R-D report (hygiene sprint)"}
---
`select.rs`'s doc now names the second draw (fixed in the hygiene sprint); the unification is open (R-D.16).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-D](../../sprints/hygiene/sessions/R-D-report.md).
