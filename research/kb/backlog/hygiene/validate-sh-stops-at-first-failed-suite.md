---
type: Backlog Item
title: "validate.sh stops at the first failed suite"
description: "pixi run validate runs cargo test without --no-fail-fast, so one missing input (a PDF set) skipped six banked suites including validate_sigma."
area: hygiene
state: open
priority: low
closes_when: "validate.sh runs every suite whatever an earlier one did and still exits non-zero on any failure, with the collator rendering every suite that ran."
blocked_by: []
opened: 2026-10-17
tags: [validation, tooling, pixi]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: f1, resource: "../../sprints/sigma-pulls-lean-high/sessions/F1-report.md", title: "sigma-pulls-lean-high F1 report, Found"}
---
On a worktree missing `validation/pdf/NNPDF31_lo_as_0130`, `validate_pdf_grid`
failed 13 of 20 tests. `cargo test` then stopped, and `validate_samples`,
`validate_scale_couplings`, `validate_scales`, `validate_sigma`,
`validate_unweighting` and `validate_vegas` never ran
([F1](../../sprints/sigma-pulls-lean-high/sessions/F1-report.md)). A
`--no-fail-fast` keeps one missing input from hiding the rest of the layer.
