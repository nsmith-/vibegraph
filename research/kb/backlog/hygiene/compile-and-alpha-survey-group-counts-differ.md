---
type: Backlog Item
title: "compile and alpha_survey progress records report different group counts"
description: "When a single-particle part is dropped because its cuts zero it, the compile record counts the group and the alpha_survey record does not."
area: hygiene
state: open
priority: low
closes_when: "The drop is decided before the compile loop, so both records report the same groups, and a test on a card with a zeroed part pins the equality."
blocked_by: []
opened: 2026-11-23
tags: [progress-records, compile, alpha-survey]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: ly1, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Y1-report.md", title: "L-Y1 report: Found"}
---
The progress records' `groups` field disagrees between the `compile` and
`alpha_survey` phases on a card where a single-particle part is dropped
because its cuts zero it. The mismatch is documented but confusing to a
reader comparing phases. Deciding the drop before the compile loop removes it.
