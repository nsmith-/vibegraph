---
type: Backlog Item
title: Fixed-budget Neyman has not been shown to beat ByAlpha under the plain-mean combine
description: "The mixed-row σ gates and the Pythia record integrate at --fixed-budget --allocate neyman, measured under the biased count-weighted combine; on bbx, plain-mean Neyman costs more variance × time."
area: performance
state: open
priority: medium
closes_when: "A 20-seed paired comparison on pp_to_ll_0j2j_mlm at the gate's fixed budget, Neyman against ByAlpha under the plain-mean combine, says which allocation the mixed-row gates should use, and the user decides any switch."
blocked_by: []
opened: 2026-10-26
tags: [multichannel, allocation, neyman, mlm, fixed-budget]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: c1, resource: "../../sprints/floor-bound-neyman/sessions/C1-report.md", title: "floor-bound-neyman C1 report, Found 2 and 3"}
---
On `bbx_to_ccx_emmm_qcd0` at 1.2M, plain-mean Neyman's variance × time is
1.84e-14 against ByAlpha's 1.44e-14. Switching to the plain mean moved
`pp_to_ll_0j2j_mlm` @2 by +0.245 pb (paired t +1.85 over 10 seeds) to
+0.88% of MadEvent, pull +2.08, its error 0.38 → 0.48 pb: the gate passes with
less headroom (3σ 1.67 pb against a 1.16 pb offset, about 0.7% of which the
registered deviations explain). The mixed-row gates
(`sigma_ll_0j2j_mlm_vs_madevent`, `sigma_ttx_0j1j_mlm_vs_madevent`) and the
Pythia record use Neyman at a fixed budget. Changing the allocation moves
them, so it needs 20+ seeds on the deciding pair.
