---
type: Backlog Item
title: "The SMEFT_alpha negative-order amplitude and σ oracles live only in scratch"
description: "K-O1 matched SMEFT_alpha per-diagram amplitudes and σ against MadGraph for the QED:-1 vev insertion, but the model is not vendored and the rows are not banked, so no gate holds the result."
area: validation
state: open
priority: low
closes_when: "SMEFT_alpha (or a smaller vendored model with a {NP:1, QED:-1} coupling) is vendored, with a standalone amplitude row and a seeded σ row in the manifest; the toy negative-order model gets its standalone table."
blocked_by: []
opened: 2026-11-17
tags: [validation, smeft, coupling-orders, banked]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Found by sprint K's K-O1 session
(its report, `sessions/K-O1-report.md` in the sprint folder).
The toy model's standalone table needs a `restrict_full.dat` card first.
