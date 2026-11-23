---
type: Backlog Item
title: "The reported sample σ error drops the −1/T term and overstates the error ~1.4× at 50% efficiency"
description: "sample_estimate_error = σ·√Σw²/Σw omits −1/T; the error is too large by 1/√(1−ε). Samples σ χ²/dof read against it (0.05–0.7) look falsely consistent."
area: validation
state: open
priority: medium
closes_when: "The estimator carries T (σ̂·√(Σw²/(Σw)² − 1/T)), the pull bounds that read it (e.g. cli_generate_proton's SAMPLE_PULL_MAX) are re-derived, a seed-sweep test pins the error against the measured spread, and the samples σ verdicts that leaned on χ²/dof < 1 are re-read."
blocked_by: []
opened: 2026-11-22
tags: [samples-gate, statistics, error-estimate, generate]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
sources:
  - {id: ls1, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-S1-report.md", title: "L-S1 report: Found 1"}
---
The estimator is σ̂ = W·Σwᵢ/T. Per trial its variance is estimated by
σ̂²·(Σwᵢ²/(Σwᵢ)² − 1/T). The shipped `sample_estimate_error` drops the 1/T
term. For unit weights the overstatement is 1/√(1−ε): 1.41× at ε = 0.5, and
1.29× at ε = 0.41. Its doc's "1–2%" holds only for ε ≲ 0.04.

Evidence from sprint L's PR 13:
- A toy at ε = 0.248, 400 seeds × 2000 events, gives a measured spread of
  1.85–1.91% against a binomial 1.94%. The shipped form quotes 2.24%.
- The five-seed samples σ χ²/dof read 0.05–0.72 on most rows.

**Every samples σ verdict that leaned on χ²/dof < 1 was read against an
overstated error and needs re-reading after the fix.** The error reaches the
file header's sample estimate and the reported pull.
