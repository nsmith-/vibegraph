---
type: Backlog Item
title: "The HT rows have no vibegraph-side budget ladder"
description: "probe_recarded_budget_ladder iterates RECARDED_ROWS only, so pp_to_jj_ht and pp_to_bb_iht were gated at 75000 x 10 without a measured rung-to-rung reading on this side."
area: validation
state: open
priority: low
closes_when: "A ladder probe over HT_ROWS at 75k/150k/300k/600k is run with the measured seed spread, and its reading is in the rows' integrals notes."
blocked_by: []
opened: 2026-10-26
tags: [sigma, budget-ladder, ht-cuts, probe]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: v1, resource: "../../sprints/ht-cuts/sessions/V1-report.md", title: "ht-cuts V1 report, Found 2"}
---
`probe_recarded_budget_ladder` (`vibegraph-lib/tests/validate_hadronic.rs`)
iterates `RECARDED_ROWS`; the two HT rows live in `HT_ROWS` and have no ladder.
The MadGraph side was laddered (20 + 20 seeds on `pp_to_bb_iht`, +0.40σ). This
side was not, though F1 measured the HT-only `p p > b b~` card's quoted error at
about half its seed spread at 4× (χ²/dof 3.76 over five seeds), and the gate's
five seeds read χ²/dof 1.66 with one seed 0.5 % low
([V1](../../sprints/ht-cuts/sessions/V1-report.md),
[F1](../../sprints/ht-cuts/sessions/F1-report.md)).

Per the seed policy, a rung-to-rung reading needs the measured seed spread (20+
seeds on the rungs that matter), not five.
