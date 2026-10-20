---
type: Backlog Item
title: Unimplemented cuts set to no-op values are refused as active
description: "missetmax and ptj1max..ptj4max = 1e5 cannot bind at LHC energies, yet differ from the -1 default and are refused; 3 CMS cards hit it."
area: feature
state: open
priority: medium
closes_when: "An unimplemented cut set at or beyond the value where it cannot bind (an upper bound at or above the collider's √s, for example) is treated as its default, while one that could bind is still refused, by tests on both."
blocked_by: []
opened: 2026-10-20
tags: [cuts, runcard, refusal, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
`detect_unimplemented` (`vibegraph-lib/src/cuts.rs:1035-1049`) refuses any
`UNIMPLEMENTED_CUTS` entry that differs from its default. `missetmax` and
`ptj1max`…`ptj4max` default to -1 (off); `1e5` is also off at any LHC
energy, since no transverse momentum reaches it, but is refused.

In the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md)
3 of 65 LO cards carry such values; `missetmax` was the first blocker on 1.
