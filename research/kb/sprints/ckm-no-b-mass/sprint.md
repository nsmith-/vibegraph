---
type: Sprint
title: "The built-in SM gains the ckm_no_b_mass restriction"
description: "'import model sm-ckm_no_b_mass' loads the built-in SM with the CMS-supplied restrict card, vendored with its provenance, and couplings are pinned at that restriction."
status: draft
active: true
tags: [model, restriction, ckm, sm, cms, sprint-d]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
## Goal and exit criteria

Done when the item's `closes_when` holds, as corrected below, pinned by tests.

## Scope

Backlog items claimed (one `Backlog: <slug>` line in the draft PR):

- `sm-lacks-ckm-no-b-mass-restriction`

Part of sprint D, the CMS gridpack survey fixes (planner hand-off 2026-10-20).

**Correction to the item.** MadGraph 3.7.1 does not ship
`restrict_ckm_no_b_mass.dat` (absent from `models/sm` and `models/loop_sm` at
the pinned submodule commit). CMS genproductions bundles it at
`models/sm/restrict_ckm_no_b_mass.dat`; all 88 surveyed gridpacks (MadGraph
2.4.2 to 2.9.18) carry an identical copy, sha256
`ad6e3eaf03443bc14148d392fe98deb712de29166061a7408eff521a47591b05`
(representative: gridpack 041_WWjj_SS_ewk). Against MadGraph's
`restrict_ckm.dat` it differs in three values, not two: `MB = 0`, `ymb = 0`
and `DECAY 15 = 2.27e-12` (MadGraph's is 0), so the tau width `WTA` survives
restriction. The loop_sm variant differs again and is out of scope.

A MadGraph coupling reference at this restriction needs a MadGraph run with
the CMS card dropped into `models/sm`: that is a banking request on this PR
(M3 only).

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| F1 | feature-dev (Opus) | – | hermetic incl. coupling oracle, fmt, clippy, kb-lint | the item (subject to the banking request) |
| V1 | validation-dev | F1 | coupling oracle; banking request written | – |
| HY1 | hygiene-dev (Sonnet) | V1 | hermetic, fmt, clippy | – |
