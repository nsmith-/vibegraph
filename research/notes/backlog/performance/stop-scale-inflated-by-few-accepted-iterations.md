---
type: Backlog Item
title: stop_scale makes --target-rel inert on the 2→6 rows
description: Iterations with one to three accepted points have tiny positive variance, so stop_scale inflates by up to ×758 and a 0.2% --target-rel never fires on 2→6.
area: performance
state: open
priority: medium
closes_when: stop_scale qualifies few-accepted-point iterations (a minimum accepted count or equivalent), and a 0.2% --target-rel stops on the 2→6 rows across ≥5 seeds with the achieved error matching the seed spread.
blocked_by: []
opened: 2026-08-06
tags: [budget, stop-rule, target-rel, two-to-six, wide-splits]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1044-L1058", title: "TODO.md entry T098"}
---
`ChannelHistory::stop_scale` (`vibegraph-lib/src/budget.rs:472`, via
`pooled_scale`) widens each channel's quoted variance by `max(1, emp/quoted)`
over its kept iterations. On 2→6 rows some channel iterations hold one to three
accepted points, so their sample variance is tiny but strictly positive, and
the factor explodes.
- There are no zero-variance (all-points-cut) iterations: on every wide-row seed
  the count is zero. A fix that filters on `variance == 0` would miss the whole
  effect.
- The factor cannot be calibrated as a constant. `scaled_rel/achieved_rel`
  spanned ×4.8–×30 270, structured by the α draw. Since the accepted-point floor
  it spans ×3.4–×758: a 1 % target fires on some seeds, but a 0.2 % target
  cannot.

The fix is in what `stop_scale` does with few-accepted-point iterations, such as
a minimum-accepted-count qualification. It is never a retuned factor, and the
reported statistic stays as it is. No such qualification exists in `budget.rs`
today (checked 2026-10-06).

The plain quoted error at the 40k survey cap is well calibrated: achieved_rel is
0.0021–0.0024 against a realized sd/σ of 0.0021, and 2× optimistic below the
cap.

Detail: [note 34 §2 S1 Part C and S3](../../34-draw-followup-plan.md).
