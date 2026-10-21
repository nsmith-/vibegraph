---
type: Backlog Item
title: "Proton runs assume equal beams: √s, the parton energies and the lab boost"
description: "On lpp = 1, sqrt_s_had = ebeam1 + ebeam2 and build_frames puts partons at x_i·√s/2 with β = (x1−x2)/(x1+x2), so unequal beams get the wrong √s, frames, lab cuts and written momenta."
area: validation
state: open
priority: medium
closes_when: "A proton run with unequal beams uses √s = 2√(E1E2), partons at x_i·E_i and MadGraph's cm_rap boost (genps.f:345), bit-identical for equal beams, pinned by a unit test at unequal energies and a banked unequal-beam MadGraph σ row."
blocked_by: []
opened: 2026-10-20
tags: [beams, kinematics, runcard, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: hy1, resource: "../../sprints/cut-refusal/sessions/HY1-report.md", title: "cut-refusal HY1 report, Found 2"}
---
The fixed-energy path (`lpp = 0`) is already exact: `beams::partonic_s`
computes ŝ = (p_a + p_b)² and cuts boost into the lab by `lab_beta_z`; only
two docs there still say E1 + E2. The proton path (`lpp = 1`) is equal-beam
only throughout:

- `sqrt_s_had = rc.ebeam1 + rc.ebeam2` (`integrate.rs` ~:931, `generate.rs` ~:1712);
- `ProtonIntegrand::build_frames` (`proton.rs` ~:2004-2019) places the partons
  at x_i·√s/2 and boosts by β = (x1 − x2)/(x1 + x2), so parton energies, the
  lab frame, the lab-frame cuts and the written LHEF momenta are all wrong
  for unequal beams.

MadGraph (b7687064) takes stot from both beams (`genps.f:656-676`),
ŝ = x1·x2·stot (:348) and cm_rap = ½ ln(x1E1/(x2E2)) (:345), and boosts written
events into the lab (`unwgt.f:633-717`). Every validated row has equal beams,
so no gate sees it. Measured in the
[cut-refusal F2 session](../../sprints/cut-refusal/sessions/F2-report.md).
