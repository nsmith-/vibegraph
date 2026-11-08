---
type: Backlog Item
title: "validate.sh rebuilds target/release-debug/vibegraph, changing the binary under a concurrent σ sweep"
description: "The banked layer rebuilds the release-debug binary with the extended-validation features, so a σ sweep using that binary in the same target directory switches builds mid-sweep."
area: hygiene
state: open
priority: low
closes_when: "The extended-validation skill says to run validate.sh in its own CARGO_TARGET_DIR when anything else uses target/release-debug, or validate.sh does so itself."
blocked_by: []
opened: 2026-11-07
tags: [validation, tooling, skills, build]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hm2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-M2-report.md", title: "H-M2 report, Found 6"}
---
Found by H-M2. `validation/validate.sh` rebuilds
`target/release-debug/vibegraph` with the extended-validation features. A σ sweep
that runs that binary while the banked layer runs in the same target directory
changes build mid-sweep. H-M2 avoided it with
`CARGO_TARGET_DIR=target/h-m2-validate`. The rule belongs in the
`extended-validation` skill (`.agents/skills/extended-validation/`).
