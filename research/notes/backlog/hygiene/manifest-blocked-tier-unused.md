---
type: Backlog Item
title: The manifest's blocked tier is a schema slot nothing uses
description: validation/manifest.toml documents a blocked tier with a blocker field, but no cell uses it; whether to keep or retire it is a schema decision.
area: hygiene
state: needs-user
priority: low
closes_when: The blocked tier is either retired from the manifest schema and collator, or kept with a stated use.
blocked_by: []
opened: 2026-08-06
tags: [manifest, schema, collator]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L525-L531", title: "TODO.md entry T040"}
---
The header of `validation/manifest.toml` (line 56) lists `blocked` ("not runnable
until `blocker` lands") as a tier next to `hermetic`/`banked`/`long`/`covered-by`/
`uncovered`. No cell in the manifest has `tier = "blocked"`. Blockers are
handled per test instead (for example the asserted reasons in
`vibegraph-lib/tests/validate_scales.rs`). Deciding between keeping the slot for
later use and retiring it, along with the collator code that reads it, is a call
about the schema.
