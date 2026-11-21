---
type: Backlog Item
title: "g2HDM charged-Higgs MLM cards: σ unmeasured after the panic fix"
description: "#153 fixed the panic on CMS cards 889–895, and the release candidate runs them without one, but no σ is measured: M300 costs about 4–8 h a seed on the CERN host."
area: validation
state: open
priority: medium
closes_when: "M300 (and at least one other mass) agrees with MadGraph's 10.04 ± 0.039 pb (M300) at ≥ 5 seeds by the multi-seed criterion, on the batch campaign or a larger machine."
blocked_by: []
opened: 2026-11-21
tags: [mlm, g2hdm, sigma, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-21}
sources:
  - {id: cern, resource: "../../sprints/v022-smeft-and-refusals/log.md", title: "Sprint K log, CERN targeted check on 400c88f9"}
  - {id: km1, resource: "../../sprints/v022-smeft-and-refusals/sessions/K-M1-report.md", title: "K-M1 report"}
---
On 400c88f9 all 7 cards pass setup. M300 ran 2 VEGAS iterations (2176
channels, 2.7M points each) at 2.2 GB RSS without a panic, at 0.5 ms a point.
This item moves to the v0.2.2 batch campaign (`v022-1009`).
