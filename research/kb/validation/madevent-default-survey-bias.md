---
type: Measurement
title: "MadEvent's default survey under-integrates g q > t t~ q"
description: "At the default survey MadEvent 3.7.1 reads unmatched g q > t t~ q 0.55 % low, wholly in channel G2, with seed χ²/dof 7.1; hard_survey = 2 removes it, while 10× events, splitting the folded configuration or sde_strategy = 2 do not."
status: draft
tags: [madevent-reference, survey, sigma, ttx]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-10-15"
  command: "p p > t t~ j with quark-only j, 13 TeV, lhaid 247000, ickkw = 0, xqcut = 0, ptj = 30, fixed μ = 173; MadEvent pinned 3.7.1 via mg5_pinned.sh, one fresh directory per seed, nb_core 2; an adaptation-free oracle on MadGraph's standalone SMATRIX"
sources:
  - {id: v4, resource: "../sprints/sigma-ttx-mlm-at1/sessions/V4-report.md", title: "sigma-ttx-mlm-at1 V4 report"}
---
# MadEvent's default survey under-integrates `g q > t t~ q`

| estimator | σ (pb) | seeds | χ²/dof |
|---|---|---|---|
| adaptation-free oracle (two independent channelings) | 137.928 ± 0.067 | 20 | 0.70 / 1.23 |
| vibegraph | 137.903 ± 0.060 | 5 | |
| MadEvent, default survey, reported | 137.124 ± 0.211 | 10 | 4.67 |
| MadEvent, default survey, equal-weight iterations | 137.177 ± 0.197 | 10 | |
| MadEvent, `hard_survey = 2` | 137.887 ± 0.125 | 10 | 1.17 |

The whole deficit is in channel `P1_gq_ttxq/G2`, 61 % of σ. It is a t-channel
gluon off the incoming quark, with `g g* → t t̄` through a t-channel top.
Within one seed, G2's iterations agree with one another, but the seeds
disagree (χ²/dof 7.1). Each seed's survey grid carries an offset that its
quoted error does not contain, and more events do not repair it.

On the matched `pp_to_ttx_0j1j_mlm` row, 20 fresh seeds with `hard_survey = 2`
read `@1` 578.239 ± 0.520 pb. That is this side's 578.780 within 0.88σ, and
gg and gq each sit within 1σ.

**Consequence.** A MadEvent reference banked at the default survey can sit
0.5–1 % low on a process whose dominant channel's survey grid misses part of
it, and its quoted error and budget ladder both hide it. A seed χ²/dof ≫ 1 is
the visible sign. This is distinct from the iteration-combination bias
(sibling PR #22), which also reads low but closes with equal weights.

The machine-confirmed check covers the commit and the oracle's and
`hard_survey = 2` seeds, recomputed by the manager from the raw lines.
