---
type: Backlog Item
title: "Three hand-written stand-in PDF sets exist in the tests"
description: "cli_progress_position.rs, cli_generate_vanishing_group.rs and systematics/plan.rs each write their own lhagrid1 stand-in set."
area: hygiene
state: open
priority: low
closes_when: "One stand-in writer in tests/common serves all three, and proton-phase-timing-test-needs-fetched-pdf uses it."
blocked_by: []
opened: 2026-11-24
tags: [tests, pdf, hermetic, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z6-report.md", title: "L-Z6 report: Found (Z6.12)"}
---
Found by sprint L's hygiene pass L-Z6 (Found (Z6.12)); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z6-report.md).

Closing this unblocks [proton-phase-timing-test-needs-fetched-pdf](proton-phase-timing-test-needs-fetched-pdf.md).
