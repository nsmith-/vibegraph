---
type: Backlog Item
title: "Whether a signed selection is allowed is decided separately at four sites"
description: "The JAMP2, signed-pruning and signed-draw branches each test is_signed() on their own (compile.rs, run.rs, engine.rs, generate.rs), and generate refuses first."
area: hygiene
state: open
priority: low
closes_when: "A decision is recorded on whether signed selections are ever generated, and the branches that decision makes dead are deleted or the live ones tested."
blocked_by: []
opened: 2026-11-24
tags: [amplitudes, squared-orders, generate]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: report, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-Z1-report.md", title: "L-Z1 report: Found 3"}
---
Found by sprint L's hygiene pass L-Z1 (Found 3); the evidence is in
[its report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z1-report.md).

Sites: `compile.rs:342`, `run.rs:718`, `engine.rs:748`, `generate.rs:458`.
