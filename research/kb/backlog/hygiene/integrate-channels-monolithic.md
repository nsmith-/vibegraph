---
type: Backlog Item
title: "integrate_channels is one 260-line loop, and the cut-inactivity convention is written twice"
description: "budget.rs's integrate_channels does plan, draw, record, report, stop and refine inline; Cuts::pass and Cuts::admits_every_point each encode which bounds mean 'off'."
area: hygiene
state: open
priority: low
closes_when: "integrate_channels is split with the seeded VEGAS goldens bit-equal, and one is_inactive per cut type serves both Cuts::pass and admits_every_point."
blocked_by: []
opened: 2026-10-17
tags: [budget, vegas, cuts, maintainability]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: hy1, resource: "../../sprints/vegas-constant-integrand/sessions/HY1-report.md", title: "vegas-constant-integrand HY1 report, Found 1 and 2"}
---
`integrate_channels` (`vibegraph-lib/src/budget.rs` ~:669) is about 260 lines.
One iteration plans, draws, records, reports, checks the stop and refines
inline ([HY1](../../sprints/vegas-constant-integrand/sessions/HY1-report.md),
Found 1). It is the hot loop, so a split is a performance-dev session with
`to_bits` equality of the seeded goldens as its identity oracle. Separately,
`Cuts::pass` and `Cuts::admits_every_point` each encode, per cut family, that
a lower bound at or below 0 or an upper bound below 0 means "off" (Found 2).
An `is_inactive` on `SingleLegCut` and `PairCut` would keep that convention in
one place, though it touches `pass`, the hot cut path.
