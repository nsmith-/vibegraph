---
type: Backlog Item
title: "Test scratch directories are hand-rolled instead of tempfile"
description: "About fifteen tests build temp_dir()+pid paths and never remove them; tempfile is already a CLI dev-dependency."
area: hygiene
state: open
priority: low
closes_when: "vibegraph-lib's tests use tempfile for scratch directories."
blocked_by: []
opened: 2026-10-10
tags: [tests, standard-primitive]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-c-report, resource: "../../sprints/hygiene/sessions/R-C-report.md", title: "R-C report (hygiene sprint)"}
  - {id: r-f-report, resource: "../../sprints/hygiene/sessions/R-F-report.md", title: "R-F report (hygiene sprint)"}
---
Sites in `ufo/mod.rs`, `artifact.rs`, `pdf/mod.rs`, `cache/*`, `tests/validate_samples.rs`, `tests/validate_sigma.rs` (R-C, R-F and R-G cross-cluster patterns). AGENTS.md "never hand-write a standard primitive".

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-C](../../sprints/hygiene/sessions/R-C-report.md), [R-F](../../sprints/hygiene/sessions/R-F-report.md).
