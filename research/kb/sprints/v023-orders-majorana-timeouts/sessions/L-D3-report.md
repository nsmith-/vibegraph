---
type: Session Report
title: "L-D3 report: the DarkHiggs deficit is MadGraph's pre-3.7 sde_strategy = 2 channel weight"
description: "Card 880 reads σ 0.976× its MG 2.9.18 gridpack. The matrix element and width agree, and the m(dm dm hs) shapes match. μF differs: MadGraph clusters 83 % of events at mT(dm dm), vibegraph 38 %. The cause is get_channel_cut, which before 3.7 builds every propagator factor from q⁴ − M². Both programs realise CC_c/ΣCC under their own formula: 0.814 predicted against 0.827 realised for the old form, checked inside an instrumented MadGraph 3.5.7 to 2.9e-6. vibegraph follows 3.7.1. Diagnosis only, with no code change."
status: draft
tags: [darkhiggs, scales, kt-clustering, configuration-draw, sde-strategy, madgraph-versions, cms-gridpacks, report, validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-27}
---
Physics session of sprint L, run by the physics manager itself with no dev agent.
This was a diagnosis: the only commit is this report and the concept it records.
Code was read at `59045b57` (after the history rewrite; its tree equals the old
`c18d4304`). Scratch work is under `/home/user/wt/darkhiggs/` on the physics host:
`tools/` (scripts), `data/` (the validator's 61-event extract) and `mg357/`
(an instrumented MadGraph 3.5.7 output). The brief closes no backlog item.

## Summary

- **The order of checks was the planner's:** first the per-event m(dm dm hs), then
  the width, then a bit-exact comparison. The validator ran step 1 on 880, with
  card 158 as the control. The shapes agree in every bin within statistics.
  `DECAY 55` agrees, at 126.9491. Neither side applies a Breit-Wigner window,
  because a required s-channel only filters diagrams, on both sides. The
  matrix-element comparison was parked.
- **The difference is μF.** MadGraph puts 82.8 % of events at μF = mT(dm dm) and
  17.2 % at √ŝ. vibegraph puts 37.8 % and 62.2 %. On 158 both sit at 99 %
  mT(dm dm). The validator's PDF-ratio estimate gives ×1.017, about 70 % of the
  deficit.
- **The configuration draw is the same rule on both sides.** Each event's
  clustering configuration is distributed as `CC_c/ΣCC`, as
  `clustering-configuration-draw` says. Evaluated on 61 of the gridpack's own
  events with all five configurations, the pre-3.7 form predicts
  P(c4 + c5) = 0.814, against a channel-σ share of 0.827. The 3.7.1 form predicts
  a Zp → dm dm line on 0.42 of events, against vibegraph's 0.378.
- **The formula differs by version.** MadGraph 2.9.18 through 3.6 compute
  `(t−M)(t+M)` with `t = q²`, so the weight is `q⁴ − M²`. 3.7.1 corrects this to
  `t − M²`, and vibegraph transcribes 3.7.1.
- **Checked inside MadGraph.** An instrumented 3.5.7 output of the same card
  reproduces the gridpack's channel split. Its dumped `AMP2` matches my
  transcription of the old form to 2.9e-6, over 12 000 points × 5 configurations.
- **Not a regression.** `vibegraph-lib/src/coupling/` and `select.rs` are
  unchanged since v0.2.2, by tree diff. The only run-card default change since
  then (#168) is for squared-order cards. The planner moved DarkHiggs to the
  release notes as a known difference.

The full account, with both tables and the version citations, is the concept
[channel-cut-madgraph-versions](../../../scales-pdf/channel-cut-madgraph-versions.md).

## Corrections made along the way

- **A first table used three configurations.** MadGraph's `configs.inc` has five.
  c4 and c5 are the t- and u-channel Yukawa configurations, and they carry most
  of the old-form weight.
- **A first "2.9 form" was transcribed with `q² − M²`.** Reading the per-line
  dump showed the compiled code uses `q⁴ − M²`. Only that reading reproduces
  MadGraph's `AMP2`.
- **The window hypothesis came first and was dropped on a code read.** A required
  s-channel carries no window on either side (`diagrams/check.rs`; gForceBW = 0).

## Measurements and how they were taken

| quantity | value | how |
|---|---|---|
| m(dm dm hs) shape, 20 bins | agree within statistics | validator, 20k events each side |
| μF classification | MadGraph 17.2 / 82.8 %, vibegraph 62.2 / 37.8 % | validator, SCALUP matched to √ŝ or mT(dm dm) |
| gridpack channel σ shares | c1 0.04 %, c2 + c3 17.2 %, c4 + c5 82.7 % | validator, `G*/results.dat` |
| P(c) on 61 events, old form | 0.000 / 0.186 / 0.814 | `tools/cfgw5_bug.py` |
| P(c) on 61 events, 3.7.1 form | 0.075 / 0.579 / 0.345 | `tools/cfgw5.py` |
| transcription against MadGraph AMP2 | worst 2.9e-6 over 12 000 points | `tools/cmpdump.py` on `mg357/.../G*/vgdump.txt` |
| 3.5.7 survey, `u u~` | G4 5.56e-4, G5 5.52e-4, G2 2.34e-4, G1.x ~5e-7 pb | local `generate_events`, nn23lo1 |

The MadGraph 3.5.7 run needed two comma-less `$)` Fortran formats patched for the
container's gfortran, in `symmetry.f` and `reweight.f`. Both are output formatting
only. The instrumentation sits in the process directory's `genps.f` and
`matrix1_{orig,optim}.f` and template, and none of it is committed.

## Found

- **Pre-3.7 references.** Any reference made with MadGraph before 3.7 at
  `sde_strategy = 2` with a clustered scale can differ in the same way. Bounding
  how many CMS cards that touches is the validator's gpdb query.
- **Emulation is an open design question for the user.** A version switch on
  `ChannelSet::channel_cuts` would reproduce the pre-3.7 weight, which is an
  upstream defect. The recommendation is not to emulate it by default.
- **The 3.5.7 conda package fails `gensym` on a recent gfortran.** It fails on
  `(F3.1$)` with "Missing comma between descriptors". This matters to anyone
  running the `madgraph` pixi environment on Linux.
