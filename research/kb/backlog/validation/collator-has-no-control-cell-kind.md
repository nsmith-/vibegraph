---
type: Backlog Item
title: "The validation report has no cell kind for a known-wrong control"
description: "The collator rejects a measured cell whose mode differs from the manifest cell's, so a control that must fail cannot be a cell of a gate row; controls are printed and asserted only."
area: validation
state: open
priority: low
closes_when: "The manifest and collator have a control cell kind that renders as passing when the control fails its bounds, and at least one existing control writes it."
blocked_by: []
opened: 2026-10-26
tags: [validation-report, known-wrong-control, collator]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: v2, resource: "../../sprints/flavour-mixed-masses/sessions/V2-report.md", title: "flavour-mixed-masses V2 report, Found 3"}
---
`validation-report/src/main.rs` (about :380 at the time) refuses a measured cell
whose mode differs from its manifest cell. The mixed-mass control
(`ta_part_on_massless_channels`) and the unequal-beams energy-sum control are
therefore printed and asserted in their tests but absent from the report, so a
report reader cannot see that the gate was shown to fail on a known-wrong input
([V2](../../sprints/flavour-mixed-masses/sessions/V2-report.md)).

AGENTS.md asks for known-wrong comparisons to keep running; a cell kind would
make them visible where the gates are read.
