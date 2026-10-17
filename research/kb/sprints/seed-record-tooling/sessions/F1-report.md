---
type: Session Report
title: "F1 report: seed-record ladder verdict, tie-break, deep-tail p-values"
description: "What changed in seed-record, stats.rs and the seeded-sigma driver, with test, mutation and before/after rescreen evidence."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-16}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-16}
sources: [0f9a7ed9b0b08547228297a677da844388d91699]
---
## What changed (commit 0f9a7ed)

- `vibegraph-lib/tests/common/seed_record.rs`: `SeedRecord::ladder()` (mean and sample sd of the reported values per rung, sigma = sqrt(s1^2/n1 + s4^2/n4), quoted errors not read; Rebank if |diff| > 2 sigma, Stays otherwise, NotRun below 20 + 20); SCATTER sort breaks p ties on larger chi2; `chi2_p_value` uses `statrs` `gamma_ur` (chi2 <= 0 or dof 0 gives 1).
- `validation-report/src/bin/seed-record.rs`: prints a `LADDER` line after `SCREEN` (SCREEN unchanged); `p_text` shows `< 1e-300` for exact 0 (true f64 underflow).
- `vibegraph-lib/src/stats.rs`: `gammq` wrapper over `statrs::function::gamma::gamma_ur` (x <= 0 gives 1, since statrs panics there). `puruspe` removed from both crates (Cargo.lock updated).
- `validation/madgraph/bank_seeded_sigma.sh`: header recipe plus `SEED_PLAN=confirm20|ladder20` (20 seeds = 20261101-05 + 20261201-15; 4x = 20261111-12 + 20261301-18), `LADDER` in the end-of-run grep.

Crate check: `cargo search "incomplete gamma"` / `statrs` found statrs 0.19.1 (maintained), which I probed in a scratch crate before adopting: `gamma_ur(2,65) = 3.894059757454051e-27` vs `66 e^-65 = 3.8940597574540604e-27` (rel 2.4e-15); `gamma_ur(1,700)` matches `e^-700`. puruspe returned 0 for all those. No hand-written special function. `default-features = false` avoids nalgebra.

## Brief corrections

- The bundle has no 20 + 20 ladder: all 55 records hold 7 seeds (`python3` count over `validation/madgraph/output/seeded_sigma/*/seed_record.json`, every row 5 + 2). The -0.87 sigma pin is therefore a synthetic record on both sides of 2 sigma, not `ll_to_qqx_toy_dipole`. 
- With statrs, p underflows to 0 only below ~1e-308, so the tie-break now matters only for chi2 > ~1400 on 19 dof; `uux_to_ccx_emmm_qcd0` already names the chi2/dof 289 channel from the p-value alone. The tie-break is pinned by a test where p is exactly 0.
- The "not run" LADDER line still prints the readings (means, sd, sigma) for 5 + 2 records; the verdict is the only thing withheld.

## Gates (worktree /home/user/wt/seed-record-tooling)

- `cargo test -p vibegraph-validation-report`: 43 passed (seed-record bin, 14 new/changed ladder, tie-break, p-value tests among them), 4 passed (collator).
- `cargo test -p vibegraph-lib --lib stats`: 14 passed, 0 failed, 1 ignored (pre-existing ignore, `expansion_bounds_arenas`).
- `cargo test -p vibegraph-lib --features extended-validation --test validate_sigma --test validate_hadronic --no-run`: compiles (both include `seed_record.rs`); not executed (reference-data gates, sibling host is running sigma gates).
- `cargo fmt --all --check`: clean. `cargo clippy -p vibegraph-lib -p vibegraph-validation-report --all-targets -- -D warnings`: clean. Workspace-wide clippy and the banked `pixi run validate` were not run (no amplitude/eval code touched; host contention).

## Mutation evidence (script: scratchpad F1-mut.sh; one edit, test run, restore)

| mutation | failing test |
|---|---|
| ladder `p.abs() >` to `p >` | the_ladder_decides_on_the_measured_spread_either_side_of_two_sigma |
| ladder seed floor 20 to 19 | the_ladder_is_not_run_below_twenty_and_twenty |
| sigma term s1^2 doubled | the_ladder_decides_... and the_ladder_line_states_the_readings_and_the_verdict |
| tie-break removed | a_tie_at_zero_p_names_the_channel_with_the_larger_chi_squared |
| tie-break ascending | same test |
| seed_record p truncated at x > a + 20(sqrt(a)+1) | a_deep_tail_p_value_is_not_truncated_to_zero |
| stats.rs gammq truncated likewise | the_upper_incomplete_gamma_does_not_truncate_in_the_deep_tail, a_deep_tail_chi_squared_reports_its_p_value |

Blind spots: the ladder tests use symmetric alternating rungs, so they cannot detect a swap of the two rungs' sd roles (a different sd per rung would). The sd uses n-1; a test with n = 20 pins that through the `sqrt(2/19)` sigma. Quoted-error independence is pinned by 1e6 quotes in the test records.

## Before/after rescreen (all 55 records)

Commands: `for f in validation/madgraph/output/seeded_sigma/*/seed_record.json; do target/debug/seed-record --rescreen $f; done` on the stashed baseline (HEAD d07753f) and on the change; outputs `F1-rescreen-before.txt` (1100 lines) and `F1-rescreen-after.txt` (1155 lines) in the scratchpad.

- After minus `LADDER` lines, `diff` shows exactly 2 changed lines, both `SCATTER`; the 55 `LADDER` lines add 55 lines (all "not run", 5 + 2 seeds).
- SCREEN/SURVEY lines: identical md5 before and after. Verdict counts before and after: 2 FLAGGED, 53 within chance.
- Changed: `bbx_to_ccx_emmm_qcd0` p `< 1e-16` to 1.85e-25 (worst channel G1 at chi2/dof 25.88 became G121.239 at 33.48, p 5.61e-28); `uux_to_ccx_emmm_qcd0` p `< 1e-16` to 1.06e-246, worst G34.008 at 34.63 became G89 at 289.11 (p 4.40e-249). Both FLAGGED before and after.

## Request for the user (M3 only, nothing banked here)

None required for this PR. To exercise the new path: `SEED_PLAN=ladder20 SEEDED_OUT=/scratch/confirm ROWS=<row> pixi run -e madgraph bank-seeded-sigma` after copying `output/seeded_sigma/<row>` there (recipe in the script header); for the two FLAGGED rows `SEED_PLAN=confirm20`. I did not run the driver (it needs MadEvent); I tested only the seed-set resolution (`confirm20`: 20 and 2 seeds, `ladder20`: 20 and 20, no overlap between SEEDS and SEEDS_4X, bad value rejected) and `bash -n`.

## Found

- The driver never turns a "re-bank at 4x" verdict into a record: the record's budget rung stays 1x and the gates read it; D1 says the 4x seeds then become the budget rung, which needs either a `--budget` re-run of seed-record on the 4x seeds or a record field. Close when a ladder verdict of Rebank can be applied by one command.
- `SEED_PLAN` and the confirm recipe are untested end to end (needs MadEvent on the M3).
- stats.rs `chi2_homogeneity` p of exactly 0 can still occur on underflow (< 1e-308) and prints as 0 anywhere it is shown; no verdict depends on it.
