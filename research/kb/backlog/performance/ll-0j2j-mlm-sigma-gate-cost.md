---
type: Backlog Item
title: "pp_to_ll_0j2j_mlm's σ gate costs about 12 min a seed on 4 cores"
description: "validate-mlm-sigma's ll_0j2j row takes about 2 h for its ten seeds on a 4-core host, so the MLM σ gate does not fit a cloud dev session."
area: performance
state: open
priority: low
closes_when: "The row's cost per seed is recorded on the reference host and either cut or given a diagnostic budget that a 4-core session can run, with the gate's own budget unchanged."
blocked_by: []
opened: 2026-10-17
tags: [mlm, sigma-gate, cost]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v1, resource: "../../sprints/sigma-pulls-lean-high/sessions/V1-report.md", title: "V1 report, Found"}
---
In [V1](../../sprints/sigma-pulls-lean-high/sessions/V1-report.md) one seed of
`sigma_ll_0j2j_mlm_vs_madevent` took 12 min on the 4-core cloud host, about
2 h for ten, so V1 left the row out of its population. The three llj MLM rows
took 1339.5 s together.
