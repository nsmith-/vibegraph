---
type: Backlog Item
title: χ²/dof against each sample's own σ misreads heavy-tailed runs, per iteration and per seed
description: "Wide multichannel runs report χ²/dof of 10³–10⁵ while seeds scatter at 0.1–0.7; the ladder's seed χ²/dof read 2.97 from quiet seeds against a tail-raised mean. Both use each sample's own σ."
area: validation
state: open
priority: medium
closes_when: "Run and seed summaries report a robust companion beside χ²/dof (pooled emp/quoted, the median |pull|, χ² without the worst seed, and the RMS quoted error against the seed sd), and every consumer that gates on χ²/dof is checked against it."
blocked_by: []
opened: 2026-10-23
tags: [statistics, chi2, heavy-tail, two-to-six, seed-sweep]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-23}
sources:
  - {id: p1, resource: "../../sprints/two-to-six-gate/sessions/P1-report.md", title: "two-to-six-gate P1 report, Found 1"}
  - {id: b1, resource: "../../sprints/two-to-six-gate/sessions/B1-report.md", title: "two-to-six-gate B1 report, Found 4"}
---
- **Per iteration:** `combine_channels` averages each channel's
  `combine_kept` χ²/dof, which is formed against each iteration's own σᵢ.
  `budget.rs`'s module doc explains why that statistic is meaningless under
  Pareto-tail weights. Related:
  [vegas-chi2-per-dof-overflows-on-wide-splits](../performance/vegas-chi2-per-dof-overflows-on-wide-splits.md).
- **Per seed:** the same statistic one level up. The quiet seeds' errors are
  correct for what they drew, but read against a mean that one tail seed
  raised.

Filed at the close-out of [two-to-six-gate](../../sprints/two-to-six-gate/sprint.md).
