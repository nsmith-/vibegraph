---
type: Backlog Item
title: "178 campaign cards, mostly multijet MLM and VBS, time out at every seed"
description: "In v022-1009, 178 cards hit the 20-minute espresso cap on all 5 seeds and were never retried, mostly V/γ/tt̄ + 3-4 jets, VBS and VBF resonances. Which phase takes the time is unmeasured."
area: performance
state: open
priority: high
closes_when: "For v022-1009's 178 never-retried cards: (a) a phase-timed sample (the integrate --output-ldjson phase records) covers every family, at least 2 cards per family and one from each group of 'other'; (b) the dominant phase is named per family from the streamed records, a card killed at the cap counting by its last started phase; (c) the cap-limited retries are recorded as explained by the cap (117 of 155 ok at microcentury, 36 of the remaining 38 at workday); (d) the dominant causes are filed as their own items."
blocked_by: []
opened: 2026-11-07
tags: [performance, mlm, multijet, vbs, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
In `vg-validator`'s batch campaign v022-1009 (v0.2.2, 5 seeds, time caps
espresso < microcentury < workday), 178 cards timed out at espresso (20 min
wall, 4 CPUs) on all 5 seeds and were never retried: W/Z/DY + 4 jets 56,
VBS 47, VBF resonances 22, GJ-4Jets 19, TT-3Jets 10, aQGC 7, other 17 (QCD
multijet MLM; older GJets HT/DR; TT FCNC tZq; VBS-like madspin/QCD). The
list, by family:
[v022-1009-never-retried.tsv](../../sprints/v023-orders-majorana-timeouts/v022-1009-never-retried.tsv).

The cap explains most of the retried timeouts: of 155 seed runs that hit it
on cards with an ok seed, 117 were ok at microcentury and 36 of the remaining
38 at workday. Those are cost, not hangs; the never-retried 178 are the open
question.

`integrate --output-ldjson` streams a start and an end `phase` record for
each phase (input, enumerate, amplitude_probe, pdf, compile, channels,
alpha_survey, vegas, configurations, bank) with its wall time and sizes, so a
run killed at the cap names the phase it was in. The triage is a timed run
of a sample of the 178 on `vg-validator`, read here; fixes get their own
items. One older account: Zto2Q-4Jets at 202a540 had 674 680 diagrams and
11421 channels. The earlier census of 37 timeouts at 600 s:
[refusals at 202a540](../../validation/cms-gridpack-refusals-202a540.md).
