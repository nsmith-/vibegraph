---
type: Backlog Item
title: "VBFRadionTo2W M1800's VEGAS iterations disagree (χ²/dof 8–26) before the cap"
description: "Card 636's σ matches its gridpack (vg/MG ≈ 0.99 at 2.2e-15 pb), but its VEGAS iterations jump +2σ with χ²/dof 26 falling to 8, and it hit the time cap before converging."
area: validation
state: open
priority: low
closes_when: "The iteration inconsistency on card 636 is attributed to a channel or a grid-adaptation step and fixed, or shown to vanish with budget; σ is then gated against the gridpack's 2.222e-15 ± 4.2e-18 pb over ≥5 seeds."
blocked_by: []
opened: 2026-11-23
tags: [vegas, convergence, vbf, timeout, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Card 636 (15 parts, 1544 flavour groups, 236 channels) ran 11 VEGAS
iterations in 802 s to 5.65e-24 GeV⁻² (2.2e-15 pb) before the cap. The
estimate jumped about +2σ between iterations 4 and 5, with χ²/dof 26.5
falling to 8.0; at the stop rule's widened error it was ~3% from a 1%
target.

**The σ itself is not wrong.** The gridpack's MadGraph 2.9.18 gives
2.222e-15 ± 4.232e-18 pb (gen_log and `results.dat`; vg-validator,
2026-11-23), so vg/MG ≈ 0.99 on iterations 6–11. The tiny σ is the model
point's own (a near-decoupled radion). With the run unconverged that is a
consistency reading, not a gate.

Evidence: [L-T2 report](../../sprints/v023-orders-majorana-timeouts/sessions/L-T2-report.md), [phase sample](../../sprints/v023-orders-majorana-timeouts/v022-1009-phase-sample.md).
