---
type: Backlog Item
title: Sampler and phase-space doc comments describe replaced rules
description: "Doc comments in proton.rs, budget.rs, hadronic.rs, cuts.rs and phasespace/ still cite the 1/σ² combination, the χ²/dof stop, one scale per group and a test-only rung order."
area: hygiene
state: open
priority: low
closes_when: "Every site listed in the body describes the current code."
blocked_by: []
opened: 2026-10-09
tags: [stale-comment, vegas, phase-space, scales, hygiene-sprint]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: vegas, resource: "../../../../vibegraph-lib/src/vegas.rs", title: "vegas.rs IterationCombination, Unweighted is the default (~:121-135)"}
---
Each site was checked against the code on 2026-10-09 (paths under `vibegraph-lib/src/`):

- `proton.rs:2370-2372` (`ProtonIntegrand::scale_channel`): "A group's mirrored term is evaluated at the same scale as its direct one, so there is one draw per group per point". `per_group_sum` calls it again for the mirror at the mirrored argument (~:2191-2207).
- `budget.rs:196-198` (`BlockAllocation::Neyman`): "a starved channel's χ²/dof is what widens the error the stopping rule reads". The stop reads `pooled_scale`, `max(1, emp/quoted)` (~:472-517).
- `hadronic.rs:79-81` (`VEGAS_ALPHA_MAPPED`): "since iterations are combined by `1/σ²`". The default is `IterationCombination::Unweighted`.
- `phasespace/diagram_channel.rs:30-31`: "because VEGAS combines its iterations by `1/σ²`". The same.
- `phasespace/rambo.rs:11`: "Kleiss–Stirling–van der Bij 1986". RAMBO is Kleiss, Stirling and Ellis, Comput. Phys. Commun. 40 (1986) 359.
- `phasespace/diagram_channel.rs:999-1000` (`with_rung_order`): "Nothing derives an ordering this way". `RungOrder::Reversed` uses it in production (`maps.rs` ~:188).
- `cuts.rs:636-643` (`Cuts::spacelike_floor`): "a density regulator, not a kinematic limit … any non-negative value leaves the estimator unbiased". A bounded spine rung also caps `t_max` at `−fiducial_scale` (`diagram_channel.rs` ~:1692, `apply_fiducial_t_max`), which narrows support and relies on the coverage gate.

Re-measuring the tunings these comments justify is
[sampler-tunings-predate-pooled-stop](../performance/sampler-tunings-predate-pooled-stop.md).
Found by drafters D6, D7 and D10 and verifier V7.
