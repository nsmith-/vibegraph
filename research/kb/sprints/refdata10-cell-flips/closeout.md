---
type: Sprint Record
title: "Cells refdata-10 settles: ttx MLM @1 and gg_to_gg_cg"
description: "pp_to_ttx_0j1j_mlm's and gg_to_gg_cg's integrals cells agree with their refdata-10 seeded hard_survey = 2 references and now gate; no tolerance moved."
closed: 2026-10-17
status: draft
tags: [sigma, mlm, ttx, smeftsim, gate-promotion, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v1, resource: "sessions/V1-report.md", title: "V1 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/44", title: "nsmith-/vibegraph#44"}
---
One PR ([nsmith-/vibegraph#44](https://github.com/nsmith-/vibegraph/pull/44))
closing `ttx-mlm-at1-sigma-high` and `gg-to-gg-cg-integrals-cell-still-info`.
Session: [V1](sessions/V1-report.md), validation-dev on Opus.

## What was established

- **`pp_to_ttx_0j1j_mlm`.** Ten seeds on this side, read against refdata-10's
  five seeds:

  | part | vibegraph | MadEvent | pull |
  |---|---|---|---|
  | `@0` | 513.219 ± 0.179 | 513.106 ± 0.222 | +0.40 |
  | `@1` | 578.780 ± 0.327 | 578.337 ± 1.128 | +0.38 |
  | total | 1091.999 ± 0.421 | 1091.440 ± 1.149 | +0.46 |

  This side's `@1` matches `sigma-ttx-mlm-at1`'s post-fix reading to every
  printed digit. The cell gates on the existing |pull| < 3 and χ²/dof band;
  the table is in [the MLM σ gate](../../validation/mlm-sigma-gate.md).
- **`gg_to_gg_cg`.** The gate seed reads 2.707089e5 ± 330.6 pb, pull +0.15,
  rel +1.9e-4. Five seeds read pull +0.89 and rel +5.6e-4 (χ²/dof 0.88); ten
  read +1.02 and +5.1e-4. The cell gates as `Plan::Gate { 40 000, 6,
  rel_tol 0.005 }`, the SMEFTsim rows' convention, 2.2× outside the worst
  single seed (|rel| 2.29e-3).
- **`Plan::Info` has no row left.** It is kept, as the demotion rung, under a
  `#[allow(dead_code)]`.

## Trust

Machine-confirmed by the manager: both references recomputed from the seed
records (ttx total 1091.4400 ± 1.1491, χ²/dof 0.92; `gg_to_gg_cg`
270658.0 ± 88.06, χ²/dof 1.15); the ttx gate log (three `gate: pass`,
`TTX_EXIT 0`); the commits. The `gg_to_gg_cg` seed readings and the validate
run are the session's.

## Left open

- **The merge with PR #42.** This PR edits `validate_sigma.rs` on a base
  without #42's five-seed gate. The `#[allow(dead_code)]` sits a few lines
  from #42's seed constants above `enum Plan`, and whichever merges second
  resolves it.
- **A stale module doc.** `validate_sigma.rs` lines 72–76 still say
  `gg_to_gg_cg` is "not integrated at all". That is left to the calibration
  re-record (PR #45), which edits the same file's comments.

## Filed

- [ttx-mlm-at0-error-understated](../../backlog/validation/ttx-mlm-at0-error-understated.md)
