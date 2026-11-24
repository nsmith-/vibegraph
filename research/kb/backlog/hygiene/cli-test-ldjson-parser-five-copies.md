---
type: Backlog Item
title: "Five CLI tests carry their own ldjson stdout parser"
description: "cli_phase_timing, cli_progress_position, cli_run_card, cli_check_only and cli_enumerate_progress each parse ldjson stdout themselves."
area: hygiene
state: open
priority: low
closes_when: "One tests/common ldjson_objects(stdout) is used by all five."
blocked_by: []
opened: 2026-11-24
tags: [tests, progress-records, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z6-report.md", title: "L-Z6 report: Found (Z6.11)"}
---
Found by sprint L's hygiene pass L-Z6 (Found (Z6.11)); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z6-report.md).
