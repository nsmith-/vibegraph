---
type: Sprint
title: "MadGraph's default particle names, and --modelname"
description: "A model's particles answer to MadGraph's PDG-keyed default names (mu+ on loop_sm, u~ and mu- on BSM UFOs), and 'import model X --modelname' is accepted and switches only that renaming off."
status: draft
active: true
tags: [process-grammar, model, particle-names, madgraph-parity, cms, sprint-d]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
## Goal and exit criteria

Done when the item's `closes_when` holds, pinned by tests.

## Scope

Backlog items claimed (one `Backlog: <slug>` line in the draft PR):

- `madgraph-default-particle-names-missing`

Part of sprint D, the CMS gridpack survey fixes (planner hand-off 2026-10-20).
If the renaming reaches further than name resolution (names written into
outputs, diagram labels, LHEF, artifacts), the session stops and reports, and
the manager re-scopes with the planner rather than growing the PR.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| F1 | feature-dev (Opus) | – | hermetic, fmt, clippy, kb-lint | the item |
| HY1 | hygiene-dev (Sonnet) | F1 | hermetic, fmt, clippy | – |
