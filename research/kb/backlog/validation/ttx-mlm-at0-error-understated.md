---
type: Backlog Item
title: "pp_to_ttx_0j1j_mlm @0's quoted error understates its seed scatter about 1.45×"
description: "The t t~ MLM row's @0 scatters at seed χ²/dof 2.09 (10 seeds) and 3.43 (5), as at refdata-9; the gate's band tops out at 3.097, so a sampler change would trip the band before the pull."
area: validation
state: open
priority: low
closes_when: "@0's seed scatter is read over ≥20 seeds and its channels' iteration errors are compared with their scatter, and the understatement is attributed or shown to be chance."
blocked_by: []
opened: 2026-10-17
tags: [mlm, ttx, error-calibration, seeds]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v1, resource: "../../sprints/refdata10-cell-flips/sessions/V1-report.md", title: "refdata10-cell-flips V1 report, Found 3–5"}
---
Over ten gate seeds `@0` reads 513.219 ± 0.179 pb at χ²/dof 2.09, and the first
five alone read 3.43. refdata-9 read 1.98 and 2.01. The quoted error is
consistently low by about √2.1 ≈ 1.45×. The gate's χ²/dof band tops out at
3.097, so `@0` has about 1.5× headroom on its scatter, against much more on
its pull ([V1](../../sprints/refdata10-cell-flips/sessions/V1-report.md)).

Related, from the same report:
- The reference's 1.13 pb error on `@1` dominates the cell. Fifteen more
  MadEvent seeds would tighten it. That is optional, and a banking request if
  wanted.
- The row's per-event dump numbers (the samples note, 2026-10-03) predate the
  CM-frame clustering fix `6740954`. `pixi run validate-mlm-dumps` on the bank
  host would re-date them.
