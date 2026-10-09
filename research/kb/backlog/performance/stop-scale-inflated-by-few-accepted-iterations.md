---
type: Backlog Item
title: stop_scale may make --target-rel inert on the 2→6 rows
description: "Measured under the replaced χ²/dof stop factor, few-accepted-point iterations inflated the stop scale up to ×758 so a 0.2% --target-rel never fired on 2→6; unmeasured under pooled_scale."
area: performance
state: open
priority: medium
closes_when: "The inflation is re-measured under pooled_scale on the 2→6 rows (≥5 seeds); if it persists, stop_scale qualifies few-accepted-point iterations and a 0.2% --target-rel stops with the achieved error matching the seed spread."
blocked_by: []
opened: 2026-08-06
tags: [budget, stop-rule, target-rel, two-to-six, wide-splits, re-measure]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1044-L1058", title: "TODO.md entry T098"}
  - {id: budget, resource: "../../../../vibegraph-lib/src/budget.rs", title: "budget.rs pooled_scale (~:476-510), introduced in 403cff8 (PR #14, 2026-10-04)"}
---
**The numbers below predate the current stop rule** (re-checked 2026-10-09).
They come from [note 34](../../history/notes/34-draw-followup-plan.md) (August 2026) and were
measured under the χ²/dof stop factor. `ChannelHistory::stop_scale`
(`vibegraph-lib/src/budget.rs` ~:472) now calls `pooled_scale`, the
`max(1, emp/quoted)` of the point-weighted mean over kept iterations. That
arrived in 403cff8 (PR #14, 2026-10-04). Its `quoted` still sums each
iteration's own variance floored only at `f64::MIN_POSITIVE`, so the mechanism
may survive. Re-measure first.

As measured then: on 2→6 rows some channel iterations hold one to three
accepted points, so their sample variance is tiny but strictly positive, and
the factor explodes.
- There were no zero-variance (all-points-cut) iterations on any wide-row seed,
  so a fix that filters on `variance == 0` would miss the whole effect.
- `scaled_rel/achieved_rel` spanned ×4.8–×30 270, structured by the α draw.
  After the accepted-point floor it spanned ×3.4–×758: a 1 % target fired on
  some seeds, but a 0.2 % target could not.

If it persists, the fix is in what `stop_scale` does with few-accepted-point
iterations, such as a minimum-accepted-count qualification. It is never a
retuned factor, and the reported statistic stays as it is. The plain quoted
error at the 40k survey cap was well calibrated: achieved_rel 0.0021–0.0024
against a realized sd/σ of 0.0021, and 2× optimistic below the cap.

Detail: [note 34 §2 S1 Part C and S3](../../history/notes/34-draw-followup-plan.md).
Related re-measurements: [sampler-tunings-predate-pooled-stop](sampler-tunings-predate-pooled-stop.md).
