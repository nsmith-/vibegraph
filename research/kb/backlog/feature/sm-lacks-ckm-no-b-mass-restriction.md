---
type: Backlog Item
title: The built-in SM lacks the ckm_no_b_mass restriction
description: "26 of 65 LO CMS cards import sm-ckm_no_b_mass, which the built-in SM does not ship; it is the most common first blocker in the survey."
area: feature
state: open
priority: high
closes_when: "'import model sm-ckm_no_b_mass' loads the built-in SM with MadGraph's restrict_ckm_no_b_mass.dat (restrict_ckm with MB = ymb = 0), and a coupling or census test pins it against MadGraph at that restriction."
blocked_by: []
opened: 2026-10-20
tags: [model, restriction, ckm, sm, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
`SMRestrict::restrict_card_text` (`vibegraph-lib/src/ufo/sm.rs:78-91`) ships nine restrict
cards under `ufo/sm_assets/`; `ckm_no_b_mass` is not one. MadGraph's card is
`restrict_ckm.dat` with the b mass and Yukawa (`MB`, `ymb`) set to zero.

In the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md)
26 of 65 LO cards import `sm-ckm_no_b_mass` (32 counting `loop_sm` and
topBSM variants), and reached later blockers only through the gridpack's own
model copy. It is the commonest blocker before any other.
