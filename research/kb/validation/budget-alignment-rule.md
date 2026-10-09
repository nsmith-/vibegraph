---
type: Design Decision
title: Validation budgets matched to reference precision
description: "Size each sigma gate so our error is about MadGraph's, subject to seed-scatter and convergence floors that always win."
status: draft
tags: [validation, sigma, budget, vegas, seeds]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n32-premise, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/32-perf-addendum-plan.md#L48-L74", title: "Note 32 §0 — the budget-alignment argument"}
  - {id: n32-s6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/32-perf-addendum-plan.md#L354-L434", title: "Note 32 §2 S6 — the rule and its floors"}
  - {id: n32-close, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/32-perf-addendum-plan.md#L478-L597", title: "Note 32 §5.1 — which rows were cut and which held"}
  - {id: code-hadronic, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/tests/validate_hadronic.rs", title: "vibegraph-lib/tests/validate_hadronic.rs — budget constants and their ladders"}
---

# Validation budgets matched to reference precision

## The rule

A σ gate compares two Monte Carlo numbers, so its resolving power is
`√(σ_ours² + σ_MG²)`. Once this crate's error is at or below MadGraph's, more
points buy nothing the pull can see; at `1/σ²` cost scaling, a gate running at
one sixth of the reference's error spends ~40× the CPU the comparison can
use[^n32-premise]. So **each σ gate's budget is sized so that `σ_ours ≈ σ_MG`**,
read off a measured budget ladder rather than chosen from cost.

Two floors always win over the precision argument[^n32-s6]:

- **The seed-scatter floor.** The budget must give a clean multi-seed sweep
  (≥ 5 seeds, χ²/dof of the seeds about their mean consistent with 1) at the
  proposed rung. A low rung whose seeds scatter beyond their quoted errors is the
  rung a three-seed gate would be reading.
- **The convergence floor.** The ladder must be flat across the cut. A ladder
  still moving in one direction is a bias, and the answer to bias is never a
  wider tolerance: if the affordable budget is not converged, the row is
  reported `info` at that budget and the decision goes to the user.

How to read a ladder, and why few-seed scatter can mislead it, is
[seed sweeps and budget ladders](seed-sweeps-and-budget-ladders.md); the
statistics behind the floors are in [gate thresholds](gate-thresholds.md).
Wall-time effects of a budget change are measured by
[the end-to-end timing protocol](../performance/end-to-end-timing-protocol.md):
`validate` runs rows concurrently, so a budget cut moves CPU far more than wall
(one application took `validate` from 360.6 s to 342.6 s wall but 1608.9 s to
1321.3 s CPU on the M3 Max)[^n32-close].

## How it applies to the hadronic rows at `787070e`

Budgets and reasons as the constants in `validate_hadronic.rs` record them
(points per VEGAS iteration × iterations, per seed):

| row | budget | seeds | reference error | why this rung |
|---|---|---|---|---|
| `pp_to_jj` | 75 000 × 10 | 5 | 0.22 % | ladder flat over 75k–600k (span 0.08 %, χ²/dof 1.40, 0.44, 0.82, 1.21); our error 0.081 % at the lowest rung |
| `pp_to_bb_fixed` | 75 000 × 10 | 3 | 0.182 % | ladder flat (span 0.05 %); our error 0.076 % |
| `pp_to_bb`, `pp_to_bb_qcd2`, `pp_to_ll_scalefact2` (re-carded) | 75 000 × 10 | 3 | 0.071 %, 0.071 %, 0.198 % | ladders flat at the lowest rung |
| `pp_to_llj_fixed`, `pp_to_llj_dyn` | 150 000 × 10 | 5 | 0.36 %, 0.33 % | precision alone would license 75k; the 75k rung carries the largest seed scatter on both rows (χ²/dof 1.66 and 2.59), so the seed floor wins |
| `pp_to_llj` (re-carded) | 150 000 × 10 | 3 | 0.33 % | held at 150k because the 75k rung's error is inflated by one seed |
| `dy13_default`, `dy13_mmll` | 120 000 × 12 | 3 | 0.048 %, 0.049 % | already matched (our error ≈ 0.058 %); untouched |

The ℓℓj rows reach a converged rung later than the 2 → 2 rows because the same
budget is spread over 24 pooled `(group, diagram)` channels, and the per-channel
allocation floor of 512 points spends at least 12 288 evaluations an iteration
whatever the budget says.

Partonic (`validate_sigma`) rows are cheap as a block and were left alone; the
2 → 6 rows (`uux_to_ccx_emmm_qcd0`, `bbx_to_ccx_emmm_qcd0`) are not budget-limited
but estimator-limited — single seeds swing by ±4–5 % at every rung, so they are
measured and reported on the long tier rather than gated (see
[sigma-row gating exceptions](sigma-row-gating-exceptions.md) and
[two-to-six-integrals-not-enforced](../backlog/validation/two-to-six-integrals-not-enforced.md)).

## Caveats

- **Reference errors move with the bundle.** The four re-carded rows were
  re-banked on `lhaid = 247000` at `refdata-5`, which changed both their values
  and their errors; compare a budget argument only against the reference it was
  measured with ([refdata σ comparability](refdata-sigma-comparability.md)).
- **Some ladder comments in `validate_hadronic.rs` are stale.** The `LLJ_NEVAL`
  family quotes ladders measured before the cut-implied timelike floors
  ([llj-gate-comments-quote-pre-floor-ladders](../backlog/hygiene/llj-gate-comments-quote-pre-floor-ladders.md)),
  and the re-carded `pp_to_llj` entry of `RECARDED_ROWS` still describes its
  ladder as "climbing monotonically". A forty-seed-per-rung ensemble showed that
  climb was a five-seed misread: the estimator's expectation is flat from 150k up,
  and the drift hypothesis fails at 7.3σ (see
  [seed sweeps and budget ladders](seed-sweeps-and-budget-ladders.md)). The 150k
  budget stands on the seed floor either way.
- **A tolerance is not part of this rule.** Budgets move under it; tolerances
  do not. A cut that would need a wider tolerance is not licensed.

[^n32-premise]: Note 32 §0. Its reference-precision table quotes pre-`refdata-5` values for `pp_to_llj` and an older budget; the table above is the current state from the code.
[^n32-s6]: Note 32 §2, S6.
[^n32-close]: Note 32 §5.1, S6.
