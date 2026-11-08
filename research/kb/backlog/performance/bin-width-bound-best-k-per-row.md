---
type: Backlog Item
title: The best bin-width bound K differs by row; a per-dimension bound is unexplored
description: "ε_unw peaks at K = 2 on the Drell–Yan + jet rows (at ×1.2–1.4 quoted variance) and near K = 6 on pp_to_jj; the shipped default is a single K = 6."
area: performance
state: open
priority: low
closes_when: "An offline study of per-dimension bin widths in P2's trained grids (every run's grid.bin.zst) decides whether a per-dimension bound buys K = 2's efficiency without its variance, and if so it is measured over 20 seeds against K = 6."
blocked_by: []
opened: 2026-10-18
tags: [vegas, grid-rule, unweighting]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-18}
sources:
  - {id: p2, resource: "../../sprints/vegas-low-acceptance-bins/sessions/P2-report.md", title: "vegas-low-acceptance-bins P2 report, Found 2"}
---
P2's grids are in the scratchpad of the session that ran it, not in the
repository; re-running P2's driver regenerates them.

The mechanism is settled: the VEGAS config addresses the bound per coordinate
role, with `K = 6` on every role by default
([joint seam design](../../phase-space/sampler-and-map-seams.md), D4); this
study reads P2's grids by role and decides whether any override is worth it.

Filed at the close-out of [vegas-low-acceptance-bins](../../sprints/vegas-low-acceptance-bins/sprint.md).
