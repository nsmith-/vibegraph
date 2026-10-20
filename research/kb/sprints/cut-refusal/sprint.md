---
type: Sprint
title: "Unimplemented cuts: refused early, and no-op values let through"
description: "An active unimplemented cut is refused before diagram enumeration with a message naming it, and one set where it cannot bind is treated as its default."
status: draft
active: true
tags: [cuts, runcard, refusal, cms, sprint-d]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
## Goal and exit criteria

Done when both items' `closes_when` hold, each pinned by tests that fail
without the fix.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `unimplemented-cut-refused-after-enumeration`
- `no-op-cut-values-refused-as-active`

One PR: both change `detect_unimplemented` (`vibegraph-lib/src/cuts.rs`) and
read as one change. Part of sprint D, the CMS gridpack survey fixes (planner
hand-off 2026-10-20); siblings are `runcard-hidden-lines`,
`ufo-conditional-params`, `fnv-refusal`, `weighted-cap`,
`default-particle-names`, `ckm-no-b-mass`.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| F1 | feature-dev (Sonnet) | – | hermetic, fmt, clippy, kb-lint | both items |
| HY1 | hygiene-dev (Sonnet) | F1 | hermetic, fmt, clippy | – |
