---
type: Backlog Item
title: "gg_to_gg_cg's integrals cell is still info after its re-bank"
description: "The row's single high MadEvent draw is replaced by refdata-10's seeded reference, against which this side reads pull +0.15, but the cell is still info."
area: validation
state: open
priority: low
closes_when: "gg_to_gg_cg's integrals cell gates against its refdata-10 seeded reference, with its manifest note rewritten on that reference."
blocked_by: []
opened: 2026-10-16
tags: [sigma, smeftsim, madevent-reference]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
`sprints/sigma-gg-cg` attributed the −0.22 % offset to the bank being one
high MadEvent draw and kept the cell `info` until a seeded re-bank. refdata-10
carries that re-bank: five fresh-directory seeds at `hard_survey = 2`, reported
2.70658e5 ± 88 pb, against which the first banked run on it reads pull +0.15
([D1](../../sprints/sigma-reference-scatter/decisions/D1-reference-error-design.md)).
What is left is the flip to `gate` and the note.
