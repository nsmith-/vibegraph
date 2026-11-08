---
type: Backlog Item
title: "A coloured-scalar seagull vertex with colour T(1,-1,3)*T(2,4,-1) cannot be compiled"
description: "The squark-squark-gluon-gluon contact carries a product of two generators, which the colour compiler refuses as not a 3/3̄ slot pair, so stop pair production cannot be evaluated."
area: feature
state: open
priority: high
closes_when: "A vertex with colour T(a,-1,i)*T(b,j,-1) compiles, a standalone JAMP row (e.g. g g > t1 t1~ in the MSSM) matches MadGraph, and SMS-StopStop_mStop-600 integrates past compilation."
blocked_by: []
opened: 2026-11-07
tags: [colour, ufo, bsm, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
---
`amplitude compilation failed: unsupported color structure 'T(-1,3) in
'T(1,-1,3)*T(2,4,-1)' is not a 3/3̄ slot pair'`. This is the colour factor of
the two-gluon contact of any colour-triplet scalar, (T^a T^b)_{ij}, so every
squark or leptoquark pair-production model that writes it this way stops here.

Refused in the `main-202a540-200-pc` campaign (3 gridpacks):
SMS-StopStop_mStop-600, SMS-StopStop_mStop-850, HSCP-Stop_Par-M-1800_xqcut150.

Coloured-scalar pair production is a staple BSM search channel, which is why
this is ranked above its gridpack count. Census: [refusals at 202a540](../../validation/cms-gridpack-refusals-202a540.md).
