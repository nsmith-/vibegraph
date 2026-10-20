---
type: Backlog Item
title: ee_to_ee's five gate seeds scatter far less than their quoted errors
description: χ²/dof 0.07 over five seeds (spread 1.5e-4 of σ against 6.0e-4 quoted, P ≈ 0.009); either the quoted error overstates the spread on this t-channel row or the quintet is a rare draw.
area: validation
state: open
priority: low
closes_when: A 20-seed run at the gate budget shows whether the quoted error matches the per-seed spread, and the result is recorded beside the row's calibration.
blocked_by: []
opened: 2026-10-20
tags: [sigma-gate, seed-sweep, error-estimate, t-channel]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
A χ²/dof far below 1 is as loud a warning as one far above (AGENTS.md,
"Samplers gate statistically"). On tree `340683c` the five gate seeds give
χ²/dof 0.07, `P(χ²₄ < 0.28) ≈ 0.009`. The matched ℓℓj rows also quote
errors above their spread, so this may be the estimator rather than the draw.

Detail: [V1 report, Found 3](../../sprints/sigma-calibration-rerecord/sessions/V1-report.md).
