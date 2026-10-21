---
type: Sprint
title: "event_norm = sum and unity are consumed"
description: "A run card's event_norm = sum or unity is honoured: generate writes XWGTUP and IDWTUP with MadGraph's normalisation for that setting, and σ is unchanged."
status: draft
active: true
tags: [lhef, runcard, event-weights, cms, sprint-d]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
## Goal and exit criteria

Done when the item's `closes_when` holds. Its "test against a MadGraph LHE
header and weight" needs a banked MadGraph sample written under `sum` unless
one already exists; if none does, this PR carries a code-read pin and a
banking request, and whether the item closes before the banking run is the
user's call.

## Scope

Backlog items claimed (one `Backlog: <slug>` line in the draft PR):

- `event-norm-sum-refused`

Part of sprint D, the CMS gridpack survey fixes (planner hand-off 2026-10-20,
amended the same day): 20 of 65 LO CMS cards set `event_norm = sum`.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| F1 | feature-dev (Sonnet) | – | hermetic, fmt, clippy, kb-lint | the item (subject to banking) |
| HY1 | hygiene-dev (Sonnet) | F1 | hermetic, fmt, clippy | – |
