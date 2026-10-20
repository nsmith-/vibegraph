---
type: Backlog Item
title: "event_norm = sum is refused"
description: "event_norm is IgnoredPhysics, so 20 of 65 LO CMS cards are refused though sum changes only the normalisation of the written event weights."
area: feature
state: open
priority: medium
closes_when: "event_norm = sum (and unity) are consumed: generate writes XWGTUP with MadGraph's normalisation for that setting, by a test against a MadGraph LHE header and weight, and σ is unchanged."
blocked_by: []
opened: 2026-10-20
tags: [runcard, lhef, event-weights, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
`event_norm` is `IgnoredPhysics(P_EVENT_NORM)`
(`vibegraph-lib/src/runcard/classes.rs:256`), refused off its `average`
default (`runcard.rs:653`). `sum` makes each event weight σ/N rather than σ;
the integration is the same. `lhef/record.rs:30` already notes what MadGraph
writes under `sum`.

In the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md)
20 of 65 LO cards set `event_norm = sum`; it was the first blocker on 14.
