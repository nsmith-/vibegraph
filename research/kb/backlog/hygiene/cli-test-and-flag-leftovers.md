---
type: Backlog Item
title: "A few CLI tests and flags still miss what they guard"
description: "An overwrite refusal test checks only the exit code, negative flag values get clap's misleading error, parse_file omits the path on parse errors, and the n-body σ check is finite-only."
area: hygiene
state: open
priority: low
closes_when: "Each listed test asserts its message or a reference σ, the two flags accept negative numbers into their own refusal, and parse_file wraps parse errors with the path."
blocked_by: []
opened: 2026-10-10
tags: [cli, non-vacuity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: f-cli-report, resource: "../../sprints/hygiene/sessions/F-CLI-report.md", title: "F-CLI report (hygiene sprint)"}
---
F-CLI Found 1–5 (the efficiency bound's 2× headroom rests on two fixed-seed readings).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [F-CLI](../../sprints/hygiene/sessions/F-CLI-report.md).
