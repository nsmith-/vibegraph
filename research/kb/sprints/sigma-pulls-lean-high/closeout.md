---
type: Sprint Record
title: "σ pulls lean high on the seeded references"
description: "The +0.615 mean pull over 48 seed-record rows splits into this side's one shared gate seed (now five seeds), MadEvent's iteration weighting (an adaptation-free 2 → 3 estimate lands on its equal-weight value) and a budget-dependent part at screen grade; this side has no common offset, and the pp_to_bb rows are the one open residual."
closed: 2026-10-17
status: draft
tags: [sigma, madevent-reference, seeds, iteration-combination, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v1, resource: "sessions/V1-report.md", title: "V1 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: v2, resource: "sessions/V2-report.md", title: "V2 report"}
  - {id: d1, resource: "decisions/D1-fixed-beam-seeds-and-ew.md", title: "D1"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/42", title: "nsmith-/vibegraph#42"}
---
One PR ([nsmith-/vibegraph#42](https://github.com/nsmith-/vibegraph/pull/42))
closing `sigma-pulls-lean-high-on-seeded-refs`. It ran four sessions: V1,
HY1, F1 and V2. The decision is [D1](decisions/D1-fixed-beam-seeds-and-ew.md),
approved by the user.

## What was established

- **The attribution.** [V1](sessions/V1-report.md) read 48 seed-record rows
  with a committed tool, `sigma-pulls`. Each step removes one cause:

  | reading | mean pull | cause removed |
  |---|---|---|
  | as gated, one seed | +0.615 ± 0.134 | — |
  | this side over five seeds | +0.475 | about 0.14, this side's one shared gate seed |
  | against the equal-weight seed mean | +0.285 | about 0.19, MadEvent's mean²/variance weighting |
  | against the equal-weight mean of the 4× seeds | +0.110 ± 0.140 | about 0.18, a part that moves with MadEvent's budget (screen grade) |

  Read weighted in relative units, this side's common shift is
  +0.0034 % ± 0.0050 %. The one residual is the `pp_to_bb` family.
- **Five seeds on the fixed-beam σ gate.** [F1](sessions/F1-report.md) moved
  the 34 gated rows from one shared seed to five. All pass with no limit
  moved, at a gate mean pull of +0.410 over the 34 rows and +0.475 over the
  48-row population, which is V1's prediction. The headroom probe reads the
  gate's own reference. See [sigma-gate](../../validation/sigma-gate.md).
- **An adaptation-free 2 → 3 estimate.** [V2](sessions/V2-report.md) built
  `probe_fixed_density_llj_annihilation`, a fixed density with no iteration
  weighting, over 20 × 3M points.
  - `uux_to_epemg`: 0.5564310 ± 0.0000915 pb, pull +3.01 against MadEvent's
    reported mean, +0.45 against its equal-weight mean.
  - `ddx_to_epemg`: 0.6203869 ± 0.0001118 pb, pulls +2.67 and −0.19.

  MadEvent's reported value reads 0.20–0.25 % low on both. This is the 2 → 3
  case of the
  [iteration-weighting bias](../../validation/madevent-iteration-weighting-bias.md),
  whose page holds only the 2 → 4 case. D1's condition for `validated_ew` is
  met.
- **The tool.** `sigma-pulls` joins each row file to its record by value and
  fails loudly on a dropped row. [HY1](sessions/HY1-report.md) raised its
  tests from 7 to 20, which catch 35 of 36 one-line mutations.

## Trust

Machine-confirmed by the manager:
- V1's population, re-run from the committed tool on V1's recorded runs:
  +0.615, +0.475, +0.285, +0.110;
- HY1's output identity, and its `MATCH_REL` mutation;
- F1's one-seed mutation, and `validate_pdf_grid` 20/20 once the manager had
  fetched the missing multigrid PDF set;
- V2's run log, which matches its report;
- every session's commits.

The gate runs, the sweeps and the estimator's calibration runs are the
sessions' own. The banked-layer evidence is this PR's CI.

## Left open

- **Banking request on this PR**, one comment with three items: a 20 + 20
  ladder on `uux/ddx_to_epemg`; 20 seeds on `pp_to_bb` and `pp_to_bb_fixed`;
  and the `validated_ew` re-record of three rows.
- **The 2 → 3 row for the bias page.** It belongs on
  `madevent-iteration-weighting-bias.md`, a MadGraph-defect page a parallel
  drafting session owns. It is passed to the planner instead of being edited
  here.
- **`PULL_LIMIT`'s rationale.** It argues 3.5 as a bound on the worst of five
  per-seed pulls; the gate now asserts the mean's pull. The limit is
  unchanged; restating it belongs to `sigma-calibration-rerecord` (PR #45).
- **Single-seed rows.** `gg_to_gg_cg` and the two 2 → 6 rows still read one
  seed. They belong to PRs #44 and #47.

## Filed

- [pp-to-bb-sigma-high-on-seeded-refs](../../backlog/validation/pp-to-bb-sigma-high-on-seeded-refs.md)
- [validated-ew-awaits-refdata-11](../../backlog/validation/validated-ew-awaits-refdata-11.md)
- [sigma-pulls-tool-follow-ups](../../backlog/hygiene/sigma-pulls-tool-follow-ups.md)
- [validate-sh-stops-at-first-failed-suite](../../backlog/hygiene/validate-sh-stops-at-first-failed-suite.md)
- [sigma-docs-stale-costs-and-report-column](../../backlog/hygiene/sigma-docs-stale-costs-and-report-column.md)
- [probe-scatter-reads-low-pp-to-bb-qcd2](../../backlog/validation/probe-scatter-reads-low-pp-to-bb-qcd2.md)
- [ll-0j2j-mlm-sigma-gate-cost](../../backlog/performance/ll-0j2j-mlm-sigma-gate-cost.md)
