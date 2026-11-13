---
type: Backlog Item
title: "A ranking and a diagrams.json pair by diagram digest alone, which does not cover configuration grouping"
description: "diagram_set_digest covers the diagrams, not their configuration grouping, so a page can pair a ranking and a diagrams.json that group differently."
area: feature
state: open
priority: low
closes_when: "diagrams.json and the ranking both carry a digest of the configuration partition (or a grouping version), the documented pairing rule requires it to match, and inspect --ranking --diagrams refuses on it."
blocked_by: []
opened: 2026-11-10
tags: [ranking, enumerate, pairing, digest]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr126, resource: "https://github.com/nsmith-/vibegraph/pull/126", title: "inspect --ranking (found there)"}
  - {id: pr120, resource: "https://github.com/nsmith-/vibegraph/pull/120", title: "vibegraph enumerate and diagrams.json"}
---
A status page given a `diagrams.json` (nsmith-/vibegraph#120) and a ranking
(#126) pairs them by `diagram_set_digest`. That digest covers the diagram
sets, not the code that groups them into configurations, so an `enumerate`
from one build and an `integrate` from another can agree on the digest while
`s0.c3` names different diagrams. `inspect --ranking --diagrams` already
refuses that case by comparing memberships; a page reading two files cannot.
