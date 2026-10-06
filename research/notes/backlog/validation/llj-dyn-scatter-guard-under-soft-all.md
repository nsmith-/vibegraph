---
type: Backlog Item
title: pp_to_llj_dyn's five-seed scatter guard holds back soft-all as the default
description: soft-all halves llj's evaluations, but as the default it takes pp_to_llj_dyn's five-seed χ²/dof guard to 4.17 against a 4.0 limit.
area: validation
state: needs-user
priority: medium
closes_when: The pp_to_llj_dyn scatter guard is matched to its measured calibration (never a higher limit) by a statistic the user approves, and soft-all can become the default with the guard passing.
blocked_by: []
opened: 2026-09-23
tags: [phase-space-maps, soft-angle, scatter-guard, calibration, pp-to-llj]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L343-L350", title: "TODO.md entry T022"}
---
`--map-split-angle soft-all` halves `pp_to_llj_dyn`'s evaluations to 0.1%.
`MapOptions::resolve` (`vibegraph-lib/src/phasespace/maps.rs`) still defaults
to soft-emission/isotropic. The σ gate passes under soft-all (+0.23% from
MadGraph, pull +0.69), but the cell's scatter guard (χ²/dof of five seeds about
their mean, limit 4.0) reads 4.17.

`probe_llj_dyn_scatter_guard_calibration` (`validate_hadronic.rs`) runs the
gate's configuration over forty seeds: χ²/dof 0.91 under both maps,
spread/quoted 0.95 (isotropic) and 0.91 (soft-all). 1 of 8 quintets is above
4.0 under soft-all, 0 of 8 under isotropic, and the one is the gate's own. Seed
20260732 reads about 3.6σ low under every map (414.99 isotropic, 414.83
soft-all, against ≈ 416.3). The estimator is honest; the gate drew a
one-in-eight quintet.

Matching the guard to its calibration (more seeds, a pooled statistic) is the
user's decision, and never a higher limit. Flipping the rule is then a one-line
change in `resolve`. Detail:
[note 37 §6.2–6.3](../../37-madevent-map-survey-and-soft-angle.md).
