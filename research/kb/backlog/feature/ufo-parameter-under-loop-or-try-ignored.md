---
type: Backlog Item
title: "A Parameter defined under for, while, with or try in parameters.py is silently ignored"
description: "The loader walks module-level assignments and if statements only, so a Parameter under any other compound statement is neither loaded nor refused, on main and v0.2.2."
area: feature
state: open
priority: medium
closes_when: "Any Parameter constructor call outside a module-level assignment or an evaluated if branch is refused by name, with a test."
blocked_by: []
opened: 2026-11-24
tags: [ufo, parameters, refusal]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-U2-report.md", title: "L-U2 report: Found 1"}
---
Found by sprint L's hygiene pass L-U2 (Found 1); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-U2-report.md).

Evidence: `ast_util.rs` `conditional_constructor_names` matches only `Stmt::If`. A silently dropped parameter can leave a model's values wrong without a message.
