---
type: Backlog Item
title: The point-weighted iteration combine is biased low when counts follow earlier variances
description: "Under Neyman, combine_kept weights iterations by point counts set from earlier per-point sd, which a tail point inflates; bbx reads 0.10–0.14% of σ low (t −2.4, −2.8) and its spread shrinks."
area: validation
state: open
priority: high
closes_when: "Every channel combine whose point counts can vary uses weights fixed independently of the run's outcomes, the stop rule's pooled scale reads the same combination, and the CLI's Neyman target mode and the BlockAllocation::Neyman doc figures are re-measured with it against a paired outcome-independent arm."
blocked_by: []
opened: 2026-10-26
tags: [multichannel, allocation, neyman, iteration-combination, bias, budget-ladder]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: p1, resource: "../../sprints/floor-bound-neyman/sessions/P1-report.md", title: "floor-bound-neyman P1 report, Verdict and Found 1"}
---
`ChannelHistory::combine_kept` (`vibegraph-lib/src/budget.rs`) takes the
plain mean when a channel's kept iterations drew equal counts and the
point-weighted mean otherwise. Its doc argues the counts cannot correlate
with the estimates they weight and quotes "0.26–0.40 σ, no common sign".
P1 measured otherwise on `bbx_to_ccx_emmm_qcd0`, 20 paired seeds:

- Neyman − ByAlpha: −1.71e-9 pb at 600k (t −2.40), −2.41e-9 pb at 1.2M
  (t −2.77). Replaying the same Neyman iterations as plain means leaves
  −0.26e-9 and −0.30e-9 (t −0.33, −0.23).
- The shift sits in channel-runs whose largest kept iteration drew fewer than
  their mean points: a tail point raises the channel's sd, Neyman feeds it
  later, and the later iterations outweigh the tail.
- It also shrinks the per-seed spread and the quoted error (sd ratio 0.65
  as combined, 1.18 as plain means), so a budget ladder reads it as a gain.

The CLI's `--target-rel` mode defaults to Neyman and runs this combine, and
the stop rule's `pooled_scale` uses the same point weights. ByAlpha also
reaches the point-weighted branch wherever a floor re-read from acceptance
changes a floor-bound channel's count between kept iterations; whether the
fixed-budget gates move is part of the work.
