---
type: Session Report
title: "HY1 report: hygiene review of the seed-record tooling change"
description: "Findings, fixes and mutation evidence for F1's ladder verdict, tie-break, statrs p-values and SEED_PLAN driver."
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-16}
sources: [148c665]
---
## Findings (written before the edits)

| id | pt | site | evidence | conf. | disposition |
|---|---|---|---|---|---|
| H1 | 2 | `seed-record.rs` ladder tests | Both rungs have n = 20 and the same sd. Mutations `s1²/n4 + s4²/n1`, `s4` used for both rungs, `s1` used for both: old suite passes all three (batch below). | checked | fixed |
| H2 | 2 | `ladder_line` sd order | Mutation swapping `(sd s1)`/`(sd s4)` in the line: old suite passes. | checked | fixed |
| H3 | 2 | `chi2_p_value` `dof == 0` guard | Mutation removing it: old suite passes; unreachable from `scatter` (values.len() = base.len() >= 2 gives dof >= 1) so only a direct call can reach it. | checked | fixed (pinned `chi2_p_value(5.0, 0) == 1.0`) |
| H4 | 1 | `seed-record.rs:26` module doc | One 160-column line, `rustfmt` does not wrap comments. | checked | fixed |
| H5 | 1 | `bank_seeded_sigma.sh` header | One 120-column line in the new recipe. | checked | fixed |
| H6 | 4/1 | `stats.rs::gammq` vs `seed_record.rs::chi2_p_value` | Both wrap `gamma_ur` with a guard (`x <= 0` / `chi2 <= 0 or dof == 0`). The guard is the wrapper's only reason: statrs `checked_gamma_ur` errors (and `gamma_ur` panics) for `x <= 0`, `a <= 0`. They cannot share: `seed_record.rs` is path-included into crates that must not depend on `vibegraph-lib` (validation-report builds on a bare clone). | checked | leave |
| H7 | 4 | `seed-record.rs::p_text` vs `cells.rs::pval` | Same bound for exact 0 (`< 1e-300` / `<1e-300`), different formats, in two binaries of one crate with no lib target. | checked | Found (low) |
| H8 | 1 | a p of exactly 0 printed as a value | `{:.3e}` of a χ² p: `vibegraph-lib/tests/validate_samples.rs:880,906`, `vibegraph-cli/tests/validate_samples_proton.rs:508,528`, `vibegraph-cli/tests/cli_decay_chain_events.rs:500,508`. They print `0.000e0` for an underflowed tail. The report cells go through `pval` (bound). | checked | Found |
| H9 | 1 | SEED_PLAN block | `bash -n` ok; `SEED_PLAN=bad` prints `!!! SEED_PLAN is confirm20 or ladder20, not bad` (exit 1); plan sets resolve to 5/2, 20/2, 20/20 seeds with no SEEDS/SEEDS_4X overlap (the pre-existing overlap die also covers explicit values). Recipe reads correctly: paths relative to `validation/madgraph`, `SEEDED_OUT` copy, `--rescreen` command. | checked | no defect |
| H10 | 1 | comment grep on the diff for plan/session/history words | none (`truncated series` mentions give the rationale for the dependency) | checked | no defect |

