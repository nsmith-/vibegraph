---
type: Backlog Item
title: "Lorentz boosts and pT/η/Δφ are hand-rolled in several modules"
description: "About six Lorentz-boost implementations and three pT/rapidity/Δφ copies exist across helas, hadronic, onshell, lhef, coupling and phasespace."
area: hygiene
state: open
priority: low
closes_when: "Boosts and the transverse kinematics live in one module the others call, except where a copy is kept for documented conditioning reasons."
blocked_by: []
opened: 2026-10-10
tags: [kinematics, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-d-report, resource: "../../sprints/hygiene/sessions/R-D-report.md", title: "R-D report (hygiene sprint)"}
---
Boosts: `helas/repr/lorentz.rs`, `hadronic.rs` `boost_z`, `onshell.rs`, `lhef/resonance.rs`, `coupling/cluster/kt.rs`, `diagram_channel.rs` (the last deliberately γ = E/√s). Transverse helpers: `cuts.rs`, `lhef/observables.rs`, `coupling/scales.rs` (R-D cross-cluster patterns).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-D](../../sprints/hygiene/sessions/R-D-report.md).
