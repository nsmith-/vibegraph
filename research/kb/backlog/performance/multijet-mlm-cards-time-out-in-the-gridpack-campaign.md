---
type: Backlog Item
title: "37 CMS gridpacks, mostly 3-5-jet MLM cards, time out at 600 s"
description: "In main-202a540-200-pc, 37 of 199 gridpacks hit the 600 s limit; most are V/γ/tt̄ + 3-4 jets or QCD 5 jets. Where the time goes (compile, channels, sampling) is unmeasured."
area: performance
state: open
priority: high
closes_when: "Each of the 37 timeouts is attributed to a phase (enumeration, compilation, channel construction, sampling) from a timed run, recorded in the census, with the dominant causes filed as their own items."
blocked_by: []
opened: 2026-11-07
tags: [performance, mlm, multijet, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
---
Timeouts are the largest in-scope block after the refusals: 37 gridpacks at
600 s each, against 66 that complete. Examples: DYJets_HT-800to1200,
GJets-4Jets_dRGJ-0p2_HT-100to200, QCD-5Jets_HT-40, TT-3Jets_Bin-HT-800to1500,
WtoLNu-4Jets_MLNu-120to200, Zto2Nu-4Jets_HT-2500, VBS_WPMZTo4Q_5f_LO_EWK,
WWjj_SS_ewk, aQGC_WPJJWPJJjj_EWK_LO_NPle1. The full list is in the census.

This item is the triage; fixes get their own items. One card's account already
exists on the Sprint H branch (Zto2Q-4Jets: 674 680 diagrams, 11421 channels).
Census: [refusals at 202a540](../../validation/cms-gridpack-refusals-202a540.md).
