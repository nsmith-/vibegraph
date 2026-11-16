---
type: Backlog Item
title: "A UFO vertex with a list of couplings for one colour-Lorentz pair is refused"
description: "Vertices such as EWdim6NLO's V_4 give {(0,0): [GC_47, GC_268]}; MadGraph sums the list. Refused in 4 CMS-store models."
area: feature
state: open
priority: medium
closes_when: "A vertex coupling entry that is a list of couplings is loaded as their sum, carrying each coupling's orders (refusing a list whose members' orders differ, unless MadGraph's handling of that case is reproduced), with a test, and EWdim6NLO loads."
blocked_by: []
opened: 2026-11-16
tags: [ufo, vertices, refusal, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-16}
---
Reported by simshop's scan of the CMS model store: `EWdim6NLO.tar.gz` has
`V_4 couplings = {(0,0):[ C.GC_47, C.GC_268 ], ...}`. EffDM_UFO, NMSSMHET and
NMSSMHET-NoY are refused for the same form.

Read how MadGraph treats a list whose couplings carry different orders before
summing them. That decides whether the sum is one coupling or one term per
member.
