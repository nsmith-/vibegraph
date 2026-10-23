---
type: Backlog Item
title: Floor-bound multichannel channels draw the same points at every budget
description: "Under ByAlpha, a channel with α·neval under its floor draws identical points at 600k and 1.2M; on bbx_to_ccx_emmm_qcd0 551–585 of 615 channels do, so one tail point set the ladder's spread."
area: performance
state: open
priority: high
closes_when: "A ladder on bbx_to_ccx_emmm_qcd0 with the fix (Neyman allocation first, then a floor that scales with neval), 20 seeds at 600k and 1.2M, shows the per-seed sd falling about 1/√2 with budget and χ²/dof near 1, or the record says why the fix fails."
blocked_by: []
opened: 2026-10-23
tags: [multichannel, allocation, vegas, two-to-six, heavy-tail, budget-ladder]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-23}
sources:
  - {id: b1, resource: "../../sprints/two-to-six-gate/sessions/B1-report.md", title: "two-to-six-gate B1 report, mechanism and Found 1, 3, 6"}
---
On the tree the M3 ran (`784582d`), single points in floor-bound channel 551
carried the `bbx` ladder's excess at both rungs. Setting that channel's term
to its median gives sd 3.98e-9 and χ²/dof 1.03 at 1.2M. On current main the
551 tail is gone, but 563–580 channels still share draws between rungs, and
the sd falls only ×0.93 from 600k to 1.2M.

- **Falsifier for the fix:** under Neyman, channel 551's variance across seeds
  must fall with its point count.
- **For any ladder until then:** state the fraction of the budget above the
  floors, and read rungs as a paired per-seed difference. The
  `√(s₁²/n₁ + s₂²/n₂)` line assumes independent rungs, which these are not.
- `integrate_with` fixes `ByAlpha`; the probe must expose the allocation
  first.

Filed at the close-out of [two-to-six-gate](../../sprints/two-to-six-gate/sprint.md).
