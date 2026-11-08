---
type: Backlog Item
title: "A narrow s-channel resonance needs ~20M evaluations to reach 0.1% on a 2 → 2"
description: "p p > h > zp zp (Γ_h = 7.3 MeV at 125 GeV) took 144–273 iterations at χ²/dof 1.7–5.8 to reach 0.1%; the channel's Breit–Wigner map against the width is the suspect."
area: performance
state: open
priority: low
closes_when: "The cause of the slow convergence on p p > h > zp zp is found (the Breit–Wigner map's coverage of the width, or otherwise) and fixed, with the iterations and evaluations to 0.1% recorded before and after."
blocked_by: []
opened: 2026-11-07
tags: [phase-space, breit-wigner, resonance, convergence]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hm2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-M2-report.md", title: "H-M2 report, Found 3"}
---
Found by H-M2 on the ZprimeToTauTau survey card. `p p > h > zp zp`, with
Γ_h = 7.3 MeV at 125 GeV, needed 144–273 iteration lines at χ²/dof 1.7–5.8 to
reach 0.1%, about 20M evaluations for a 2 → 2. The iteration χ²/dof above 1 says
the grid keeps moving. H-M2 suggests checking the channel's Breit–Wigner map
against the resonance width (not checked).
