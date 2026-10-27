---
type: Backlog Item
title: "No banked row can see the CKM orientation beyond one annihilation vertex"
description: "Every banked amplitude row but ubx_to_epve_ckm_no_b_mass has real or i·real couplings, so a conjugated CKM in t-channel or internal W positions passes every gate."
area: validation
state: open
priority: medium
closes_when: "A MadGraph row for a CP-sensitive process at sm-ckm_no_b_mass (u c~ > b b~ d d~ QCD=0, 32 diagrams, a 5e-4 |M|² effect) gates on amplitudes per diagram."
blocked_by: []
opened: 2026-10-26
tags: [ckm, amplitudes, coupling, convention, madgraph-reference]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: v3, resource: "../../sprints/ckm-no-b-mass/sessions/V3-report.md", title: "ckm-no-b-mass V3 report, Found 1"}
---
The CKM conjugation fix (`UFOModel::bound_coupling`, `vibegraph-lib/src/ufo/mod.rs`)
is pinned against MadGraph by one row, `ubx_to_epve_ckm_no_b_mass`: a single
s-channel W whose `u b~` vertex is an annihilation. Across
`validation/madgraph/couplings/*.json` it is the only table whose Fortran
couplings include a genuinely complex value; the others are real or `i·real`,
which the exchange leaves unchanged.

The [V3 report](../../sprints/ckm-no-b-mass/sessions/V3-report.md) measured, at
five RAMBO points, `|M|²(+η)` against `|M|²(−η)`: `u c~ > b b~ d d~` differs by
3.7e-2 (8 diagrams) and its `QCD=0` form by 5.3e-4 (32 diagrams), where the W–quark
vertices sit in annihilation, t-channel and internal positions. The fix maps
old to new bit for bit on those processes, but only the per-vertex argument ties
the new orientation to MadGraph's beyond the one banked vertex.

Bank one of these processes at `sm-ckm_no_b_mass` and gate it per diagram, as
the existing CKM rows are.
