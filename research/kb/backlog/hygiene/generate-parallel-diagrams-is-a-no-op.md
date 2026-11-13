---
type: Backlog Item
title: "generate accepts --parallel-diagrams and ignores it"
description: "generate no longer enumerates (it reads the artifact's diagram sets), but --parallel-diagrams still reaches it through the shared ParallelArgs and does nothing."
area: hygiene
state: open
priority: low
closes_when: "generate either does not accept --parallel-diagrams or refuses it with an explanation, and the CLI reference is regenerated."
blocked_by: []
opened: 2026-11-10
tags: [cli, generate, parallel]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr121, resource: "https://github.com/nsmith-/vibegraph/pull/121", title: "Self-contained integrate artifact (found there)"}
---
Since nsmith-/vibegraph#121, `generate` takes the diagram sets from the
artifact and enumerates nothing, so the enumeration pool option is inert.
It is still accepted because `GenerateArgs` flattens the shared
`ParallelArgs`. Split the option out of the shared group, or refuse it.
