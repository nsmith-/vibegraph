---
type: Backlog Item
title: "The generate thread-count identity test crosses batch boundaries only by accident"
description: "the_sample_does_not_depend_on_the_thread_count spans three batches at -j 4 but less than one at -j 16, because its event count is not tied to the batch width."
area: hygiene
state: open
priority: low
closes_when: "The test derives its event count from the batch width, or asserts from its sampling line that the trials span at least two batches at every -j it compares."
blocked_by: []
opened: 2026-11-24
tags: [generate, tests, parallelism]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z8-report.md", title: "L-Z8 report: Found"}
---
Found by sprint L's hygiene pass L-Z8 (Found); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z8-report.md).
