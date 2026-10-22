---
type: Backlog Item
title: pp_to_jj draws outside its exact 2 → 2 jet-cut window
description: "The jet pT and η cuts of a 2 → 2 jet process are an exact window given (τ, y); drawing inside it would remove every rejection, a ceiling of about 1.3× in variance × CPU."
area: performance
state: open
priority: low
closes_when: "A map option (off by default, recorded in MapChoices) narrows pp_to_jj's polar coordinate to |t| ∈ [t₋, ŝ − t₋] and |cos θ*| < tanh(η_max − |y|) given (τ, y); a point-level test finds zero accepted window-off points outside the window in ≥10⁶ draws per channel; σ with the window on agrees with it off at ≥5 seeds per arm, and the variance × CPU gain is measured at 20+ seeds on the deciding pair."
blocked_by: []
opened: 2026-10-22
tags: [phase-space-map, cut-edge, window, pp_to_jj]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
sources:
  - {id: design, resource: "../../phase-space/sampler-and-map-seams.md", title: "The importance sampler and the phase-space map, designed together (D1)"}
---
M1 found that two thirds of `pp_to_jj`'s top-0.1% second moment sits within 5%
of the jet `pT` edge in the `t` and `u` channels, and that the edge is pinned by
`τ` alone (`R = 0.08`); the `η` edge is pinned by `(τ, y)`. Together they are all
of the row's rejections. At fixed `ŝ` the accepted region is exact:
`t₋ = (ŝ/2)(1 − √(1 − 4ptj²/ŝ))`, with `|y| < η_max` and
`|cos θ*| < tanh(η_max − |y|)` given `y`.

Under [the joint seam design](../../phase-space/sampler-and-map-seams.md) (D1)
the map owns the window: relative coordinates, the window's width in the map's
weight, and `density_at` recomputing it at a foreign point. It is the first
window to build: exact, clean oracle, M5 ceiling 1.30×.
