---
type: Sprint
title: "Majorana and fermion-number-violating models are refused, not panicked on"
description: "A process with a Majorana fermion or a fermion-number-violating vertex is refused before evaluation with an error naming the fermion flow, and the evaluator's panic sites are unreachable from such models."
status: draft
active: false
tags: [majorana, fermion-flow, refusal, panic, sprint-d]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
## Goal and exit criteria

Done when the item's `closes_when` holds, pinned by tests that fail without
the fix.

## Scope

Backlog items claimed (one `Backlog: <slug>` line in the draft PR):

- `fermion-number-violation-panics-instead-of-refusing`

Part of sprint D, the CMS gridpack survey fixes (planner hand-off 2026-10-20).
The survey's BSM models are on a CERN host and not in this repository; tests
use synthetic toy UFOs. Majorana *support* stays descoped
(`majorana-fermions-unsupported`); this PR only refuses.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| F1 | feature-dev (Opus) | – | hermetic, fmt, clippy, kb-lint | the item |
| HY1 | hygiene-dev | F1 | hermetic, fmt, clippy | – |
