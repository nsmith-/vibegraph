---
type: Backlog Item
title: "2 → 1 processes (p p > z, p p > zp, e+ e- > z) cannot be integrated: there is no one-body phase space"
description: "vibegraph has no δ(ŝ − M²) one-body phase space, so every 2 → 1 process is refused; MadGraph integrates them with τ = m²/s."
area: feature
state: open
priority: high
closes_when: "p p > z and e+ e- > z integrate through a one-body phase space with the δ(ŝ − M²) Jacobian (MadGraph's genps.f τ = m²/s), with σ matching local MadGraph by the multi-seed criterion on a hadronic and a fixed-beam card, events written with the resonance at rest in the partonic frame, and the OneBodyFinalState refusal removed."
blocked_by: []
opened: 2026-11-07
tags: [phase-space, one-body, resonance, hadronic, fixed-energy, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hg2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-G2-report.md", title: "H-G2 report: G1 and Found 1"}
  - {id: genps, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/genps.f#L276", title: "genps.f: τ = m²/s for one final-state particle"}
---
Every 2 → 1 process is refused before integration:
- **Hadronic beams:** `ProtonError::OneBodyFinalState`.
- **Fixed-energy runs and `generate`:** `refuse_one_body_final_state`.

The phase space has no one-body map. RAMBO and the diagram channels need at least
two final-state momenta. Before #109 these processes panicked instead: in RAMBO via
`derive_flavor_groups`, and on fixed beams in the scale probe.

MadGraph integrates them. For one final-state particle, `genps.f:276` fixes
τ = m²/s with the δ(ŝ − M²) Jacobian, so a hadronic run integrates only the
rapidity and the parton densities.

Until then, a multi-multiplicity card keeps running only when the cuts zero its
one-body part (a lone particle has pT = 0 exactly). That is how DarkPhoton
(`p p > zp` plus jets, `pt_min_pdg` on the zp) runs and matches MadGraph, where
that part contributes 0. A card whose one-body part survives the cuts is refused:
`p p > z` on its own, or a matched `p p > z (j)` card, which is common in production.

The user set the priority high (2026-11-07).
