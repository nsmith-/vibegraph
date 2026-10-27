---
type: Sprint
title: "ht-cuts"
description: "Compile MadGraph's HT cuts (htjmin, htjmax, ihtmin, ihtmax) instead of refusing them, matching cuts.f's definitions, with σ validated on an HT-binned card."
status: draft
active: false
tags: [cuts, ht, runcard, sprint-d]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
---

## Goal and exit criteria

`htjmin`, `htjmax`, `ihtmin` and `ihtmax` are compiled as cuts with
MadGraph's definitions: which partons count for each, and the inclusive
variant. An HT-binned card's σ matches MadGraph's within the σ-gate tolerance.
These cuts were the first blocker on 10 of the CMS survey's 65 LO cards.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):
- `ht-cuts-unimplemented`

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| F1 | feature-dev (Opus) | – | workspace tests; σ against a banked or requested MadGraph reference | – |
| HY1 | hygiene-dev | F1 | workspace tests | `ht-cuts-unimplemented` |
