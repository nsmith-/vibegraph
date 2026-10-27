---
type: Sprint
title: "flavour-mixed-masses"
description: "Split a flavour group whose subprocesses have different outgoing masses into separate groups, rather than refusing it, with no change to the phase-space map code."
status: draft
active: false
tags: [flavour-groups, masses, sprint-d]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
---

## Goal and exit criteria

`derive_flavor_groups` puts subprocesses with different outgoing masses into
separate groups, each with its own phase-space map, instead of refusing with
`ProtonError::UnequalFinalMasses`. One of the CMS survey's two cards then
passes grouping, and its σ matches MadGraph's.

**Scope boundary (user, 2026-10-22).** This sprint changes grouping only. The
phase-space map code (`vegas.rs`, `budget.rs`, the channel maps) belongs to
another design session, and nothing in it is edited here. Any change the maps
would need is written up in `map-changes-for-the-maps-session.md` in this
folder, for that session to make.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):
- `flavour-group-mixed-outgoing-masses-refused`

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| F1 | feature-dev (Opus) | – | workspace tests and a σ check against MadGraph (banking request if no reference exists) | – |
| HY1 | hygiene-dev | F1 | workspace tests | `flavour-group-mixed-outgoing-masses-refused` |
