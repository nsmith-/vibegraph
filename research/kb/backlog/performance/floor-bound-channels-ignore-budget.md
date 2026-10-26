---
type: Backlog Item
title: Floor-bound multichannel channels draw the same points at every budget
description: "On bbx_to_ccx_emmm_qcd0, 551–585 of 615 channels sit at their floor and draw identical points at 600k and 1.2M; Neyman, which moves only points above the floors, does not change that."
area: performance
state: blocked
priority: high
closes_when: "A ladder on bbx_to_ccx_emmm_qcd0 with a floor that scales with neval, read through an outcome-independent combine, 20 paired seeds at 600k and 1.2M, shows the per-seed sd falling about 1/√2 with budget and χ²/dof near 1, or the record says why the fix fails."
blocked_by: [point-weighted-combine-biased-by-earlier-iterations]
opened: 2026-10-23
tags: [multichannel, allocation, vegas, two-to-six, heavy-tail, budget-ladder]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: b1, resource: "../../sprints/two-to-six-gate/sessions/B1-report.md", title: "two-to-six-gate B1 report, mechanism and Found 1, 3, 6"}
  - {id: p1, resource: "../../sprints/floor-bound-neyman/sessions/P1-report.md", title: "floor-bound-neyman P1 report, Neyman measured and the scaling-floor proposal"}
---
Single points in floor-bound channels carried the `bbx` ladder's excess (B1).
Under ByAlpha the per-seed sd ratio 1.2M/600k is 1.24 (1.05 without each
rung's worst seed) over 20 seeds.

- **Neyman is ruled out as the fix** (P1). It re-splits only the points above
  the floors: both rules draw the same points per iteration, about 580–590
  channels stay floor-bound, and 551–575 still share draws between rungs. Its
  apparent sd ratio 0.65 is the combine's bias; as plain means it is 1.18.
- **Next:** the floor that scales with `neval`, as P1's report proposes. It
  waits on an outcome-independent combine, or it reads the same false gain
  wherever its counts vary between iterations.
- **For any ladder until then:** state the fraction of the budget above the
  floors (0.32 at 600k, 0.49 at 1.2M), and read rungs as paired per-seed
  differences. `probe_2to6_budget_ladder` now takes `VG_2TO6_ALLOC` and
  prints the paired difference.
- **Falsifier for the scaling floor:** channel 551's quoted variance must fall
  with its point count at every seed, not only on the seeds a tail point
  lifted it (Neyman: 6 of 20 at 1.2M).

Filed at the close-out of [two-to-six-gate](../../sprints/two-to-six-gate/sprint.md).
