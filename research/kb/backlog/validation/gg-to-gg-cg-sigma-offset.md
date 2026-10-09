---
type: Backlog Item
title: gg_to_gg_cg's cross section sits at a converged −0.22% offset
description: gg_to_gg_cg σ converges to rel −2.21e-3 over five seeds, 2.6× the reference's own error; neither the scale formula nor the process explains it.
area: validation
state: open
priority: high
closes_when: The −0.22% offset is attributed with a measurement and the row's integrals cell is flipped from info to gate (or the attribution says why it stays info).
blocked_by: []
opened: 2026-09-07
tags: [smeftsim, sigma, scale-fallback, dynamical-scale-choice-3, info-cell]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L414-L442", title: "TODO.md entry T029"}
---
`gg_to_gg_cg` is the one banked σ taken at a `setscales.f` closed form
(`dynamical_scale_choice = 3`). It is a `SCALE_FALLBACK_ROWS` member
(`vibegraph-lib/tests/validate_sigma.rs:218`).

Its `integrals` cell is `info`. The measurement:
- Five seeds are mutually consistent (χ²/dof 1.01), with an inverse-variance
  mean at rel **−2.21e-3**. The reference's own relative error is 8.5e-4.
- The budget ladder settles rather than shrinks, at −1.2e-3 to −2.9e-3 across
  sixteen times the budget, so this is not sampling.

What it is not:
- **The scale formula.** `validate_scales` replays all 10000 banked events,
  worst at 0.999 of the printing budget, with zero misses.
- **The process.** `gg_to_gg` under a card differing in that one field sits at
  +9.8e-6.

What remains is localised to the coupling this row runs at across the cut
region, rather than on MadGraph's kept events. The `samples` cell gates, but it
cannot see a normalisation offset: every statistic there is computed on
normalised distributions.

Detail: the row's notes in `validation/manifest.toml`;
[note 36 §7](../../history/notes/36-banked-open-ends-plan.md).
