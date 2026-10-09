---
type: Backlog Item
title: The α-survey bounds were tuned at ~24 channels, not hundreds
description: MIN/MAX_ADAPT_SURVEY clamp the survey at 10k–40k points; at 579–615 channels a channel at α = 10⁻³ draws ~40 own-map points per iteration.
area: performance
state: open
priority: low
closes_when: The n_survey ∈ {10k, 40k, 160k, 640k} experiment is run on the 2→6 rows with pp_to_llj as control, and the cap is kept or made channel-count-aware on its result.
blocked_by: []
opened: 2026-08-06
tags: [multichannel, alpha-survey, budget, two-to-six, wide-splits]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1059-L1075", title: "TODO.md entry T099"}
---
`MIN_ADAPT_SURVEY = 10_000` and `MAX_ADAPT_SURVEY = 40_000`
(`vibegraph-cli/src/integrate.rs:97-98`) clamp `--neval` to set the α-survey's
points per iteration. Above 40k, `--neval` buys no better split. The cap is
confirmed on the rows measured, but those rows carry about 24 channels.
Own-map exploration scales with `αⱼ`, so at the cap a channel at `αⱼ = 10⁻³`
draws about 40 points from its own map per iteration, and `Wⱼ` is what would
raise its `αⱼ`. Per-channel estimator starvation is not the concern, because
every drawn point updates every `Wⱼ`.

The experiment: fixed seed, `n_survey ∈ {10k, 40k, 160k, 640k}`. Record the α
trajectory, the converged α vector, and σ with its ≥5-seed spread from the run
those αs drive. Run it on `bbx_to_ccx_emmm_qcd0` / `uux_to_ccx_emmm_qcd0` (615
and 579 channels, banked σ), with `pp_to_llj` (24 channels) as control.

The survey's per-point cost is the `Σⱼ αⱼgⱼ` loop. Measuring it just before a
density-loop change would price a loop that is about to change. A clamp already
warns, as does an iteration whose spend is set by the `MIN_CHANNEL_NEVAL` floor.

Detail: [note 34 §2 S1](../../history/notes/34-draw-followup-plan.md).
