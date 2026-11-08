---
type: Backlog Item
title: "A panic in a rayon worker did not end the process"
description: "An instrumented run on the CERN host stayed alive about 40 minutes after a worker panic; a worker panic should end the run promptly with the panic's message and a non-zero exit."
area: feature
state: open
priority: medium
closes_when: "A panic injected into a parallel integration worker (a test-only hook) ends the CLI within seconds with a non-zero exit and the panic message, in text and ldjson modes, pinned by a CLI test; the 40-minute case is explained."
blocked_by: []
opened: 2026-11-08
tags: [panic, rayon, robustness, cli, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log, 2026-11-07: C diagnosed, Found"}
---
While diagnosing issue C (the idm `NoAdmissiblePair` panic), the CERN host's
instrumented run stayed alive for about 40 minutes after its panic. Why is
unexplained. Rayon re-raises a worker panic at the `join`/`scope` that spawned
it, so the likely causes are a long-running sibling task that the panic does
not cancel, or a spawn outside a scope (both inferred, neither checked).

Under gpdb's 600 s limit, such a panic reads as a timeout, not a panic, which
misfiles the card. A worker panic should end the run promptly.
[ldjson-stream-gaps](ldjson-stream-gaps.md) covers the separate gap that a
panic leaves the ldjson stream without a final object.
