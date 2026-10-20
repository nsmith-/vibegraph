---
type: Backlog Item
title: CLI integration tests copy their helpers and card text per file
description: "vibegraph(cwd, home), stderr_of and the fixed-beam run-card text are re-implemented in several vibegraph-cli/tests files; a shared tests/common module would hold them once."
area: hygiene
state: open
priority: low
closes_when: "The CLI test files share one helper module for invoking the binary, reading stderr and writing the standard fixed-beam cards, and no file re-implements them."
blocked_by: []
opened: 2026-10-20
tags: [tests, cli, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: hy1, resource: "../../sprints/cut-refusal/sessions/HY1-report.md", title: "cut-refusal HY1 report, Found 3"}
---
`fn vibegraph(cwd, home)` and `fn stderr_of` appear in `cli_hard_errors.rs` and
`cli_first_run.rs` (the latter with a dead-proxy variant). The `0 = lpp1 … ebeam2`
card text appears in `cli_first_run.rs`, `cli_hard_errors.rs` (4 places),
`cli_fixed_energy.rs` (4 places) and `cli_generate.rs`. A
`vibegraph-cli/tests/common/mod.rs` would hold them once.
Found by the [cut-refusal hygiene review](../../sprints/cut-refusal/sessions/HY1-report.md).
