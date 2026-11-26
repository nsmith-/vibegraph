---
type: Backlog Item
title: "`import model X-restrict` is not applied, so an order the restriction removes is accepted"
description: "vibegraph loads the unrestricted UFO. On card 6, FCNC=0 is accepted, while MG 3.7.1 refuses it on the restricted dim6top model, which has no FCNC order left."
area: feature
state: open
priority: low
closes_when: "A model's restriction card is applied as MadGraph applies it (0 removes, 1 fixes, others kept), or its absence is reported. The coupling orders are those of the restricted model, with a test that an order the restriction removes is handled as MG handles it."
blocked_by: []
opened: 2026-11-26
tags: [ufo, restriction, coupling-orders, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-26}
---
Found by sprint L's L-D1 probe on card 6. The restricted and unrestricted
dim6top models differ only by vertices with zero coupling, so σ is unaffected
there. What differs is the accept/refuse behaviour, and any restriction entry
equal to 1, which MadGraph folds into a fixed parameter.
