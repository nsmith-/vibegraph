---
type: Backlog Item
title: "ModelChanged tells a user of the interned SM to restore files"
description: "GenerateError::ModelChanged says the model's files changed after integrate ran and to restore them, also at location (built in), where a digest mismatch means a different build."
area: hygiene
state: open
priority: low
closes_when: "An interned-model digest mismatch gets its own wording (this build's built-in model differs from the integrating build's; re-integrate), with a test, and integrate-generate-split.md §3 says so."
blocked_by: []
opened: 2026-10-26
tags: [generate, model-identity, messages]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: hy1, resource: "../../sprints/gridpack-generate/sessions/HY1-report.md", title: "gridpack-generate HY1 report, Found 1"}
---
`GenerateError::ModelChanged` (`vibegraph-lib/src/generate.rs`, the `#[error]`
near :193) reads "its files changed after vibegraph integrate ran; re-integrate,
or restore them". For the interned SM the location is `(built in)` and there are
no files: a mismatch means the build that generates differs from the one that
integrated. The text is the design's
([integrate and generate split](../../pipeline/integrate-generate-split.md) §3)
verbatim, so the concept needs the same correction
([HY1](../../sprints/gridpack-generate/sessions/HY1-report.md)).
