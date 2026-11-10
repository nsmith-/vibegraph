---
type: Backlog Item
title: "Passing a grid.bin.zst to integrate gives a diagram-artifact decode error"
description: "An integrate artifact shares the diagram artifact's zstd magic, so integrate refuses a grid.bin.zst with a diagram-artifact format-version message instead of naming it."
area: hygiene
state: open
priority: low
closes_when: "integrate given an integrate artifact refuses it naming what it is (and pointing to generate or inspect), pinned by a CLI test."
blocked_by: []
opened: 2026-11-10
tags: [cli, integrate, artifact, errors]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr121, resource: "https://github.com/nsmith-/vibegraph/pull/121", title: "Self-contained integrate artifact (found there)"}
---
`integrate`'s input sniffing (nsmith-/vibegraph#121) reads any file starting
with the zstd magic as a diagram artifact. `grid.bin.zst` starts with it too,
so the user who passes the wrong file is told about diagram artifact format
versions. Both artifacts decode their version header first, so the reader
can tell them apart and say which one it got.
