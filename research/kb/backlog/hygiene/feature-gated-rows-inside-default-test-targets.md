---
type: Backlog Item
title: "Two CLI test files gate rows with #[cfg(feature)] instead of a required-features target"
description: "cli_config_share and cli_onshell_veto gate extended-validation rows with #[cfg(feature)] inside default targets; the extended-validation skill asks for required-features targets."
area: hygiene
state: open
priority: low
closes_when: "Both files follow one convention, the one the extended-validation skill documents (or the skill is updated to allow the in-file form)."
blocked_by: []
opened: 2026-11-10
tags: [tests, extended-validation, cargo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr124, resource: "https://github.com/nsmith-/vibegraph/pull/124", title: "Configuration shares (found there)"}
---
`cli_config_share.rs` (nsmith-/vibegraph#124) copied `cli_onshell_veto.rs`:
its proton row is `#[cfg(feature = "extended-validation")]` inside a test
target that builds by default. The `extended-validation` skill
(`.agents/skills/extended-validation/SKILL.md`) says such tests use
`required-features`. Align both, or change the skill.
