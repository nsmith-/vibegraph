---
type: Backlog Item
title: "GJ-4Jets HT-40to100 is σ = 0 at LO by its cuts; MadGraph's gridpack σ is unrecorded"
description: "pta 200 forces jet HT ≥ 200 by pT balance against htjmax 100, so vibegraph refuses gridpack 244 (none passed the cuts); if the gridpack holds events, MadGraph reads the card differently."
area: validation
state: open
priority: medium
closes_when: "Gridpack 244's results.dat (σ, events) is read on the CERN host; if non-zero, the cut whose reading differs is named and the difference fixed or recorded."
blocked_by: []
opened: 2026-11-19
tags: [madgraph-parity, cuts, htj, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
`htj` sums every jet-class parton with no pT threshold (`cuts.f:874-913`),
and with maxjetflavor 5 every other outgoing parton is jet-class.

Found by K-G1 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-G1-report.md), Found 3).
