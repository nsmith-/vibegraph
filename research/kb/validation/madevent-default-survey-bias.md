---
type: Measurement
title: "MadEvent's default survey settings bias σ low"
description: "MadEvent 3.7.1's default survey can leave a channel's grid short, biasing σ low by 0.5–1 % (g q > t t~ q: −0.55 % in G2); hard_survey = 2 removes it, but is inert when mmjj > 1 % of ebeam1 + ebeam2, and does not fix rare-large-weight channels."
status: draft
tags: [madevent-reference, survey, sigma, ttx, madgraph-defect, approximation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-10-15"
  command: "p p > t t~ j with quark-only j, 13 TeV, lhaid 247000, ickkw = 0, xqcut = 0, ptj = 30, fixed μ = 173; MadEvent pinned 3.7.1 via mg5_pinned.sh, one fresh directory per seed, nb_core 2; an adaptation-free oracle on MadGraph's standalone SMATRIX"
sources:
  - {id: v4, resource: "../sprints/sigma-ttx-mlm-at1/sessions/V4-report.md", title: "sigma-ttx-mlm-at1 V4 report"}
  - {id: hs2, resource: "https://github.com/nsmith-/vibegraph/pull/25", title: "hard_survey = 2 cross-check on the sibling rows (PR #25 D1, second amendment)"}
  - {id: mg-interface, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/madevent_interface.py#L3526-L3529", title: "madevent_interface.py: mmjj forces the difficult-integration mode before hard_survey is read"}
---
# MadEvent's default survey settings bias σ low

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

## Across rows

The same test was run on the rows diagnosed in this sprint: pinned 3.7.1,
fresh directories, each compared with an adaptation-free or vibegraph value.

| row | default, reported | `hard_survey = 2`, reported |
|---|---|---|
| unmatched `g q > t t~ q` | −0.58 % | −0.03 % |
| `pp_to_llj_xqcut_only` (20 seeds) | +0.08 %; equal-weight −0.26 % | −0.005 % |
| `ee_to_mumua` (39 seeds) | −0.95 % | −0.66 % (−14σ) |
| top-free `u u~ > w+ b w- b~ / t t~` (20 seeds) | −2.03 % | −0.68 % (−6σ) |

- **What `hard_survey = 2` fixes.** It closes the rows where MadEvent missed
  through its survey grid or through a long refine chain. The run-time cost is
  1.05–1.09× at 10k events, and 1.2–1.5× at 2000 events, where the survey is a
  larger share of the run.
- **What it does not fix.** It does not repair channels with rare, very large
  weights (`ee_to_mumua`'s G6/G8, the W-pair continuum's G8/G9). Those also
  carry the iteration-combination bias, recorded on PR #22's branch as
  `validation/madevent-iteration-weighting-bias.md`.
- **Where it is inert.** At the pin, `madevent_interface.py:3526-3529` checks
  `mmjj > 0.01 · (ebeam1 + ebeam2)` first. If that holds, it enters the
  difficult-integration mode at rate 1 and never reads `hard_survey`. A card
  with that `mmjj` cut gets no benefit from the setting, and nothing says
  so. The effective `min_events`/`max_events` in each run's
  `Source/run_config.inc` show which mode ran: 10000/30000 at
  `hard_survey = 2`, 7500/20000 at rate 1, 1000/2000 at the default. It is
  only partly inert: `gen_ximprove.py:114` still sets `combining_job = 1`
  from `hard_survey > 1`, so the refine's job layout changes.
- **A reused directory keeps its first rate.** The difficult-integration mode
  rewrites `run_config.inc` by substituting the pattern
  `\(min_events = \d+\)` with `(min_events = %i )`
  (`madevent_interface.py:3593`). Its own output has a space before
  `)`, so the pattern never matches again: a directory that ran once in that
  mode keeps that rate for every later run, whatever its `hard_survey`. Fresh
  copies avoid it, and the recorded `run_config.inc` exposes it.

**Consequence.** A MadEvent reference banked at the default survey can sit
0.5–1 % low on a process whose dominant channel's survey grid misses part of
it, and its quoted error and budget ladder both hide it. A seed χ²/dof ≫ 1 is
the visible sign. This is distinct from the iteration-combination bias
(sibling PR #22), which also reads low but closes with equal weights.

**Policy.** This is a MadGraph approximation, a survey sized for speed, under
[madgraph-defect-policy](madgraph-defect-policy.md). References are banked with
`hard_survey = 2` in fresh directories, each run's effective
`min_events`/`max_events` is recorded, and a row whose channels still scatter
beyond their quoted errors keeps its σ cell `info` (D1 on PR #25). It is a candidate for an upstream report
([madgraph-defect-reports-unfiled](../backlog/validation/madgraph-defect-reports-unfiled.md)).

The machine-confirmed check covers the commit and the oracle's and
`hard_survey = 2` seeds, recomputed by the manager from the raw lines.
