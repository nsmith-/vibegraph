---
type: Design Decision
title: "Tolerances for the matched MLM comparison through Pythia"
description: "Proposed: each per-@N acceptance, the overall acceptance and the merged σ gate at |pull| < 3; the three jet-rate χ² gate on a look-elsewhere p-value, enforced only while MadEvent's own A-vs-A null passes the same rule."
status: draft
tags: [mlm, pythia, matching, gate-promotion, look-elsewhere]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: gate, resource: "../../../validation/mlm-pythia-matched-comparison.md", title: "Matched MLM comparison through Pythia: results, nulls, controls"}
  - {id: d1-scatter, resource: "../../sigma-reference-scatter/decisions/D1-reference-error-design.md", title: "D1, third amendment: the look-elsewhere scatter rule"}
---
**Awaiting the user's decision.** Nothing here is implemented.

## What the comparison measures today

From [the gate concept](../../../validation/mlm-pythia-matched-comparison.md),
21 MadEvent files against 20 vibegraph files, ten Pythia seeds:

| quantity | pull B − A | per-file χ²/dof, A / B |
|---|---|---|
| acceptance `@0` / `@1` / `@2` | −0.80 / −0.95 / −0.98 | 0.63–1.00 / 0.75–0.92 |
| acceptance, all | −0.66 | |
| merged σ | +0.19 | 1.04 / 0.53 |

| jet rate | B vs A | A vs A null (files 1–11 / 12–21) | B vs B null (seeds 28–37 / 38–47) |
|---|---|---|---|
| d01 | 24.5/26, p 0.55 | 17.1/25, p 0.88 | 34.3/25, p 0.10 |
| d12 | 14.4/23, p 0.91 | **35.6/22, p 0.034** | 13.6/22, p 0.92 |
| d23 | 20.7/20, p 0.41 | 18.3/19, p 0.50 | 21.7/19, p 0.30 |

(p-values are χ² survival functions computed from the recorded χ²/dof.) The
per-side errors are honest: files scatter as their errors say on both sides.
The comparison is not vacuous: deleting `<scales>` moves `@1` to +25σ and the
jet-rate χ² to 937/24, and qCut 45 on both sides is followed within 1.3σ.

## Proposal (the planner's, made concrete)

1. **Acceptances and merged σ: |pull| < 3** on each of `@0`, `@1`, `@2`, all,
   and the merged σ. Five correlated tests; if independent, a correct
   comparison fails by chance about 1.3 % of the time (1 − (1 − 0.0027)⁵).
2. **Jet rates: the look-elsewhere rule** already decided for SCATTER.
   The cell fails when min(p_d01, p_d12, p_d23) × 3 < 1 %, i.e. any one DJR at
   p < 0.0033. Today's worst is p 0.41 × 3.
3. **Only relative to the null.** The driver also computes MadEvent's A-vs-A
   split under the same rule. If the null fails, the statistic is not
   calibrated on that bundle: the cell reports `info` with "null failed" and
   does not pass or fail on the jet rates. Today the null's worst is
   d12 p 0.034 × 3 = 0.10, so it passes. The B-vs-B null is recorded beside it
   for information.
4. **Collator row.** `mlm_match.py` writes one validation-report row with the
   five pulls, the three B-vs-A p-values, the null's p-values and the verdict.
   The manifest flips the cell from `info` to `gate` (long tier, never in CI) on
   the first recorded run under the rule.

## What it cannot see

- A shift below the per-side resolution: ±0.3 % on `@1` and ±0.5 % on `@2` at
  1σ, so the gate passes offsets up to about 1–1.5 %.
- Anything the ten Pythia seeds and these 41 files share: the Pythia version,
  `main164`'s settings, the qCut rule.
- One null split is one draw. A failing null blocks the gate rather than
  rescaling it, so no hand-tuned factor enters.

## Options not proposed

- **Gate χ²/dof against a multiple of the null's χ²/dof.** One null draw
  (35.6/22) would set the scale, and the rule would loosen whenever the null
  reads high.
- **|pull| < 3.23 on all eight tests under one look-elsewhere rule.** This is
  uniform with SCATTER but slightly looser on the acceptances than the
  planner's 3.
