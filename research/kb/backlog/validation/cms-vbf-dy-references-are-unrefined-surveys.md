---
type: Backlog Item
title: The CMS VBF/EWK Drell-Yan MadGraph references are single unrefined surveys
description: "The three CMS VBF/EWK DY gridpacks' σ come from one MadGraph 2.9.18 survey each (no refine, one seed); a converged MadGraph 3.7.1 seeded reference is needed before they gate anything."
area: validation
state: open
priority: medium
closes_when: "Each of the three cards (DYto2L-2Jets-EWK_Bin-MLL-50, VBFto2L_Bin-MLL-50, VBFto2L_Bin-MLL-4to50) has a banked MadGraph 3.7.1 σ at ≥5 fixed seeds with refine, at its shipped param card and PDF, and vibegraph's σ on each is compared against it under the seed policy."
blocked_by: []
opened: 2026-10-31
tags: [cms, vbf, drell-yan, reference, madgraph, seed-policy]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-31}
---
The CERN host read MadGraph's own convergence record from the tarballs
(branch `vbf-dy-spine-inputs`, 3e3b5d37, `mg_channels.csv`). All three ran
`survey pilotrun --accuracy=0.01 --points=2000 --iterations=8` with no refine
and `iseed = 0`. VBFto2L_Bin-MLL-4to50's τ part is not converged: its dominant
channel holds 82% of the part, is still rising, and has χ²/dof 5.24.

On DY MLL-50, vibegraph reads 1.020 ± 0.005 of that survey at 4× budget
(sprint E, T-F2). Against a properly banked generic row of the same class
(`pp_to_epemjj_qcd0`, since retired: at `ptj = 0` it diverges at tree level,
sprint I D1), vibegraph agreed at pull −0.92. So the CMS-card residual
cannot be read until the references are converged. Each card's survey cost 2–4
CPU hours, and a seeded bank with refine costs more. The cards need
`loop_sm`/`ckm_no_b_mass`, their shipped `param_card.dat` and PDF 325300.
