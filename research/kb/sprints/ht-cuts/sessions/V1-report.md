---
type: Session Report
title: "V1 report: the HT σ gates"
description: "The two σ gates for the HT-binned rows, their planned-row handling, the synthetic-record measurement and the mutations on the planned/absent guard."
verified: [{by: claude-code/claude-sonnet-5-5, at: 2026-10-23}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-23}
---
## What changed

- `vibegraph-lib/tests/validate_hadronic.rs:3347-3398`: `HT_ROWS`, `seed_record_banked`, `measure_ht_sigma`, and the tests `sigma_jj_ht_window_vs_mg` and `sigma_bb_iht_window_vs_mg`. Each runs `measure_recarded_sigma` (the path the re-carded rows use: the banked run's `Cards/run_card.dat`, `groups_for`, five seeds at 75000 x 10, `combine`, `|pull| < 3`, `|rel| < 0.005`, χ²/dof < 4, writes the `integrals` cell). The reference is the row's seed record only. Record absent and row `planned`: prints `NOT YET BANKED`, passes, writes no cell. Record absent and row not planned: `validation::require`.
- `vibegraph-lib/tests/common/manifest.rs:46,106-117`: `status` read, `planned_rows()`. Ported byte-identical from `origin/sprintd-flavour-mixed-masses` (identical there and on `origin/sprintd-unequal-beams`).
- `validation-report/src/main.rs:76,361-368`: the `ProcessStatus` import and the collator's planned branch (a planned row with no cell renders `⏳`, no problem). Ported byte-identical, same source.
- `validation-report/src/main.rs:586-`: test `each_planned_row_with_no_cell_waits_and_an_ordinary_one_is_missing`, placed at the end of the test module. The two other PRs' tests sit at its head and name their own rows (`pp_to_ll_etau`, `pp_to_ll_beams_6500_4000`), which do not exist here, so this one is not a copy; it is a different name in a different position so the merges do not collide on it.
- `validation/manifest.toml:649,664`: both rows keep `status = "planned"`; `integrals` is `tier = "banked", mode = "gate"` with a note carrying `<...>` placeholders for the measured numbers (the form `pp_to_ll_beams_6500_4000` uses).

## Synthetic-record measurement

A script (scratchpad, not committed) wrote `validation/madgraph/output/pp_to_<row>/Cards/run_card.dat` (beams 6500, `lhapdf` 247000, the HT lines) and `output/seeded_sigma/<row>/seed_record.json` with five seeds at budget: jj mean 2.13516e5, seed sd 393; bb mean 4841.24, per-seed sd and quoted error 7.1 (so the mean's error is 3.175). Assumption: the M3's bb χ²/dof 0.14 is not reproduced (it needs quoted errors above the spread, and then the mean's error is not 3.175). The directory is gitignored and removed afterwards.

```
cargo test -p vibegraph-lib --features extended-validation --test validate_hadronic -- sigma_jj_ht sigma_bb_iht --nocapture --test-threads 1
[pp_to_bb_iht] MadEvent reference ... 5 reported seeds at nevents 10000, 4.841240e3 +- 3.175e0 pb
[pp_to_bb_iht] GATE vibegraph σ = 4.844779e3 ± 4.722e0 pb (5 seeds, χ²/dof = 1.66) | MG σ = 4.841240e3 ± 3.175e0 pb | pull = +0.62 | rel = +0.0007
[pp_to_jj_ht] MadEvent reference ... 5 reported seeds at nevents 10000, 2.135160e5 +- 1.758e2 pb
[pp_to_jj_ht] GATE vibegraph σ = 2.134238e5 ± 2.005e2 pb (5 seeds, χ²/dof = 0.64) | MG σ = 2.135160e5 ± 1.758e2 pb | pull = -0.35 | rel = -0.0004
test result: ok. 2 passed; 0 failed; finished in 57.41s
```

The cells (`target/validation-report/integrals/pp_to_jj_ht.json`) read `mode gate`, `status pass`. Both rows together take 57 s at 75000 x 10, so the HT-only bb card's budget is not a cost problem. Per-seed bb values: 4848.1, 4850.1, 4825.9, 4849.6, 4850.1 (one seed 0.5 % low); the gate's seeds (20260901-05) read χ²/dof 1.66 against the 4.0 bound.

With the M3's own reference the numbers the gate would read are these only if the real record matches the synthetic one; the pull is the measurement, and it moves with the real mean and errors.

## Mutations (planned/absent guard)

Each mutation applied to the file, the two tests (or the collator tests) run, the file restored.

| mutation | state | result |
|---|---|---|
| none | record absent, row planned | `NOT YET BANKED` for both, 2 passed |
| remove `status = "planned"` from both rows | record absent | both FAIL through `validation::require` ("needs a banked seed record (pp_to_jj_ht)") |
| `if false && planned_rows().contains(row)` | record absent, planned | both FAIL (the guard is what lets a planned row pass) |
| `seed_record::read` result ignored (`if false`) | record present, planned | both print `NOT YET BANKED` and pass, no `GATE` line, no cell |
| collator planned branch removed (`if false && ...`) | planned row with no cell | `each_planned_row_with_no_cell_waits...` FAILS: "banked and no gate wrote it" |

The fourth row is the planned state's blind spot: while a row is planned, a gate that ignores a present record still passes. What catches it is the `GATE vibegraph σ` line and the cell, which the M3 checks for on landing (below). Once `status` is removed the same mutation fails through `require`.

## Gates

```
cargo fmt --all --check                          exit 0
cargo clippy --workspace --all-targets -- -D warnings     Finished, exit 0
cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings   Finished, exit 0
cargo test --workspace                           9 distinct "test result: ok" lines in the summary, no "failed" count above 0
cargo test -p vibegraph-validation-report        45 passed; 20 passed; 12 passed (3 binaries)
python3 scripts/kb.py lint                       913 concepts, 296 backlog items; 0 errors, 4 warnings (the four blocked-without-blocked_by items, untouched)
cargo test -p vibegraph-lib --features extended-validation --test validate_hadronic -- sigma_jj_ht sigma_bb_iht
                                                 2 passed, both NOT YET BANKED (no record in the tree)
```

`pixi run validate` was not run: the tree has no reference bundle and nothing here changes amplitudes or eval.

## Landing the records (for the M3)

1. Put the bundle's `output/pp_to_jj_ht/`, `output/pp_to_bb_iht/` (the `Cards/run_card.dat` is read) and `output/seeded_sigma/<row>/seed_record.json` into `validation/madgraph/output`.
2. `cargo test -p vibegraph-lib --features extended-validation --test validate_hadronic -- sigma_jj_ht sigma_bb_iht --nocapture`. `NOT YET BANKED` must be replaced by `MadEvent reference from seeded_sigma/...` and `GATE vibegraph σ`; if the line is still `NOT YET BANKED` with the record in place, the record is not where the gate looks.
3. Record pull, rel, χ²/dof and the record's numbers in the `<...>` placeholders of the two `integrals` notes. For `pp_to_bb_iht`, hold the flip until the 20 + 20 budget ladder is read; the gate sits at 75000 x 10 and a ladder move on the MadGraph side may need a different `neval` in `HT_ROWS` (`validate_hadronic.rs:3347`).
4. Delete `status = "planned"` from the row. After that an absent record fails the test through `require`, and a missing cell is a collator problem.
5. Run `pixi run --skip-deps validate` for the report. If the bundle is re-cut in a different PR, add `bundled = false`.

## Found

- **Planned-row handling has several spellings.** `dyn_run_present`, `run_present`, `check_dy_run`, `not_yet_banked` and now `seed_record_banked` each decide absent-and-planned separately (the unequal-beams report found the same). One helper in `common/manifest.rs` returning the row's declared state would close it.
- **The vibegraph-side budget ladder does not cover the HT rows.** `probe_recarded_budget_ladder` iterates `RECARDED_ROWS` and skips planned rows through none of the new code, so the HT rows (kept out of that table) have no ladder probe. Close: a probe over `HT_ROWS` at 75k/150k/300k/600k once the records land.
- **The sampling-regulator gap (F1 Found 1) shows at the gate.** The bb gate's seeds read χ²/dof 1.66 and one seed 0.5 % low; this is not a failure at 75000 x 10, and the map code was not touched.
