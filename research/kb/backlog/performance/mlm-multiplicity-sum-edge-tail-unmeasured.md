---
type: Backlog Item
title: The edge and tail probe does not cover a multi-multiplicity MLM row
description: "M1 measured single-multiplicity rows only; pp_to_ll_0j2j_mlm, a MultiplicitySum where the outer (τ, y) edge bins once carried the tail, is unmeasured."
area: performance
state: open
priority: low
closes_when: "The M1 probe (near-edge tail shares, rejection causes, edge classes) is driven per part through MultiplicitySum::value_in_channel with the lab frame per part, and reports pp_to_ll_0j2j_mlm at five seeds."
blocked_by: []
opened: 2026-10-22
tags: [vegas, weight-tail, mlm, measurement]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
sources:
  - {id: m1, resource: "../../sprints/vegas-absolute-grid-coords/sessions/M1-report.md", title: "M1 report, Found"}
---
`edge_probe.rs`'s `rows()` covers single integrands. `pp_to_ll_0j2j_mlm` is a
three-part `MultiplicitySum`, where
[the weight-tail study](../../phase-space/vegas-grid-weight-tail.md) saw the
outer `(τ, y)` edge bins carry the tail before `K = 6`. Whether its tail is
window, sampler or tier territory under
[the joint seam design](../../phase-space/sampler-and-map-seams.md) is unknown.
