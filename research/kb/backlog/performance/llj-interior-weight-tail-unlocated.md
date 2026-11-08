---
type: Backlog Item
title: Where the interior half of the ℓℓj weight tail sits is unmeasured
description: "53–76% of the ℓℓj top-0.1% second moment lies away from every cut edge; its location decides how much the sampler route (VEGAS+, flows) can win."
area: performance
state: open
priority: low
closes_when: "The non-edge remainder of the ℓℓj tail (points beyond ε = 5% of every edge) is binned by region at the trained grids on pp_to_llj_fixed and pp_to_llj_mlm, five seeds, with the share of the top-0.1% second moment per region and coordinate reported with its spread."
blocked_by: []
opened: 2026-10-22
tags: [vegas, weight-tail, measurement, pp_to_llj]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
sources:
  - {id: m1, resource: "../../sprints/vegas-absolute-grid-coords/sessions/M1-report.md", title: "M1 report, tail table and Found"}
---
M1 found only 24–47% of the ℓℓj top-0.1% second moment within 5% of any edge;
the rest is interior, and no window reaches it. Under
[the joint seam design](../../phase-space/sampler-and-map-seams.md) (D1) that
share is the sampler's to take, so where it sits — by grid coordinate, `m_ll`,
jet `pT`, `√ŝ` — sizes the VEGAS+ and flow candidates. Base: M1's
`edge_probe.rs` plus the region binning of `probe_llj_weight_tail_regions`.
