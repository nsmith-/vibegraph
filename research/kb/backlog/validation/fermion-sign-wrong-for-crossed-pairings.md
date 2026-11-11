---
type: Backlog Item
title: "Amplitudes with two interfering fermion pairings carry a wrong relative sign in two cases"
description: "e+ e- > e- e+ gives σ 8.3% above e+ e- > e+ e-; u~ u > u u~ is off up to 27%; u u~ > t t~ b b~ QED<=2 has its W-exchange class at the opposite sign to MadGraph. All silent."
area: validation
state: open
priority: high
closes_when: "e+ e- > e- e+ and e+ e- > e+ e- give the same σ, u~ u > u u~ matches u u~ > u u~ at the exchanged point, and u u~ > t t~ b b~ QED<=2 agrees with MadGraph standalone per flow × helicity; each as an enforced row failing before the fix, every banked row unchanged."
blocked_by: []
opened: 2026-11-11
tags: [amplitudes, fermion-sign, silent-wrong-answer, release]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-11}
sources:
  - {id: sprint, resource: "../../sprints/v021-fixes/sprint.md", title: "Sprint J plan, PR 5"}
---
Found in sprint J (J-W1, J-E1, the manager's check). At fixed energy (SM,
`ebeam = 250`, `ptl 20`, `etal 2.5`, `drll 0.4`, seed 3), `e+ e- > e+ e-` and
`e- e+ > e- e+` give 155.96 ± 0.16 pb and `e+ e- > e- e+` gives
168.92 ± 0.16 pb with symmetric cuts. The pattern is an antiparticle-first
incoming order with a particle-first outgoing order, the same flavour in and
out: `u~ u > u u~` is off 6.6% at one point and up to 27% for `u~ u > z u u~`.
Multi-particle proton labels mirror the antiquark-first order and look immune.

Separately or by the same cause: `u u~ > t t~ b b~ QED<=2` (plain `sm`)
disagrees with MadGraph by 0.495 per flow. The four W-exchange diagrams match
only with the opposite overall sign. `t t~ c c~` and `/ w+ w-` agree. Session
J-F1 diagnoses and fixes both.
