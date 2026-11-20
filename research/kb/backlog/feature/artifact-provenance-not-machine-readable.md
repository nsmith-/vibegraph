---
type: Backlog Item
title: "An integrate result does not say which build wrote its artifact, and artifact refusals carry no machine-readable reason"
description: "The integrated result records format_version but not enumeration_version or the vibegraph version; every artifact refusal is error_kind artifact, so callers parse messages to spot stale artifacts."
area: feature
state: open
priority: medium
closes_when: "The integrate and inspect result objects carry the artifact's enumeration_version and the vibegraph version that wrote it, and an artifact-kind refusal carries a stable reason (enumeration_version with found/expected, format_version, digest mismatch, model mismatch, collider energy), all as added fields under schema_version 1, documented in the ldjson schema and pinned by CLI tests including a version-2 artifact."
blocked_by: []
opened: 2026-11-20
tags: [cli, ldjson, artifact, refusal, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-20}
---
Reported by simshop when v0.2.2 raised `ENUMERATION_VERSION` from 2 to 3. The
user asked for it in v0.2.3.

simshop stores each sample's integrate `artifact_sha256`, and generate shards
added later read that artifact. Under v0.2.2 an artifact integrated by v0.2.1 is
refused, and it must be re-integrated from its cards, never mixed with new
shards. Today simshop cannot tell that case apart without parsing the message:
- the `refusal` object's `error_kind` is `artifact`, which also covers a
  different format version, a digest mismatch (corrupt or edited file), a model
  digest mismatch and a changed collider energy;
- the `integrated` result's `format_version` is 24 in both v0.2.1 and v0.2.2,
  and the result records neither `enumeration_version` nor the vibegraph
  version.

simshop's interim workaround is to record the vibegraph version it ran and use
`inspect --ldjson` as a backstop. The ldjson schema rule allows this item
without a version bump: adding a field keeps `schema_version`, while adding an
error kind would bump it, so the reason is a field on the existing `artifact`
refusal, not a new kind.

Related: [ldjson-error-kinds-too-coarse](ldjson-error-kinds-too-coarse.md)
(generate's catch-all errors under the wrong kind).
