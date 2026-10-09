---
type: Backlog Item
title: Two σ tolerance cells sit under 2× headroom by budget
description: ddx_to_epemg (1.6×) and gux_to_epemux (1.9×) clear rel_tol by under 2× over five seeds; margin is bought with points, a gate-cost decision.
area: validation
state: needs-user
priority: medium
closes_when: The user decides the budget for both rows, and both either clear ≥2× headroom over a ≥5-seed sweep at the chosen budget or carry the decision not to buy the margin.
blocked_by: []
opened: 2026-09-07
tags: [vegas, seed-sweep, headroom, gate-budget]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L414-L442", title: "TODO.md entry T029"}
---
Two `validate_sigma.rs` rows clear `rel_tol` by under 2×, measured over five
seeds. In both, the tolerance is right and the budget is what is thin.

- **`ddx_to_epemg`: 1.6× against `rel_tol` 0.01.** The worst `|rel|` is
  6.296e-3, with mean +4.509e-3. This is a converged +0.45% offset against a
  0.20% reference error, flat to +0.42% at 4× budget, where the worst is
  5.5e-3. The pull is reference-bounded.
- **`gux_to_epemux`: 1.9× against `rel_tol` 0.005.** The worst `|rel|` is
  2.628e-3, from one-seed scatter: one seed at −2.63e-3 and four inside 8.7e-4.
  At 4× budget the worst falls to 1.37e-3.

The decision is gate cost against margin; neither tolerance is to move.

Two other cells are the thinnest beside them:
- `ll_to_qqx_toy_tensor` σ: 2.2×, χ²/dof 2.77, with its channel set collapsed
  from 3 to 1.
- `ee_to_wpwm_cw` samples: KS p 2.4e-4 against the 1e-4 floor.

Detail: [note 36a §1c, §7](../../history/notes/36a-seed-headroom-census.md).
