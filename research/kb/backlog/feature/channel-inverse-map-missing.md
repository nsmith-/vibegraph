---
type: Backlog Item
title: "No phase-space channel maps a momentum configuration back to its unit-cube coordinates, so a trained grid's density at a given event cannot be evaluated"
description: "DiagramChannel::density_at rebuilds a channel's Jacobian from invariants but never the coordinates u, so a coverage check at MadGraph's events can use only untrained grids, not trained VEGAS grids."
area: feature
state: open
priority: medium
closes_when: "Each channel map has an inverse p → u (e.g. DiagramChannel::coordinates_at(√ŝ, momenta)) with the outer (τ, y) inverse, a round-trip test u → p → u to 1e-12 on every map kind, and a trained grid's density at given momenta (VegasGrid jacobian at u × the channel density) is evaluable."
blocked_by: []
opened: 2026-11-28
tags: [phasespace, multichannel, vegas, diagnostics, coverage]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-28}
---
Found by `vg-validator` while localising the γ+4j deficit of
`four-jet-photon-cards-confidently-low-at-campaign-budget`. Evaluating the
multichannel density g = Σ αⱼ gⱼ at MadGraph's own events works for untrained
grids, because `density_at` rebuilds each channel's Jacobian from the event's
invariants. A trained VEGAS grid's density needs the point's coordinates u in
each channel, and no map provides the inverse. The analytic outer (τ, y) inverse
exists for `InverseSquare` only in a scratch harness, and
`VegasGrid::jacobian_factors` is `pub(crate)`.

Without it, whether training moves density away from a region the survey found
can be tested only through generated events (vibegraph's unweighted sample
against MadGraph's), which costs a full integrate and generate.
