---
type: Sprint Record
title: "u u~ > w+ b w- b~ $ t t~ offset"
description: "The ~2 % gap on the on-shell-veto row is MadEvent under-integrating the top-free continuum (~94 % of σ) in channels G7–G9: −2.0 % over six fresh seeds against vibegraph and a non-adaptive estimator."
closed: 2026-10-15
status: draft
tags: [onshell-veto, madevent-reference, sigma, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: v1, resource: "sessions/V1-report.md", title: "V1 report"}
  - {id: v2, resource: "sessions/V2-report.md", title: "V2 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/28", title: "nsmith-/vibegraph#28"}
---
One PR ([nsmith-/vibegraph#28](https://github.com/nsmith-/vibegraph/pull/28))
closing `uux-wbwb-onshell-veto-tt-2pct-off`. Chronology in [log.md](log.md).

## What was established

- **Not the veto, the ALOHA patch, the top windows or H1.** H1 is bounded at
  ≤ 0.28 % by `sigma-h1-ickkw0-census`. Every top-window slice is low on
  MadEvent's side.
- **The top-free continuum carries it.** For `u u~ > w+ b w- b~ / t t~`, about
  94 % of the row:

  | estimator | σ (pb) | seeds |
  |---|---|---|
  | vibegraph | 0.0458886 ± 0.000022 | 5 |
  | MadEvent, reported | 0.044956 ± 0.00011 (χ²/dof 4.4) | 5 fresh, plus V1's one |
  | MadEvent, equal-weight iterations | 0.045306 ± 0.00011 | 5 |

  At fixed μ, a non-adaptive three-channel sampler reads 0.054842 ± 0.000051.
  vibegraph is within 0.27 % of it, MadEvent 2.3 % below.
- **Where MadEvent loses it.** Channels G7–G9 carry 72 % of σ and the whole
  deficit. In G8/G9, iterations that sample large weights alternate with ones
  that miss them, and the mean²/variance combination favours the misses.
  G7's grid never samples the region. The both-W |cos θ| > 0.99 slice is not
  where σ is lost. The region inside the complement is not localised.
- **The backlog item's "MadEvent drifts down with budget"** did not
  reproduce.

The row stays reported, not gated, and is now documented as a measured
MadEvent defect. Whether such rows ever gate against MadEvent's σ is the
reference-error decision's question (D1 on PR #25,
`sprints/sigma-reference-scatter/decisions/D1-reference-error-design.md`).

## Trust

V1 and V2 are machine-confirmed by the manager: the third-opinion mean and
the vibegraph and MadEvent seeds, recomputed from the raw result lines. The
per-channel analysis is the sessions'.

## Filed

- [vegas-under-covers-heavily-cut-slice](../../backlog/validation/vegas-under-covers-heavily-cut-slice.md)
  (this side, medium)
- [vegas-printed-chi2-overflows](../../backlog/validation/vegas-printed-chi2-overflows.md)
- [madevent-mirror-pair-consistency](../../backlog/validation/madevent-mirror-pair-consistency.md)
