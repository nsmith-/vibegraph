---
type: Backlog Item
title: bbx_to_ccx_emmm_qcd0 moves away from MadEvent's reported reference as budget grows
description: "On 20 seeds, ByAlpha reads +2.46σ at 600k and +3.47σ at 1.2M against the reported 20-seed MadEvent mean (Neyman's plain-mean replay +2.17 → +3.12); unexplained."
area: validation
state: open
priority: medium
closes_when: "A 2.4M rung on 20 paired seeds, or a per-channel comparison against the MadEvent record, says whether the drift is ours (a budget-dependent bias) or the reference's (iteration weighting, SCATTER), with the falsifier stated."
blocked_by: []
opened: 2026-10-26
tags: [two-to-six, madevent-reference, budget-ladder, bias]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: p1, resource: "../../sprints/floor-bound-neyman/sessions/P1-report.md", title: "floor-bound-neyman P1 report, Step 5 and Found 4"}
---
Pulls use the measured 20-seed spread. Against the equal-weight reference
the same arms read +1.37 → +2.32 (ByAlpha) and +1.17 → +2.07 (plain mean),
so the drift survives either reference. The references and why the cells
stay `info` are in
[two-to-six-integrals-info-on-scattering-references](two-to-six-integrals-info-on-scattering-references.md);
`validated-ew-awaits-refdata-11` holds the related EW discrepancy.
