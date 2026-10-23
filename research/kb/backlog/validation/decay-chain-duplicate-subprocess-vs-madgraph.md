---
type: Backlog Item
title: "The duplicate-subprocess refusal across decay assignments may be stricter than MadGraph"
description: "Three CMS decay-chain cards (e.g. p p > w+ j j, w+ > j j) are refused as producing one subprocess twice; whether MadGraph counts them once, twice or rejects them is unchecked."
area: validation
state: open
priority: medium
closes_when: "MadGraph's handling of cards 024, 029 and 068 is measured on the pinned checkout, and vibegraph either matches it (census row) or keeps the refusal with the MadGraph behaviour it departs from recorded."
blocked_by: []
opened: 2026-10-20
tags: [decay-chains, process-grammar, madgraph-parity, triage, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
`DiagramError::DuplicateSubprocess` refuses a decay chain whose decay
assignments give the same (initial, sorted final) subprocess twice
(`vibegraph-lib/src/diagrams/chain.rs:204`), so it is not added to σ twice.
In the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md)
cards 024 (`p p > w+ j j, w+ > j j`), 029 and 068 hit it.

MadGraph keeps one matrix element per distinct process after
`combine_decay_chain_processes` ([decay chains](../../process/decay-chains.md));
it may merge these rather than double-count, in which case the refusal is
stricter than needed. **Inferred** from source reading; measure it with
MadGraph's generation on these cards, as the decay-chain census does.
