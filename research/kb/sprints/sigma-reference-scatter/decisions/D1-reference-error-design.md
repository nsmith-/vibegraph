---
type: Design Decision
title: "MadEvent σ references: independent seeds, recorded iterations, equal-weight recombination"
description: "Proposed: every gated MadEvent σ reference banks fresh-directory seeds at two budgets and keeps each channel's iterations; the gate reads the equal-weight recombination, with the measured seed scatter folded into its error."
status: draft
tags: [madevent-reference, sigma, seeds, iteration-combination, refdata]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: seed-policy, resource: "../../../validation/madevent-reference-seed-policy.md", title: "Seed policy for MadEvent references"}
  - {id: pr22, resource: "https://github.com/nsmith-/vibegraph/pull/22", title: "sigma-2to4-offset: the iteration-weighting bias on a fixed-beam 2 → 4"}
  - {id: pr26, resource: "https://github.com/nsmith-/vibegraph/pull/26", title: "sigma-ee-mumua: the bias that does not shrink with budget"}
  - {id: pr28, resource: "https://github.com/nsmith-/vibegraph/pull/28", title: "sigma-uux-wbwb-veto: the W-pair continuum deficit"}
---
**Awaiting the user's decision.** Nothing here is implemented.

## What this sprint measured

MadEvent combines each channel's VEGAS iterations with weights
mean²/variance (`dsample.f:1995-1997`). An iteration that misses a peak reads
both low and precise, so it gets extra weight. The seed policy recorded the
effect once, on `pp_to_llj` (−0.146 %). This sprint measured it on three more
rows, all pinned 3.7.1 with fresh directories:

| row | reported σ vs an adaptation-free estimate | equal-weight recombination vs the same estimate | shrinks with budget? |
|---|---|---|---|
| fixed-beam `u u~ > e+ e- g g` (#22) | −0.17 % (60 seeds) | +0.02 % | yes |
| `ee_to_mumua` (#26), banked 10k | −0.95 % | −0.55 % | no: −0.94 % at 100k, χ²/dof 3.6 |
| `ee_to_mumua` at 100k | −0.94 % | −0.19 % ± 0.34 % | — |
| `u u~ > w+ b w- b~ / t t~` (#28), fixed μ | −1.9 % (1 seed) | moves up 0.5–1.2 % | no clear trend |

On every row this side agrees with the adaptation-free estimate within its
errors. The reported MadEvent value is the outlier, and its quoted error does
not cover the bias. Seeds sharing a directory hide the bias further, because
they inherit its grid (χ²/dof 0.3–0.8 on the MLM rows).

## Options

- **A (proposed). Recombine, and record the scatter.**
  - Every gated σ reference is banked from ≥ 5 fresh-directory seeds at the
    banked budget plus ≥ 2 at 4×. Each run keeps its `G*/results.dat`.
  - The bundle records, per seed, the reported σ and the equal-weight sum of
    each channel's iterations, and, per row, both values' seed χ²/dof and the
    budget shift.
  - The gate reads the equal-weight mean. Its error is the seed policy's
    max(quoted, spread/√n), widened to the measured budget shift where that
    is larger.
  - A row whose equal-weight mean still moves with budget (`ee_to_mumua` at
    10k) is re-banked at the budget where it stops moving.
  - Cost: a recombination script beside `madevent_seeds.sh`, the bundle
    schema, the σ gates' reader, and an M3 re-bank of every gated row.
- **B. Keep the reported σ and register per-row tolerances.** No re-bank,
  but each row carries a hand-sized allowance for a bias the reference does
  not quote, and a new row's allowance is unknown until someone measures it.
- **C. Raise the MadEvent budgets until the bias is negligible.** It fails on
  `ee_to_mumua`, which does not converge at 10× budget.

A non-adaptive estimator per row, the third opinion three sessions here
wrote, is the strongest reference but is process-specific code. It stays a
diagnostic, not a gate.

## What A cannot see

- A region that every MadEvent iteration misses. Equal weights do not help
  there, and only an independent estimator can find it.
- The `#28` continuum deficit may be partly that.
