---
type: Backlog Item
title: "vibegraph-lib tests hand-roll temporary directories"
description: "Ten vibegraph-lib test sites build scratch directories from temp_dir() plus the pid and remove them by hand, leaking them when a test fails."
area: hygiene
state: open
priority: low
closes_when: "vibegraph-lib's tests take scratch directories from the tempfile crate (a dev-dependency), with no hand-built temp paths left."
blocked_by: []
opened: 2026-10-16
tags: [hygiene, testing, dependencies]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
Sites: `src/cache/resolve.rs:134`, `cache/pinned.rs:223`, `cache/store.rs:195`,
`ufo/mod.rs:808`, `:845`, `:871`, `artifact.rs:885`, `:926`, `:1072`, `:1113`.
`vibegraph-cli` and `validation-report` already use `tempfile`. Found in the
[HY1 report](../../sprints/sigma-reference-scatter/sessions/HY1-report.md).
