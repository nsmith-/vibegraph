---
type: Sprint
title: "The automatic WEIGHTED search reaches MadGraph's cap"
description: "With no explicit orders, the lowest-WEIGHTED search runs up to (n_ext−2) × the largest WEIGHTED of any model interaction, as MadGraph's does, so a QED+NP vertex's diagrams are found."
status: draft
active: true
tags: [diagrams, coupling-orders, weighted, madgraph-parity, sprint-d]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
## Goal and exit criteria

Done when the item's `closes_when` holds. Its MadGraph census on a model
whose lowest order lies above the old cap needs a MadGraph run (M3 only):
this PR carries a hermetic hand-checked count and a banking request, and
whether the item closes before the banking run is the user's call.

## Scope

Backlog items claimed (one `Backlog: <slug>` line in the draft PR):

- `auto-weighted-search-cap-uses-hierarchy`

Not in scope: `weighted-order-computed-three-ways` (the three WEIGHTED
computations and their missing-order rules); if the cap's computation would
add a fourth, say so instead of absorbing that item.

Part of sprint D, the CMS gridpack survey fixes (planner hand-off 2026-10-20).
The survey's models (cards 060, 067, 050, 071) are on a CERN host; tests use a
synthetic toy UFO.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| F1 | feature-dev (Opus) | – | hermetic incl. diagram oracle, fmt, clippy, kb-lint | the item (subject to banking) |
| V1 | validation-dev | F1 | diagram census unmoved; banking request | – |
| HY1 | hygiene-dev (Sonnet) | V1 | hermetic, fmt, clippy | – |
