---
type: Backlog Item
title: A one-diagram DiagramSet is built by hand in seven places
description: "renumbering, rooting_soundness (four sites), amplitude_oracle and standalone_jamps each assemble a single-diagram DiagramSet inline to compile one diagram through production."
area: hygiene
state: open
priority: low
closes_when: "One DiagramSet constructor (or helper) builds a single-diagram set, and every site uses it."
blocked_by: []
opened: 2026-10-15
tags: [duplication, test-helpers, evaluator]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: qh, resource: "../../sprints/amplitude-signs/sessions/Q-H-report.md", title: "Q-H report, Found"}
---
Sites: `helas/eval/renumbering.rs:241`, `helas/eval/rooting_soundness.rs`
(~:471, :616, :748, :870), `tests/amplitude_oracle.rs:1053`, and
`tests/standalone_jamps.rs` (`GraphLevel::read`). The per-graph route that
several amplitude sessions now rely on makes this the common way to read one
diagram from production.

`renumbering::single_diagram_amplitudes_at` (PR #30) is the natural home for the shared helper.