Backlog check (grep of path tails `/seed_record.rs`, `/stats.rs`, `bank_seeded_sigma`, `seed-record.rs`): `chi2-p-value-floors-at-zero` and `seed-record-ladder-verdict-and-tie-break` (this PR's items), `madgraph-file-parsers-duplicated`, `madevent-generators-accept-empty-results`, `cli-integrate-dy13-skips-seed-record`; the last three name other sites and are unaffected. `chi2-p-value-floors-at-zero`'s `closes_when` is an either/or: its first branch (a non-truncating gamma pinned by Q(2, 65) = 66e^-65) is met by F1 (`a_deep_tail_p_value_is_not_truncated_to_zero`, `the_upper_incomplete_gamma_does_not_truncate_in_the_deep_tail`), so H8 does not hold it open.

## Fixed (commit 148c665)

- H1/H2: `ladder_record_with(n1, n4, amp4, centre4)`; new `the_ladder_weights_each_rungs_spread_by_its_own_count` (20 seeds at ±1, 30 at ±3: σ² = 1/19 + 9/29, asserted with sd, mean, pull and the verdict at 2.1σ and 1.9σ, with a precondition that the swapped σ and the one-spread σ would flip those verdicts) and `the_ladder_line_keeps_the_two_rungs_apart`.
- H3: `chi2_p_value(5.0, 0) == 1.0` in `a_deep_tail_p_value_is_not_truncated_to_zero`.
- H4, H5: comment rewraps.

## Mutations (script: scratchpad copy of `hy1-mut.sh`; one sed edit, `cargo test -p vibegraph-validation-report --bin seed-record`, restore; logs `/tmp/claude-0/hy1-mut-{new,old}.log`)

| mutation (in `vibegraph-lib/tests/common/seed_record.rs` unless noted) | old tests (HEAD, 43) | new tests (45) |
|---|---|---|
| ladder σ as `s1²/n4 + s4²/n1` | pass | `the_ladder_weights_each_rungs_spread_by_its_own_count` fails |
| `s1` used for both rungs' variance | pass | same test fails |
| `s4` used for both rungs' variance | pass | same test fails |
| `chi2_p_value` guard `chi2 <= 0.0 \|\| dof == 0` to `chi2 <= 0.0` | pass | `a_deep_tail_p_value_is_not_truncated_to_zero` fails (statrs panic) |
| ladder gate `&&` to `\|\|` on the two seed counts | `the_ladder_is_not_run_below_twenty_and_twenty` fails | same fails |
| `ladder_line` sd order swapped (`seed-record.rs`) | pass | `the_ladder_line_keeps_the_two_rungs_apart` fails |
| baseline (no-op edit) | 43 pass | 45 pass |

No mutation reaches `p.abs() > LADDER_SIGMA_LIMIT` versus `>=` (an equal-to-2σ pull is not constructible in f64); F1's own batch covers the other sites.

## Gate
Commands run in the worktree with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` after the final edit:
- `cargo fmt --all --check`: rc 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: rc 0 (workspace-wide).
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: rc 0.
- `cargo test -p vibegraph-validation-report`: 45 passed (43 + the two new ladder tests; the dof guard sits in an existing test), 4 passed.
- `cargo test -p vibegraph-lib --lib stats`: 14 passed, 1 ignored (pre-existing).
- `cargo test -p vibegraph-lib --features extended-validation --test validate_sigma --test validate_hadronic --no-run`: compiles (both include `seed_record.rs`; not executed).
- `cargo doc -p vibegraph-validation-report --no-deps --document-private-items`: no warnings. `cargo doc -p vibegraph ...` (not touched) shows one existing warning, unresolved link `cache::resolve::locate` in `vibegraph-cli/src/assets.rs:5`.
- `cargo test --workspace` was started and killed on the manager's instruction (shared host with a heavy sigma run); not run. No banked gate run: no amplitude, sampling or reference-data code changed.

## Method
Diff read in full (about 40%), mutation batch on test power (about 35%), a workspace grep for p-value prints and duplicates (15%), running the SEED_PLAN resolution (10%). Held up: the ±1 alternation blind spot named in F1's report, and the dof guard. Did not: the `gammq`/`chi2_p_value` merge (structurally blocked), history-word comments (none).

## Found
- Printing of χ² p-values as values: `{:.3e}` of an exact 0 at the six sites in H8; close with one shared formatter that reads `< 1e-300` at 0 (a lib-level helper in `vibegraph-lib::stats` for the lib and CLI tests; `cells::pval` and `p_text` stay in their binaries, or the validation-report gains a lib target that both bins use).
- `p_text` (seed-record) and `cells::pval` duplicate the bound rule in two binaries of one crate (H7).
- F1's Found stands: a ladder `Rebank` verdict is not applied by any command.

## Brief corrections
None; the brief's "x = 0 guard" description of `gammq` is right (statrs errors at `x <= 0`). The workspace test run was dropped on the manager's instruction (shared host); workspace-wide coverage is the two clippy runs.
