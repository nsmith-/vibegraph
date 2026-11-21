---
type: Backlog Item
title: "Three tests copy a vendored UFO into a temp dir with the same loop"
description: "One test in config.rs and the two K-H1 added in ufo/mod.rs each copy a vendored UFO directory into a tempdir file by file before editing one file."
area: hygiene
state: open
priority: low
closes_when: "One test helper copies a vendored UFO and edits a named file, and all three tests use it."
blocked_by: []
opened: 2026-11-19
tags: [tests, ufo, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Found by K-H1 (hygiene pass on #141, #144 and #145, PR #157), Found 5.
