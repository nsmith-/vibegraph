---
type: Sprint
title: "Run-card hidden lines and the systematics warning"
description: "Run-card lines tagged '# hidden_parameter' are read as MadGraph reads them, and a card asking for systematics weights warns that none will be written."
status: draft
active: true
tags: [runcard, parser, systematics, cms, sprint-d]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
## Goal and exit criteria

Done when both items' `closes_when` hold, each pinned by a test that fails
without the fix.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `runcard-hidden-parameter-lines-dropped`
- `systematics-request-dropped-silently`

One PR: both are run-card reading (`runcard.rs`, `runcard/classes.rs`) and
read as one change. Part of sprint D, the CMS gridpack survey fixes (planner
hand-off 2026-10-20); siblings are `cut-refusal`, `ufo-conditional-params`,
`fnv-refusal`, `weighted-cap`, `default-particle-names`, `ckm-no-b-mass`.

The survey's data (cards, gridpacks) is on a CERN host and not in this
repository; tests use constructed cards.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| F1 | feature-dev (Sonnet) | – | hermetic, fmt, clippy, kb-lint | both items |
| HY1 | hygiene-dev (Sonnet) | F1 | hermetic, fmt, clippy | – |
