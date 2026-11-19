---
type: Backlog Item
title: "misset = -1 adds a 1 GeV² term to MadGraph's lepton ŝ floor that vibegraph does not apply"
description: "setcuts.f adds misset² to the lepton ŝ floor; vibegraph reads misset ≤ 0 as off, so floors differ on lepton cards with no other lepton cut ≥ 1 GeV. Seven campaign cards set misset = -1."
area: feature
state: open
priority: high
closes_when: "Either the floor provably cannot bind on any card (proof recorded at the NONPOSITIVE_IS_OFF comment in cuts.rs), or vibegraph applies the misset² term (pinned against MadGraph's own smin on a card where it is the maximum) or refuses misset < 0 by name where it would be the maximum; banked cards stay bit for bit."
blocked_by: []
opened: 2026-11-19
tags: [cuts, run-card, misset, smin, madgraph-parity, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
`setcuts.f:675` (pinned template) adds, for a subprocess with a charged
lepton, `max(smin_p**2, smin_m, mmnl**2, ptllmin**2, misset**2)` to `smin`.
#152 compiles `misset <= 0` to the default cuts and names this as a residual.
The manager's rule: a silent wrong answer is worse than a named refusal, so
the residual is either proved inert, implemented or refused.

Found by K-C1 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-C1-report.md), "What each test cannot detect").
