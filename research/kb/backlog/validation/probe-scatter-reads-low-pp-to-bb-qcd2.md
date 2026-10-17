---
type: Backlog Item
title: "Two seed sets scatter far below their quoted errors"
description: "pp_to_bb_qcd2's fresh seeds scatter at χ²/dof 0.03 (0.27 over ten), and the fixed-density q q̄ → e⁺e⁻g probe reads χ²/dof 0.82 on 199 dof at 200 × 100k but 1.02 at 2000 × 10k."
area: validation
state: open
priority: low
closes_when: "pp_to_bb_qcd2's per-iteration variance is compared with its iteration scatter over twenty seeds, and the fixed-density probe's χ²/dof is read at 200 × 1M; each is either nominal or its error is attributed."
blocked_by: []
opened: 2026-10-17
tags: [sigma, seeds, error-calibration]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v1, resource: "../../sprints/sigma-pulls-lean-high/sessions/V1-report.md", title: "V1 report, Found"}
  - {id: v2, resource: "../../sprints/sigma-pulls-lean-high/sessions/V2-report.md", title: "V2 report, Found 2"}
---
A χ²/dof far below 1 is as loud a warning as one far above (AGENTS.md):

- **`pp_to_bb_qcd2`.** Five fresh seeds scatter at χ²/dof 0.03, with spread
  2.52e4 against a quoted 1.36e5, and 0.27 over ten
  ([V1](../../sprints/sigma-pulls-lean-high/sessions/V1-report.md)). Either
  this side's quoted error overstates its scatter on this row, or its seeds
  are correlated.
- **`probe_fixed_density_llj_annihilation`.** It reads 0.82 and 0.80 on 199
  dof at 200 × 100k, about 2σ low, and is calibrated at 2000 × 10k (1.02,
  1.01) ([V2](../../sprints/sigma-pulls-lean-high/sessions/V2-report.md)). Its
  verdict does not depend on this.
