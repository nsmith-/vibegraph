---
type: Sprint Record
title: "seed-record: ladder verdict, 20-seed recipe, p-value floor"
description: "seed-record prints the 20 + 20 ladder verdict on measured spreads, SCATTER's worst channel breaks p ties on χ², χ² p-values come from a non-truncating upper incomplete gamma, and the banking driver carries the 20-seed confirmation and ladder recipes."
closed: 2026-10-16
status: draft
tags: [seed-record, tooling, chi2, ladder, refdata, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/43", title: "nsmith-/vibegraph#43"}
---
One PR ([nsmith-/vibegraph#43](https://github.com/nsmith-/vibegraph/pull/43))
closing `seed-record-ladder-verdict-and-tie-break` and
`chi2-p-value-floors-at-zero`. Sessions: [F1](sessions/F1-report.md)
(validation-dev, Sonnet) and [HY1](sessions/HY1-report.md) (hygiene-dev,
Sonnet).

## What was established

- **Ladder verdict.** `seed-record` prints a `LADDER` line after `SCREEN`: σ =
  √(s₁ₓ²/n₁ₓ + s₄ₓ²/n₄ₓ) from the seeds' sample standard deviations, re-bank at
  4× beyond 2σ, "not run" below 20 + 20 (D1's fourth amendment in
  `sprints/sigma-reference-scatter`). No record in refdata-10 holds a 20 + 20
  ladder (all 55 are 5 + 2), so the verdict is pinned on synthetic records,
  with unequal rung counts and spreads so a swap of the two rungs is seen.
- **Tie-break.** SCATTER's worst channel breaks p ties on the larger χ². Over
  all 55 records only the two flagged rows' named channel changed
  (`uux_to_ccx_emmm_qcd0`: G89, χ²/dof 289; `bbx_to_ccx_emmm_qcd0`: G121.239,
  33.5); verdicts are unchanged, 2 flagged and 53 within chance.
- **p-values.** `statrs` 0.19 `gamma_ur` (default features off) replaces
  `puruspe`'s truncating series in `seed_record.rs` and `stats.rs`; Q(2, 65)
  matches 66·e⁻⁶⁵ to 2.4e-15. An exact 0 now means f64 underflow (below about
  1e-308), and `seed-record` prints it as `< 1e-300`.
- **Recipes.** `bank_seeded_sigma.sh` documents and offers
  `SEED_PLAN=confirm20` (20 seeds at 1×) and `ladder20` (20 + 20), each into a
  separate `SEEDED_OUT` so the banked seeds are read back.

## Trust

Machine-confirmed by the manager: the commits' author and trailers; both
crates' tests (45 + 4 in `vibegraph-validation-report`) and `cargo fmt`
re-run; one of HY1's mutations (the 4× rung's spread replaced by the 1×'s)
re-run and failing `the_ladder_weights_each_rungs_spread_by_its_own_count`.
Asserted by the sessions: the 55-record rescreen diff, workspace clippy, the
other mutations. The `SEED_PLAN` recipes have not run against MadEvent.

## Filed

- [seed-record-rebank-verdict-not-applied](../../backlog/hygiene/seed-record-rebank-verdict-not-applied.md)
- [chi2-p-prints-underflow-as-zero](../../backlog/hygiene/chi2-p-prints-underflow-as-zero.md)
