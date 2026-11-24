---
type: Backlog Item
title: "A configuration.py syntax error is reported as a couplings.py parse failure"
description: "couplings.rs maps a configuration.py parse error to CouplingError::Parse, whose text says couplings.py."
area: hygiene
state: open
priority: low
closes_when: "configuration.py parse errors have their own variant naming the file, with a test."
blocked_by: []
opened: 2026-11-24
tags: [ufo, errors, user-facing]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z5-report.md", title: "L-Z5 report: Found 7"}
---
Found by sprint L's hygiene pass L-Z5 (Found 7); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z5-report.md).

Site: `couplings.rs:88-90`.
