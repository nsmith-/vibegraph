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
Sites (`std::env::temp_dir()` in `vibegraph-lib/src` tests, 2026-10-26):
`cache/resolve.rs:125`, `cache/pinned.rs:233`, `cache/store.rs:194`,
`ufo/mod.rs:1134`, `:1171`, `:1197`, `artifact.rs:501`, `:550`, `:591`, `:649`,
`:673`, `pdf/mod.rs:495`, `:538`.
`vibegraph-cli` and `validation-report` already use `tempfile`. Found in the
[HY1 report](../../sprints/sigma-reference-scatter/sessions/HY1-report.md).
