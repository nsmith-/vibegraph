---
type: Backlog Item
title: The sampler and phase-space plugin seams are being designed separately
description: "The importance-sampler seam, cut-aware windows, multi-tier grids and MadSpin-style decay plugins cut through one draw → map → weight path; one design session should settle both seams together."
area: feature
state: needs-user
priority: high
closes_when: "A design concept agreed with the user in a dedicated session defines the importance-sampler seam and the phase-space-map seam together. It says which side owns coordinate windows (including cut-aware conditional windows), any shared (τ, y) tier, per-dimension bin-width bounds, and decay plugins of the MadSpin kind. It keeps the F: Real genericity the sampler seam requires. sampler-hardwired-to-vegas-grid and madspin-decays-unsupported are rewritten against it."
blocked_by: []
opened: 2026-10-20
tags: [sampler, phase-space-map, plugin, vegas, madspin, design, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: tiers, resource: "../../phase-space/multi-tier-vegas-and-cut-edges.md", title: "Multi-tier VEGAS, absolute grid coordinates and sharp cut edges (draft)"}
  - {id: seam, resource: "../../phase-space/importance-sampler-seam.md", title: "A pluggable importance sampler (proposed)"}
  - {id: m1, resource: "../../sprints/vegas-absolute-grid-coords/sessions/M1-report.md", title: "M1: where the weight tail meets the cut edges, per channel"}
---
The user asked (2026-10-20) for the pluggable sampler and the pluggable phase
space to be designed together.

Several pieces of planned work meet in the same draw → map → weight path:
- [The sampler seam](../../phase-space/importance-sampler-seam.md) puts a
  density in front of a fixed map.
- The [multi-tier draft](../../phase-space/multi-tier-vegas-and-cut-edges.md)
  proposes cut-aware conditional windows. Each window is narrowed per
  coordinate given the earlier ones, which belongs to the map. It also proposes
  a shared (τ, y) tier and a per-dimension bin-width bound, which belong to the
  sampler.
- Decays of the MadSpin kind
  ([madspin-decays-unsupported](madspin-decays-unsupported.md)) are in scope
  only as a phase-space plugin, a seam that does not exist yet.
- Per-coordinate windowed draws do not compose with whole-point samplers such as
  flows. So which side owns the window is the central question.

Designed apart, each seam would fix an interface the other then has to work
around. The design session should start from the measurements in
[M1's report](../../sprints/vegas-absolute-grid-coords/sessions/M1-report.md),
taken on `pp_to_llj_fixed`, `pp_to_jj` and `pp_to_llj_mlm`. They cover:
- each edge's share of σ, variance and w_max per channel;
- whether the trained (τ, y) grids agree across channels;
- what a rejected point costs against an accepted one.

M1 left four inputs the session must settle (its Found 1–4):
- the multi-multiplicity MLM tail (`pp_to_ll_0j2j_mlm`) is unmeasured;
- a shared (τ, y) tier needs an offline variance replay to the pooled q*;
- where the interior half of the ℓℓj tail sits is unmeasured;
- the ℓℓj jet-pT window is two intervals in all-timelike channels.

Rewritten against the outcome:
- [sampler-hardwired-to-vegas-grid](sampler-hardwired-to-vegas-grid.md)
- [madspin-decays-unsupported](madspin-decays-unsupported.md)
- [vegas-grid-coords-not-absolute](../performance/vegas-grid-coords-not-absolute.md)
