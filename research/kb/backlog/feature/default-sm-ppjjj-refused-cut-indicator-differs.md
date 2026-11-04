---
type: Backlog Item
title: p p > j j j under the default SM is refused with CutIndicatorDiffers
description: "At maxjetflavor 4 the b quark is its own cut class, so g g > g u u~ and g g > g b b~ fall in one flavour group with different cut indicators, and grouping refuses the process."
area: feature
state: open
priority: medium
closes_when: "p p > j j j with the default SM and maxjetflavor 4 integrates, subprocesses whose cut indicators differ go to separate groups (or one group carries per-member cut indicators), and σ agrees with MadGraph by the multi-seed criterion."
blocked_by: []
opened: 2026-11-04
tags: [flavour-groups, hadronic, cuts, cms-survey]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: proton, resource: "../../../../vibegraph-lib/src/proton.rs", title: "vibegraph-lib/src/proton.rs, CutIndicatorDiffers (~:857)"}
  - {id: groups, resource: "../../hadronic/flavour-groups.md", title: "Flavour groups"}
---
`p p > j j j` under the default SM is refused with `CutIndicatorDiffers`, from
`g g > g u u~` against `g g > g b b~`: at `maxjetflavor 4`, b is a separate cut
class from the light quarks, so the two subprocesses are joined into one group
with different cut indicators. The colour-basis split (branch
`colour-split-groups`) made colour compatibility a join condition. Treating
cut-indicator agreement the same way, so that a disagreeing subprocess starts
its own group, is the candidate fix. Found by the CERN host.
