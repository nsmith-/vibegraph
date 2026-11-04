---
type: Backlog Item
title: "A proton card that generates one beam ordering is integrated in both, doubling σ"
description: "Subprocess::has_mirror is incoming[0] != incoming[1], so generate b b~ > mu+ mu- gets the mirrored ordering MadGraph only adds when the card generates it: 81.68 pb against MadGraph's 40.79."
area: validation
state: open
priority: high
closes_when: "Each subprocess carries whether its mirrored ordering was generated (MadGraph's mirror_process), the mirror term gets luminosity only then, and σ rows on explicit single-ordering and both-ordering cards, massless and massive, match MadGraph by the multi-seed criterion."
blocked_by: []
opened: 2026-11-04
tags: [proton, beams, mirror, madgraph-parity, silent-wrong-answer]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: hm1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-M1-report.md", title: "H-M1 report, Found 1 and §5"}
---
`Subprocess::has_mirror` (`vibegraph-lib/src/proton.rs:198`) returns
`incoming[0] != incoming[1]`, so every subprocess with unequal incoming
partons gets its mirrored beam ordering's luminosity, whatever the proc card
generated. MadGraph adds the mirror only when the card generates both
orderings (`mirror_process`).

Measured in sprint H's H-M1 (local MadGraph at the pinned b7687064,
`nn23lo1`): `generate b b~ > mu+ mu-` gives 81.68 pb here, against MadGraph's
40.79 pb with b on beam 1 in all 10 000 events. MadGraph's 81.59 pb is the card
that generates both orderings. `g b > z b` gives 497.2 pb in MadGraph against
994.98 pb with both orderings.

The massless case (`generate u u~ > e+ e-`) runs through the same line, but it
has not been measured. Multiparticle cards (`p p > …`) generate both orderings
and are not affected. This is a silent wrong answer, not a refusal.
