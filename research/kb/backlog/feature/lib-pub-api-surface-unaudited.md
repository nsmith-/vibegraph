---
type: Backlog Item
title: vibegraph-lib's pub API surface has never been audited
description: vibegraph-lib exports much that only the CLI and validation crates consume; the supported library surface is undecided ahead of any compatibility promise.
area: feature
state: needs-user
priority: low
closes_when: The supported vibegraph-lib surface is decided and documented, and items only the CLI and validation crates use are demoted (pub(crate), a doc-hidden internal module, or moved), before any 1.0 compatibility promise.
blocked_by: []
opened: 2026-08-02
tags: [api, quality, pre-1.0]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L850-L854", title: "TODO.md entry T071"}
---
`vibegraph-lib/src/lib.rs` exposes 24 `pub mod`s. Much of that is consumed only by
`vibegraph-cli` and the validation crates. The user asked (2026-08-02) for a quality
pass before any backwards-compatibility promise:

1. audit what the library exports;
2. demote what only the CLI and validation crates use;
3. decide what the supported library surface actually is.

Step 3 is the user's call, so this item is `needs-user`. Until it is done, releases
stay on the 0.x line.
