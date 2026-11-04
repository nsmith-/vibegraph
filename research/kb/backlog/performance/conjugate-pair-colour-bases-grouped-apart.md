---
type: Backlog Item
title: Conjugate subprocess pairs with permuted colour flows are grouped apart
description: "A subprocess and its conjugate can list the same colour flows in a different order; the group join compares CF matrices literally, so such pairs start separate groups and are evaluated separately."
area: performance
state: open
priority: low
closes_when: "The group join compares CF matrices under the flow relabelling π between the two subprocesses, a per-flow JAMP2 oracle on u d > d g u and its conjugate pins that π maps flow to flow, and conjugate pairs such as u b > b g u and u u > g g u u land in one group."
blocked_by: []
opened: 2026-11-04
tags: [colour, flavour-groups, hadronic]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: proton, resource: "../../../../vibegraph-lib/src/proton.rs", title: "vibegraph-lib/src/proton.rs, derive_flavor_groups and colour_pairing"}
  - {id: groups, resource: "../../hadronic/flavour-groups.md", title: "Flavour groups"}
---
With colour compatibility as a group join condition (equal flow count, equal
CF matrix, a successful `flow_permutation`), a subprocess whose conjugate
lists the same flows in another order starts its own group. The CF matrices
are equal under the permutation π that relabels the flows: for `u b > b g u`,
π = [0,2,3,1]; for `u u > g g u u` (12 flows), 96 of 144 entries differ
literally and all agree under π. Comparing under π would keep these pairs in
one group, which saves evaluations.

The relabelling is a convention claim, so a per-flow JAMP2 comparison on
`u d > d g u` and its conjugate has to pin it before the join relies on it.
Measured by the CERN host on branch `colour-split-groups` (729e88af).
