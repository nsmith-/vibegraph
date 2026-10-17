---
type: Design Decision
title: "Fixed-beam σ rows gate on five seeds; equal-weight reads stay where validated"
description: "Decided: the 34 fixed-beam σ rows read this side over five seeds, as the hadronic and grammar rows do; the equal-weight reference is read on no new row unless an adaptation-free 2 → 3 estimate on uux_to_epemg lands on it; the item closes on V1's attribution."
decided: 2026-10-17
decided_by: human:nsmith-
status: draft
tags: [sigma-gate, seeds, madevent-reference, iteration-combination]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v1, resource: "../sessions/V1-report.md", title: "V1 report: the σ pull population against the seed records"}
---
**Approved by the user, relayed by the planner on 2026-10-17.**

[V1](../sessions/V1-report.md) attributed the +0.615 mean pull over 48
seed-record rows. About 0.14 is this side's one gate seed, which all 34
fixed-beam rows share. About 0.19 is MadEvent's mean²/variance iteration
weighting on four multi-leg fixed-beam rows. About 0.18 moves with MadEvent's
budget, at screen grade. The rest is consistent with zero, except `pp_to_bb`.

1. **Five seeds on this side for the 34 fixed-beam σ rows.** Each row's gate
   reads the unweighted mean of five seeds (`HEADROOM_SEEDS`, the gate's seed
   first), with the seed policy's error. Its pull is measured against the
   row's seed record, as the hadronic and grammar rows already are. Limits do
   not move. The suite goes from about 275 s to about 753 s on a 4-core host.
   This lands before the σ calibrations are re-recorded
   (`sigma-calibration-rerecord`, PR #45).
2. **No equal-weight reading on more rows.** On rows whose EW − R is not
   resolved it moves the mean by only −0.06, and it is heavy-tailed on 7 rows.
   `validated_ew` is set on `uux_to_epemg`, `ddx_to_epemg` and
   `ee_to_mumu_tata_qcd0` only if an adaptation-free 2 → 3 estimate on
   `uux_to_epemg` lands on EW rather than on MadEvent's reported value. That
   estimate runs in this sprint (V2), with `ddx_to_epemg` as control.
3. **`sigma-pulls-lean-high-on-seeded-refs` closes on V1's attribution.** The
   `pp_to_bb` residual is filed as its own item, waiting on the 20-seed banking
   request posted on PR #42.
