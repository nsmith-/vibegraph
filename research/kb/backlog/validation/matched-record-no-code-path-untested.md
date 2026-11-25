---
type: Backlog Item
title: "matched_event_record's path for a line with no resonance code is untested"
description: "No test builds a matched (ickkw = 1) record for a group member whose line carries no code; mutation M20 survives."
area: validation
state: open
priority: low
closes_when: "A test builds such a record and asserts no status-2 line and the warning count, failing on M20."
blocked_by: []
opened: 2026-11-24
tags: [lhef, resonance, mlm, tests]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z3-report.md", title: "L-Z3 report: F16"}
---
Found by sprint L's hygiene pass L-Z3 (F16); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z3-report.md).

PR 15 added the no-code path; its unmatched twin is tested.
