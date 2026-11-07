---
type: Backlog Item
title: Crossing pairs that share a matrix element but not a colour basis are refused
description: "Conjugate pairs now share a group (#100); only the event colour-label check on a real HT-binned γ/QCD MLM survey card remains, since H-G1 checked a non-matched stand-in."
area: feature
state: open
priority: low
closes_when: "A flavour group may hold subprocesses whose colour bases differ by a conjugation or crossing, with each event's colour flow drawn in its own subprocess's basis, and an HT-binned γ/QCD MLM card from the survey passes grouping with the event colour labels checked per subprocess."
blocked_by: []
opened: 2026-10-20
tags: [flavour-groups, colour, crossing, mlm, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
  - {id: hg1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-G1-report.md", title: "H-G1: the colour pairing under π and its per-subprocess label oracle"}
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log: the closing campaign"}
---
Since #100, `derive_flavor_groups` pairs two subprocesses' colour bases under
the fingerprint permutation π and keeps conjugate pairs such as
`u u > g g u u` with `u~ u~ > g g u~ u~` in one group. Each event's flow is
written from the member's own table under π. Pairs that cannot be paired are
grouped apart rather than refused.

What is left is the last clause of `closes_when`. [H-G1](../../sprints/cms-gridpack-coverage/sessions/H-G1-report.md)
checked the event colour labels per subprocess against MadEvent's own events,
but on a local stand-in: HT-binned (`ihtmin = 300`), with neither MLM matching
nor the photon. In the closing campaign `main-202a540-200-pc`, 10 of the 19
colour-split survey cards integrate, at 0.967–1.098 of MadGraph σ, but no event
colour labels were compared.

To close: run one of the survey's HT-binned γ/QCD MLM cards and compare its
event colour labels per subprocess with MadEvent's, by the method of
H-G1's section (b). Its script was scratch, not committed.
