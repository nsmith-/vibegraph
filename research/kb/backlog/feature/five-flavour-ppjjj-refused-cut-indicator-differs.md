---
type: Backlog Item
title: Five-flavour p p > j j j at maxjetflavor 4 is refused with CutIndicatorDiffers
description: "With a massless b and maxjetflavor 4, g g > g u u~ and g g > g b b~ share a matrix element but compile to different cut filters, so grouping refuses one of the most common hadronic processes."
area: feature
state: open
priority: high
closes_when: "p p > j j j under sm-no_b_mass with a run card that sets maxjetflavor 4 integrates, subprocesses whose cut indicators differ go to separate groups (or one group carries per-member cut indicators), and σ agrees with MadGraph by the multi-seed criterion."
blocked_by: []
opened: 2026-11-04
tags: [flavour-groups, hadronic, cuts, cms-survey]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: proton, resource: "../../../../vibegraph-lib/src/proton.rs", title: "vibegraph-lib/src/proton.rs, CutIndicatorDiffers (~:857)"}
  - {id: groups, resource: "../../hadronic/flavour-groups.md", title: "Flavour groups"}
  - {id: default, resource: "../validation/absent-run-card-maxjetflavor-ignores-beam-flavours.md", title: "The absent-run-card maxjetflavor default"}
---
`import model sm-no_b_mass` + `generate p p > j j j` with `maxjetflavor 4`
fails with "subprocesses g g > g u u~ and g g > g b b~ share a matrix element
but compile to different cut filters, so no single cut indicator covers the
sum" (reproduced on main 23d3a766, debug build, 2026-11-04). The default `sm`
is not affected: its b is massive, so `j` has no b and the same card runs.

At `maxjetflavor 4` a b is its own cut class, so the two subprocesses have the
same matrix element and different cuts. MadGraph runs this card. CMS 5F cards
that keep `maxjetflavor 4` hit it, and so does every 5F card run with no run
card until the [default is fixed](../validation/absent-run-card-maxjetflavor-ignores-beam-flavours.md).
The colour-basis split (branch `colour-split-groups`) made colour
compatibility a join condition. Treating cut-indicator agreement the same
way, so that a disagreeing subprocess starts its own group, is the candidate
fix. Found by the CERN host.
