---
type: Backlog Item
title: "An exactly flat integrand prints its rounding as overweights and a 1.5e-154 error"
description: "A constant-integrand artifact makes generate report every event as overweight (w/w_max 1.000), and a zero-variance integration prints ± 1.5e-154, f64::MIN_POSITIVE leaking through combine_kept."
area: hygiene
state: open
priority: low
closes_when: "generate compares weights to w_max at a relative tolerance, and a run stopped as Constant prints its error as exact or at the rounding floor, each pinned by a test."
blocked_by: []
opened: 2026-10-17
tags: [unweighting, decay-width, output, constant-integrand]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: p1, resource: "../../sprints/vegas-constant-integrand/sessions/P1-report.md", title: "vegas-constant-integrand P1 report, Found 2 and 3"}
---
Since a two-body decay integrates once
([P1](../../sprints/vegas-constant-integrand/sessions/P1-report.md)), its
weights are equal to rounding. `generate` from the `z > e+ e-` artifact
prints `overweight: rate 1.000e0 … largest w/w_max 1.000`, because the
comparison with `w_max` fires on ulp noise. A zero-variance iteration prints
`± 1.5e-154`, which is `f64::MIN_POSITIVE` reaching the printed error through
`combine_kept`. Neither changes a result.
