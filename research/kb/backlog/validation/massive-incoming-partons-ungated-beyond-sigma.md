---
type: Backlog Item
title: "Massive incoming partons are checked by σ only: MLM, kT scales and reweighting are unoracled"
description: "#95's oracles are σ rows and per-helicity amplitudes; MLM matching, the kT-clustered scale and reweighting with massive partons have no per-event check, and LL_HAHM's σ is blind to the mass."
area: validation
state: open
priority: medium
closes_when: "A banked row with a percent-level incoming-mass effect gates the massive kinematics, and a matched or kT-scale massive-parton card is compared per event with MadGraph's scales and MLM decisions (validate-scales style); reweighting is checked or refused for massive partons."
blocked_by: []
opened: 2026-11-07
tags: [massive-incoming, mlm, scales, reweight, oracle-blind-spot]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hm1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-M1-report.md", title: "H-M1 report, Found 4"}
  - {id: hm2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-M2-report.md", title: "H-M2 report, Found 4 and 5"}
---
#95 (massive incoming partons) was validated by per-helicity amplitudes against
MadGraph standalone and by σ rows (H-M1, H-M2). Three gaps remain:

- **No per-event oracle** (H-M1 Found 4, H-M2 Found 5). MLM matching now
  clusters with massive partonic-CM beams, as `cluster.f` does; the kT-clustered
  scale beyond 2 → 2 and reweighting also see the masses. None was run against
  MadGraph. ZprimeToTauTau's per-multiplicity σ is the only matched oracle.
- **A blind σ row** (H-M2 Found 4). LL_HAHM's mass effect is 5e-6 of σ, so it
  cannot detect the incoming-mass treatment. A gating row needs a card where the
  massive initial state is a large fraction and the effect is percent-level,
  e.g. H-M1's `b b~ > mu+ mu-` at `mmll = 20` (+5.3%).
- **No banked row** at all for massive incoming partons (inferred from H-M2
  measuring against local MadGraph only).

The σ disagreements on cards with MeV-scale light-quark masses from the closing
campaign are sprint H's own diagnosis, not this item.
