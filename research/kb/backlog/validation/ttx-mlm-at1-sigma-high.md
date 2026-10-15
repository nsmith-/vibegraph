---
type: Backlog Item
title: pp_to_ttx_0j1j_mlm's integrals cell waits on a hard_survey = 2 reference
description: The @1 excess is attributed and this side's part fixed (CM-frame clustering); the cell can gate once the reference is re-banked with hard_survey = 2, against which @1 already reads 0.88σ.
area: validation
state: blocked
priority: high
closes_when: The row's MadEvent reference is re-banked with hard_survey = 2 from fresh directories (in the seeded batch on PR #25), the integrals cell gates on a five-seed agreement with |pull| < 3, and the row's manifest note is rewritten on the fixed code.
blocked_by: [reference-sigma-seed-scatter-untested]
opened: 2026-10-03
tags: [mlm, sigma, ttx, madevent-reference, survey]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: sprint, resource: "../../sprints/sigma-ttx-mlm-at1/closeout.md", title: "sigma-ttx-mlm-at1 sprint record"}
---
The +1.28 % on `@1` had two causes
([sigma-ttx-mlm-at1](../../sprints/sigma-ttx-mlm-at1/closeout.md)):

- **This side, fixed:** the hadronic integrand clustered in the lab frame,
  where MadEvent clusters in the partonic CM. The fix moved `@1` by −4.465 ±
  0.083 pb.
- **MadEvent's default survey:** it under-integrates `g q > t t~ q`
  ([the measurement](../../validation/madevent-default-survey-bias.md)). With
  `hard_survey = 2`, 20 fresh seeds read `@1` 578.239 ± 0.520 against this
  side's 578.780 ± 0.327, a pull of 0.88.

What is left: re-bank the row with `hard_survey = 2`, then flip its
integrals cell from `info` to `gate`. The current bank reads a pull of 3.59.
The row's manifest note still quotes pre-fix numbers.
