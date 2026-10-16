---
type: Backlog Item
title: AmplitudeEvaluator::compile returns an error type callers cannot name
description: "EvalError lives in the private helas::eval::error module with no re-export, as do CompileError and RootLorentzError, so tests must match error messages rather than variants."
area: hygiene
state: open
priority: low
closes_when: "The evaluator's error tree is re-exported where compile's callers can name it, and the refusal tests match the variant."
blocked_by: []
opened: 2026-10-15
tags: [visibility, errors, evaluator, tests]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: ch, resource: "../../sprints/amplitude-signs/sessions/C-H-report.md", title: "C-H report, Found"}
---
Found when `toy_models.rs::mixed_sign_vertex_is_refused_at_compile` had to match
the refusal's text. Ties into the supported-surface decision in
`lib-pub-api-surface-unaudited`.
