---
type: Backlog Item
title: "Run the full CMS gridpack campaign at the v0.2.0 revision"
description: "The full 199-gridpack campaign was skipped before the v0.2.0 tag (only #113's refusal census ran); run it at the cut revision alongside the performance benchmarking."
area: validation
state: open
priority: high
closes_when: "A full campaign at the v0.2.0 tag's commit is recorded (ok/refused/timeout/panic and |pull| > 5), with no unexplained ok → not-ok against main-202a540-200-pc and no panics, and the performance benchmarks run on the same revision."
blocked_by: []
opened: 2026-11-09
tags: [cms, gridpacks, campaign, release, performance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-09}
sources:
  - {id: iz2, resource: "../../sprints/v02-last-refusals/sessions/I-Z2-report.md", title: "I-Z2 report"}
---
The user's decision (2026-11-08): the refusal-only census of #113 was the
real-card check for v0.2.0, and the full campaign runs after the cut, at that
revision, alongside the performance benchmarking.

In the census follow-up, 11 cards timed out at a 300 s limit, and
QCD-5Jets_Bin-HT-40 hit a 4.5 GB memory cap. The campaign needs the memory caps
the host's census relaunch used, because a global OOM once took down the whole
session (systemd `OOMPolicy=stop`).

Compare against `main-202a540-200-pc` (66 ok, 3 panics). The bar is no
unexplained ok → not-ok and no panics, not a count of 66. Expected drops:
- the three D1-family cards #113's census found (DYto2L-2Jets-EWK MLL-50,
  VBFto2L MLL-50, VBFto2L MLL-4to50);
- the four GJ/VBFtoG photon cards #109 refuses.

The census left 18 cards undecided. Sprint I added
refusals (the uncut-limit guards, untabulated partons, the zero τ floor) and
changed the absent-run-card defaults, so every changed outcome needs a reason.
