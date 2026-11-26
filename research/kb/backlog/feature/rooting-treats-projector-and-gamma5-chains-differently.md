---
type: Backlog Item
title: "The rooting exchanges chirality on a projector-written two-gamma chain but never on a Gamma5-written one"
description: "dim6top's [γ,p̸](P_L−P_R) and SMEFTsim's [γ,p̸]γ⁵ are one matrix but need opposite coupling signs on self-conjugate vertices. The binding compensates by sign; the rooting should treat them alike."
area: feature
state: open
priority: medium
closes_when: "A chain's chirality is put in one normal form before rooting, so projector- and Gamma5-written two-gamma chains root identically. The self-conjugate branch of conjugate_couplings is then +1 throughout. The toy CP-odd rows, the SMEFTsim dipole rows and a new charged Gamma5-written two-gamma row all pass."
blocked_by: []
opened: 2026-11-26
tags: [rooting, chirality, gamma5, couplings, helas, cp-violation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-26}
sources:
  - {id: l-d2, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-D2-report.md", title: "L-D2 report, Found 1"}
---
Found by sprint L's L-D2 (#214). `chiral_correction`
(`helas/eval/root_lorentz.rs`) exchanges a projector reached through a gamma on
a line read against the vertex's arrow. That is right through one gamma, but it
leaves a two-gamma chain's chirality flipped, while a `Gamma5` node is never
conjugated.

#214 ships an empirical compensation: `even_chain_chirality_flipped` negates
the term on self-conjugate vertices. Tests pin it, but nothing derives it. A
charged `Gamma5`-written two-gamma chain has no row today.
