---
type: Backlog Item
title: A flavour group whose subprocesses have different outgoing masses is refused
description: "When subprocesses sharing a matrix element differ in outgoing masses, derive_flavor_groups refuses rather than splitting them; 2 CMS cards stop there."
area: feature
state: open
priority: low
closes_when: "Subprocesses with different outgoing masses are put in separate groups (each with its own phase-space map) instead of refused, and one of the survey's two cards passes grouping with σ matching MadGraph."
blocked_by: []
opened: 2026-10-20
tags: [flavour-groups, masses, phase-space, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
`ProtonError::UnequalFinalMasses` (`vibegraph-lib/src/proton.rs:788`)
refuses a group whose members have different outgoing masses, since one
phase-space map cannot cover them. In the
[CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md) this is
2 LO cards (first blocker on 1).
