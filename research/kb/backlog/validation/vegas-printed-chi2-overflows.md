---
type: Backlog Item
title: vibegraph's printed VEGAS χ²/dof overflows on cut slices
description: On narrow cut cells the run summary's iteration χ²/dof reads 1e180–1e260 while σ and its error are sane, so the statistic fails exactly where it is needed.
area: validation
state: open
priority: medium
closes_when: The iteration χ²/dof is computed so that a near-zero-variance iteration or channel cannot blow it up, with a test that feeds one, and it reads sanely on a cut cell.
blocked_by: []
opened: 2026-10-15
tags: [vegas, diagnostics, chi2]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: uux, resource: "../../sprints/sigma-uux-wbwb-veto/sessions/V2-report.md", title: "sigma-uux-wbwb-veto V2, Found 4"}
---
Seen independently twice in one day. On `ee_to_mumua` with cuts
`pta 39.4–77, mmll 86–96`, four of five seeds printed χ²/dof of 1e256–1e260.
On sliced `u u~ > w+ b w- b~ / t t~` runs, values like 8845 and integers of
order 1e180 appeared. The likely cause is an iteration or channel with
(near-)zero variance entering the χ² sum.

**Also on a full production card, with no cut slice** (vg-validator,
2026-11-23). On g2HDM card 894 the five v0.2.2 campaign runs (stop on
`target_rel` 0.01, 6–8 iterations, Neyman allocation) print χ²/dof between 4e6
and 6e10, while successive iteration σ values agree within about 1%. Either
the χ² is formed against something other than the printed iteration values,
or the combination is wrong on this card. Not yet read on main.
