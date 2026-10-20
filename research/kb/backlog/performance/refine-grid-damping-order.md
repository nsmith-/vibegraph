---
type: Backlog Item
title: refine_grid damps before smoothing, unlike Lepage's VEGAS and MadEvent
description: "refine_grid forms r = m/avg and damps before nearest-neighbour smoothing; Lepage and MadEvent smooth xo = m/Σm first, then damp with ((xo−1)/ln xo)^1.5."
area: performance
state: open
priority: low
closes_when: "A GridRule damping option implements the smooth-then-damp order, measured against today's over 20 seeds on pp_to_llj, pp_to_llj_mlm and pp_to_jj (σ, ε_unw, Hill, quoted²), and the better one is the default or the reason it is not is recorded."
blocked_by: []
opened: 2026-10-18
tags: [vegas, grid-refinement, madevent]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-18}
sources:
  - {id: p1, resource: "../../sprints/vegas-low-acceptance-bins/sessions/P1-report.md", title: "vegas-low-acceptance-bins P1 report, Found 2"}
---
MadEvent: `dsample.f:2157-2174` (damping), `average_grid` at `:2500`. The Σ|w|
histogram losing to Σw² in P1 may be this refinement's property rather than
the histogram's.

Filed at the close-out of [vegas-low-acceptance-bins](../../sprints/vegas-low-acceptance-bins/sprint.md).
