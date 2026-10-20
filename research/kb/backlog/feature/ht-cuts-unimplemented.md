---
type: Backlog Item
title: "The HT cuts htjmin, ihtmin and htjmax are unimplemented"
description: "HT-binned CMS samples set htjmin (12 of 65 LO cards) and ihtmin (8); both are refused as unimplemented cuts."
area: feature
state: open
priority: medium
closes_when: "htjmin, htjmax, ihtmin and ihtmax are compiled as cuts with MadGraph's definitions (which partons count, the inclusive variant) and an HT-binned card's σ matches MadGraph's within the σ-gate tolerance."
blocked_by: []
opened: 2026-10-20
tags: [cuts, ht, runcard, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
`htjmin`, `htjmax`, `ihtmin` and `ihtmax` are in `UNIMPLEMENTED_CUTS`
(`vibegraph-lib/src/cuts.rs:379-382`); `detect_unimplemented` (`:1035`)
refuses each off its default.

In the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md)
12 of 65 LO cards set `htjmin > 0` and 8 set `ihtmin > 0`: every HT-binned
γ+jets, QCD and V+jets sample. `htjmin` was the first blocker on 9 cards and
`ihtmin` on 1. MadGraph's definitions are in `cuts.f`; see
[MadGraph cut conventions](../../run-card/madgraph-cut-conventions.md).

The refusal itself arrives late and mislabelled:
[unimplemented-cut-refused-after-enumeration](../hygiene/unimplemented-cut-refused-after-enumeration.md).
