---
type: Sprint Record
title: "t t~ MLM @1 σ excess"
description: "The +1.28 % @1 excess was two things: this side clustering MLM scales in the lab frame (fixed, −4.47 pb) and MadEvent's default survey under-integrating g q > t t~ q; against a hard_survey = 2 reference @1 reads 0.88σ."
closed: 2026-10-15
status: draft
tags: [mlm, sigma, ttx, clustering, madevent-reference, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: v1, resource: "sessions/V1-report.md", title: "V1"}
  - {id: v2, resource: "sessions/V2-report.md", title: "V2"}
  - {id: v3, resource: "sessions/V3-report.md", title: "V3"}
  - {id: v4, resource: "sessions/V4-report.md", title: "V4"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/23", title: "nsmith-/vibegraph#23"}
---
One PR ([nsmith-/vibegraph#23](https://github.com/nsmith-/vibegraph/pull/23)).
Chronology in [log.md](log.md).

## What was established

1. **Not the reference draw, not H1 (V1).** 20 fresh MadEvent directories
   reproduce the bank. H1 is +0.03 ± 0.11 pb on this row.
2. **This side clustered in the lab frame (V2, F1).** `cluster.f` breaks an
   initial-state beam tie on the sign of p_z, and the massive-pair measure reads
   pseudorapidities. Neither is boost-invariant, and MadEvent clusters its
   partonic-CM `PP`. In the CM, 0 of about 26 000 points differ from an
   instrumented `DSIG1`, factor by factor. In the lab, 3330 of 20 000 differ.
   The fix (`6740954`) moved `@1` by −4.465 ± 0.083 pb (V3, paired). It also
   resolved `pp_to_jj`'s nine "tie-break" replay misses.
3. **MadEvent's default survey (V4).** The residual is MadEvent
   under-integrating `g q > t t~ q` in one channel; `hard_survey = 2` removes
   it ([measurement](../../validation/madevent-default-survey-bias.md)).
   Against a `hard_survey = 2` reference, `@1` reads 578.239 ± 0.520 against
   578.780 ± 0.327 (pull 0.88).
4. **|M|²** was ruled out independently by the parallel sprint (PR #29).

## What landed

- `6740954`: CM-frame clustering in `ProtonIntegrand`; the CM replay in
  `validate_scales` with zero allowed misses; the test
  `clustered_scales_are_read_in_the_partonic_centre_of_mass`.
- `2dd52f4`: the frame each scale path reads, documented (HY1).
- Gates on the fix: fmt, clippy, `cargo test --workspace`, and
  `--skip-deps validate` all green (F1, HY1). HY1's mutations: 8/8 fail.

## Left open

`ttx-mlm-at1-sigma-high`, rewritten: the cell gates once the reference is
re-banked with `hard_survey = 2` (PR #25's batch). Its claim is released.

## Trust

Every session report is machine-confirmed by the manager, with the checks in
[log.md](log.md). The headline numbers were recomputed from raw files: the
paired −4.465 shift, the per-point mismatch counts, the oracle and the
`hard_survey = 2` means.

## Filed and closed

- Closed: `pp-to-jj-tie-break-no-cluster-dump`.
- Filed:
  - [closed-form-scale-choice-1-lab-frame](../../backlog/validation/closed-form-scale-choice-1-lab-frame.md)
  - [generate-refuses-quark-only-jet-ttx](../../backlog/feature/generate-refuses-quark-only-jet-ttx.md)
  - [integrand-path-unchecked-per-event](../../backlog/validation/integrand-path-unchecked-per-event.md)
- Appended to `kinematic-helpers-hand-rolled` (the test-side boost
  duplicate).
