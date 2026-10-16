---
type: Backlog Item
title: "σ pulls lean high against the seeded MadEvent references"
description: "Across 47 rows the σ pulls against refdata-10's seeded references average +0.65 (spread 0.87, about 5 standard errors from zero), the direction of MadEvent's iteration-combination bias."
area: validation
state: open
priority: medium
closes_when: "The population mean of the σ pulls is re-read against each record's equal-weight mean, and the lean is either explained (reference-side bias, quantified) or traced to this side."
blocked_by: []
opened: 2026-10-16
tags: [sigma, madevent-reference, seeds, iteration-combination, refdata]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
The first banked run on refdata-10 (CI run 38923646457, `c7bb711`) passes
every σ gate: max |pull| is 2.45. Read as a population, though, the 47 pulls
average +0.65 with a spread of 0.87, about 5 standard errors from zero. Each
pull is one fixed-seed run on this side, so no single row says anything; the
population does.

The direction matches MadEvent's mean²/variance iteration combination, which
reads σ low (PR #22, concept `validation/madevent-iteration-weighting-bias.md` on its branch).
The gates read the reported mean wherever equal weights have not been
validated (D1, first amendment, in `sprints/sigma-reference-scatter`). Every
seed record already carries its equal-weight mean, so re-reading the pulls
against it needs no MadEvent run. If the lean survives that, the next oracle
is a few-row adaptation-free estimate (as #26 built for `ee_to_mumua`).
