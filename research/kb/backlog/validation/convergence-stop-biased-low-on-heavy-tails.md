---
type: Backlog Item
title: The convergence stop is biased low when a channel's weights have no finite variance
description: "With a Hill tail index below 2 the stop ends on iterations that missed the tail, so early-stopped runs read low; the scale factor, built from iteration scatter, cannot see it. Latent on main."
area: validation
state: open
priority: low
closes_when: "A stopping rule, or a print-only warning, that sees a point-level heavy tail is chosen and measured: its hit rate on a tree where the bias shows (the V2-era tree's both-W |cos θ| > 0.99 slice, ≥ 10 seeds) and its false-positive rate on every banked σ row and main's slice, with its cost in the reduction; or a decision that no change is needed, recorded with that evidence."
blocked_by: []
opened: 2026-11-04
tags: [vegas, convergence, heavy-tail, statistics, sprint-g]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: g-v2, resource: "../../sprints/v02-diagnoses/sessions/G-V2-report.md", title: "Sprint G, G-V2 report: mechanism, seed tables and the Hill-index proposal"}
---
G-V2 diagnosed the −3 % reading of `u u~ > w+ b w- b~ / t t~` at both W's
|cos θ| > 0.99 ([report](../../sprints/v02-diagnoses/sessions/G-V2-report.md)).
On main the slice is correct at the default budget (30 seeds, −0.07 % and
+0.07 %, χ²/dof ≈ 1, against a 1e6 reference and a frozen-grid estimate).

The mechanism that made it low on the older tree is still in the stop:

- no channel maps the forward W emission, so the weights' tail index fell
  below 2 (no finite variance);
- most seeds then drew too few tail points, and the convergence stop ended on
  calm iterations: stopping iteration and σ correlate at Pearson 0.80, the
  early stoppers reading −2.2 % to −3.4 %;
- the stop's scale factor comes from iteration scatter, which is calm in
  exactly those runs.

The bin-width bound (#48) and the peripheral spine (#81) removed the tail on
this slice, so the bias is latent: main's slice reads a tail index of 2.08 to
2.4, the banked rows 2.24 at least.

**Proposal measured, not landed** (user, 2026-11-04): a per-channel median
Hill index below 2.0 on a channel carrying ≥ 2 % of σ flags 16 of 17 at-risk
runs and 0 of 330 correct ones. The margin to main is thin, only one process
has shown the failure, the estimator's k was chosen on it, and it needs a
top-k selection in the reduction. The scripts are in the report.
