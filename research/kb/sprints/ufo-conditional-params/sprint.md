---
type: Sprint
title: "UFO parameters defined under 'if configuration' load or are refused"
description: "A parameters.py that defines a parameter only inside 'if configuration.…' branches loads with MadGraph's default branch or is refused naming the parameter; expr::eval no longer panics on a missing parameter."
status: draft
active: false
tags: [ufo, parser, panic, sprint-d]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
## Goal and exit criteria

Done when the item's `closes_when` holds, pinned by tests that fail without
the fix.

## Scope

Backlog items claimed (one `Backlog: <slug>` line in the draft PR):

- `ufo-parameters-conditional-branches-panic`

Part of sprint D, the CMS gridpack survey fixes (planner hand-off 2026-10-20).
The survey's dim6top_LO_UFO is on a CERN host and not in this repository;
tests use a synthetic toy UFO.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| F1 | feature-dev (Sonnet) | – | hermetic, fmt, clippy, kb-lint | the item |
| HY1 | hygiene-dev (Sonnet) | F1 | hermetic, fmt, clippy | – |
