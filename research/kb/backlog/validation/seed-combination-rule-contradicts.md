---
type: Backlog Item
title: "The seed-combination rule for this side's seeds contradicts itself"
description: "The grammar σ gate combines this side's seeds by 1/σ², the MLM gates unweighted; the written policy says 1/σ², combine_seeds' doc says 1/σ² is biased on equal-budget seeds."
area: validation
state: open
priority: medium
closes_when: "One rule is chosen and written in the seed policy, and every gate uses one helper implementing it."
blocked_by: []
opened: 2026-10-10
tags: [seed-policy, sigma-gate, user-call]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-g2-report, resource: "../../sprints/hygiene/sessions/R-G2-report.md", title: "R-G2 report (hygiene sprint)"}
---
R-G2.1. The user deferred the decision on 2026-10-09 and made it on
2026-10-14: seeds of equal budget, on either side, are combined by their
**unweighted mean**, with uncertainty `max(quoted, spread/√n)`, where "quoted"
is `√(Σσᵢ²)/n` and "spread" the seeds' sample standard deviation. The χ²/dof
of the seeds about that mean stays a separate scatter check. A weighted mean
is rejected because a seed that comes out biased with a small variance (a
missed region) is up-weighted by it. Kb sides: [seed-sweeps-and-budget-ladders](../../validation/seed-sweeps-and-budget-ladders.md) vs [madevent-reference-seed-policy](../../validation/madevent-reference-seed-policy.md).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-G2](../../sprints/hygiene/sessions/R-G2-report.md).
