---
type: Backlog Item
title: Majorana and fermion-number-violating models panic instead of refusing
description: "Seven CMS BSM cards reach panics in the HELAS evaluator (run.rs fermion arena reads, kernel.rs resolve_bra_ket) instead of an error saying the model is unsupported."
area: feature
state: open
priority: high
closes_when: "A process with a Majorana fermion and one with a fermion-number-violating vertex are refused before evaluation with an error naming the unsupported fermion flow, by a test, and the three panic sites are unreachable from such models."
blocked_by: []
opened: 2026-10-20
tags: [majorana, fermion-flow, refusal, panic, non-sm-ufo, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
  - {id: run, resource: "../../../../vibegraph-lib/src/helas/eval/run.rs", title: "helas/eval/run.rs GammaVout and Bilinear instruction arms (~:1279, ~:1351)"}
  - {id: kernel, resource: "../../../../vibegraph-lib/src/helas/eval/kernel.rs", title: "helas/eval/kernel.rs resolve_bra_ket (~:569)"}
---
Majorana support is descoped
([majorana-fermions-unsupported](majorana-fermions-unsupported.md)), but the
release scope asks for a hard error, not a crash. Rooting refuses the UFO `C`
operator, yet models whose vertices violate fermion number without it get
through compilation and panic at evaluation:

- `helas/eval/run.rs` ~:1351, the `Bilinear` arm's fermion-arena read
  (cards 047, 053, 054, 061);
- `run.rs` ~:1279, the `GammaVout` arm's read (052);
- `helas/eval/kernel.rs` ~:569, `resolve_bra_ket`'s "a fermion bilinear
  needs exactly one ket and one bra leg" (049).

Measured in the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md).
The spin-2 analogue is
[spin-2-external-panics-instead-of-refusing](spin-2-external-panics-instead-of-refusing.md).
Unchecked: whether a check on fermion-line continuity at diagram or compile
time catches every case these cards hit.
