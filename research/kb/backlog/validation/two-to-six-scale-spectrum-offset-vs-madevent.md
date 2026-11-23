---
type: Backlog Item
title: "The 2→6 uux samples row's SCALUP and AQCDUP spectra sit off MadEvent's, while every per-event scale given the channel matches"
description: "On uux_to_ccx_emmm_qcd0 the serial and blocked samplers both give SCALUP/AQCDUP KS p down to 1e-5 against MadEvent (D 0.025–0.031) while kinematics pass; the configuration each scale reads is the suspect."
area: validation
state: open
priority: high
closes_when: "The offset is attributed with a per-event measurement on MadEvent's own events (which configuration each scale reads, against the AMP2_c draw), and either fixed so the row's SCALUP and AQCDUP gate at the 1e-4 floor on three seeds, or shown to be MadEvent-side and the columns kept at info with that measurement in the cell note."
blocked_by: []
opened: 2026-11-23
tags: [scales, samples-gate, two-to-six, clustering, configuration-draw, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: channel, resource: "../../scales-pdf/cluster-scale-channel-dependence.md", title: "MadGraph's clustering scale depends on the integration channel; stale isbw on 1.6 % of this row's events"}
  - {id: draw, resource: "../../scales-pdf/clustering-configuration-draw.md", title: "The configuration draw ∝ AMP2_c"}
  - {id: replay, resource: "../../validation/scale-replay-gate.md", title: "Scale replay gate; the 2→6 rows are declined there and enforced against the instrumented dump"}
---
The `uux_to_ccx_emmm_qcd0` samples row ran for the first time in sprint L,
in the pass that added the long-tier sample rows. Its `SCALUP` and
`AQCDUP` columns fail the 1e-4 KS floor against MadEvent's banked sample
on one of three seeds, and read low on all three: p 1.7e-4 to 2.8e-5 on the
serial sampler, 2.9e-3 to 1.2e-5 on the blocked one, at D 0.025–0.031. The
kinematic columns pass on the same events, with worst p 0.03–0.15.
`bbx_to_ccx_emmm_qcd0` passes its scale floor, with lowest p 4.8e-3.

**Not the sampler.** On the same frozen grids and maxima, a probe drove
`Unweighter::trial`, the trial loop main ships, and the blocked parallel
pass. The two passes agree on every column: two-sample p 0.62–1.0 on the
scales, 0.29–0.79 on the worst kinematic column. Both fail against MadEvent.
The patch and per-event tables are on the validation host
(`bench-v0.2.0/pr13-probe-uux/`).

**Not the scale computation.** Given MadEvent's channel and its carried
on-shell flags, `validate_kt_cluster.rs` reproduces all 20 000 of this row's
events: every candidate pair, every merge, both scales.

So the difference has to be **which configuration** each event's scale
reads. Three candidates:

- **The draw rule against MadEvent's realised channels.** vibegraph draws the
  configuration ∝ `AMP2_c`, which is the conditional that single-diagram
  enhancement induces only if MadEvent visits every one of the 579 channels.
  A channel that MadEvent's survey drops or under-produces changes which
  scale its region's events read.
- **Stale `isbw`.** On 163 of the 20 000 MadEvent events (0.8 %), the
  on-shell flags are left over from the previous event. Alone, that is too
  few for a 3 % D, unless their scales move far.
- **The resonance tagging (`checkbw`) of the drawn configuration.** It
  differs from MadEvent's because the configurations differ.

**First step, on data already banked.** The instrumented dump names
MadEvent's channel for each event. For every event:

1. compute vibegraph's scale under every configuration;
2. form the `AMP2_c`-weighted distribution of `SCALUP` that the draw rule
   implies at MadEvent's momenta;
3. compare it with MadEvent's printed `SCALUP`.

Also tabulate MadEvent's realised channel frequencies against the mean
`AMP2_c` share. If the draw-rule distribution reproduces the offset, the
cause is configuration choice and the realised channel table says which side
it sits on. No sampling or seeds are needed.

**What the offset does to σ.** At `QCD=0` no αs enters |M|², so the scale
reaches σ only through μF in the PDFs. The row's σ cell is informational for
its estimator tail, so it cannot bound this. Clustered cards that do carry αs
in |M|² would move.

Until this closes, the row's `SCALUP` and `AQCDUP` sample columns are
reported at info, citing this item.
