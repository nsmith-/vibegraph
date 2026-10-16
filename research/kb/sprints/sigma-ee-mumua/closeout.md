---
type: Sprint Record
title: "ee → μμγ radiative-return excess"
description: "The +1.04 % excess was MadEvent's single default-survey bank under-weighting rare large Z-ISR weights; against the seeded hard_survey = 2 reference the row reads −0.074 % and its pull is asserted."
closed: 2026-10-16
status: draft
tags: [ee-to-mumua, sigma, madevent-reference, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: v1, resource: "sessions/V1-report.md", title: "V1 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/26", title: "nsmith-/vibegraph#26"}
---
One PR ([nsmith-/vibegraph#26](https://github.com/nsmith-/vibegraph/pull/26))
closing `ee-mumua-radiative-return-sigma-high`. Chronology in [log.md](log.md).

## What was established

| estimate | σ (pb) | vs fixed-density |
|---|---|---|
| bank: one MadEvent run, 10k events, default survey | 9.98009e-2 | −1.00 % |
| fixed-density (adaptation-free) estimator | 1.00806e-1 ± 1.1e-5 | — |
| refdata-10: 5 seeds of 100k-event `m(μμ)` window sums, `hard_survey = 2`, reported | 1.007732e-1 ± 1.9e-5 | −0.033 % |
| this side, gate seed | 0.100699 ± 0.000177 | −0.11 % |

- **The excess was the reference's.** MadEvent's Z radiative-return channels
  G6 and G8 carry rare large weights. Its mean²/variance iteration combination
  discounts them, and its default survey under-samples them; at 10k events
  equal-weight recombination closes about 40 % of the gap, and at 100k it
  closes all of it. A fixed-density estimator, which never adapts, agrees with
  this side in total, in both radiative-return windows and in the named 2D
  cell.
- **The row is an ordinary gate.** Against the seeded reference it reads
  −0.074 %, pull −0.42, and its pull is asserted;
  `PULL_REPORTED_NOT_ASSERTED` is empty.

## Trust

V1 is machine-confirmed by the manager (the estimator's per-seed totals, the
bank's value). The refdata-10 reference and its records are the user's M3
bank, rescreened by the manager; the −0.074 % reading is CI's banked run on
PR #25 and this PR's local gate run.

## Filed

- [ee-mumua-sample-reference-mis-shaped](../../backlog/validation/ee-mumua-sample-reference-mis-shaped.md)
- [vg-part-probe-sizes-point-by-grid-ndim](../../backlog/hygiene/vg-part-probe-sizes-point-by-grid-ndim.md)

V1's other findings are covered elsewhere: the χ²/dof overflow on a narrow
cut cell by `vegas-chi2-per-dof-overflows-on-wide-splits`, and the
equal-weight reference statistic by D1 on PR #25 and
`sigma-pulls-lean-high-on-seeded-refs`.
