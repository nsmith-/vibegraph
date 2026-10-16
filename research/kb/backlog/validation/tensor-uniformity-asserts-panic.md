---
type: Backlog Item
title: A vertex mixing a cyclic tensor term with a tree-rooted term panics at compile
description: "VertexTerm::tensor and VertexInfo::tensor panic on a vertex mixing a cyclic tensor term with a tree-rooted term under one coupling order; the sign asserts beside them now return errors."
area: validation
state: open
priority: low
closes_when: "Those asserts return an error naming the vertex, as the mixed-sign ones do, and a hermetic test pins the refusal (or the case compiles correctly)."
blocked_by: []
opened: 2026-10-15
tags: [evaluator, refusal, ufo, robustness]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: c2, resource: "../../sprints/amplitude-signs/sessions/C2-report.md", title: "C2 report, Found"}
---
In `helas/eval/diagram_eval.rs`. Same treatment as the mixed rooting-sign
asserts, which now go through `uniform_sign` and return
`RootLorentzError::UnsupportedVertex`.
