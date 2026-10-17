---
type: Session Report
title: "HY1 report: hygiene review of the sigma-pulls tool"
description: "sigma-pulls reuses the seed policy and the record types; its row loss was silent in four places and 27 of 36 one-line mutations of its statistics survived its seven tests. Now loud and 35 of 36 killed, output byte-identical."
verified: [{by: claude-code/claude-sonnet-5-5, at: 2026-10-16}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-16}
sources: [9cc7296]
---
Scope: `validation-report/src/bin/sigma-pulls.rs` at `7e92ac2` (the PR's only
new code). One code commit, `9cc7296`, edits that file only. Logs and drivers
are in the session scratchpad under `hy1/sigmalean-HY1-*`.

## Findings (written before the edits)

| id | pt | site | evidence | conf. | disposition |
|---|---|---|---|---|---|
| H1 | 2 | `Cell::parse` / `read_cells` | A row file whose `pull` is `null` (how a gate writes NaN) returns `Ok(None)` and is skipped. On a copy of the V1 report with `ee_to_mumu.json`'s pull nulled, the base binary exits 0 on **47** rows (48 before), no message. | checked (run) | fixed |
| H2 | 2 | `read_cells` | `.filter_map(\|e\| e.ok()...)` discards a `read_dir` entry error. | checked (read); no test can raise an entry error | fixed, unreachable by a mutation |
| H3 | 2 | `run`, the `> 1e-9` check | Inline in `run`, no test. M13 (tolerance 1e-9 to 1.0) survives the 7 tests. `NaN > 1e-9` is false, so a NaN pull passes the check. | checked | fixed |
| H4 | 2 | `find_record` | M11 (`MATCH_REL` 1e-12 to 1e-7) and M12 (ambiguous join takes the first hit) survive. The one test only moves the value by 1e-6. | checked | fixed |
| H5 | 2 | `population`, `relative_means`, `median`, `Row::*` | Survive: M01 (sd over n), M06, M07 (chi2 dof), M08, M09, M10, M18 (ew_shift error), M20 (heavy), M22, M23, M26, M27, M29, M35. The tests assert `se == sd/sqrt(n)`, which holds for a wrong sd, and never read `unweighted_se`, `weighted_err`, `chi2_per_dof`, `mg_variance_share`, `heavy`, `draws`. | checked (mutations run) | fixed |
| H6 | 2 | `same_draw`, `clusters` | M14-M17 survive. The cluster test's `b` is `a` with every deviation `+0.05`; the draws are deviations from the seeds' mean, so a uniform shift cancels and `b` is exactly `a` (RMS 0, not 0.05). No test has a channel mismatch, a single record with few seeds, or an RMS near 0.35. | checked (arithmetic and mutations) | fixed |
| H7 | 2 | `with_this_side`, `parse_this_side` | A sweep record naming no row, holding one seed, or repeating a seed was silently dropped or double-counted. M31-M33 survive. The `is_err()` assert on a 3-field line matches any error. | checked | fixed |
| H8 | 2 | `read_records` | A record under the wrong directory is keyed by its `row` field; two directories claiming one row overwrite. `seed_record::read` asserts `record.row == row`; this reader did not. | checked | fixed |
| H9 | 1 | doc of `SAME_DRAW_RMS` | "one below 0.35 by chance has probability under 1% at five seeds". The squared differences sum to 2·chi2(4); `1 - exp(-x/2)(1+x/2)` at `x = 0.35^2*5/2` is 0.0106. | checked (`python3`) | fixed ("about 1%") |
| H10 | 1 | `print_population` | 165 lines, with the relative-means line printed by two copies of one `println!` and the list `[R, EW, R4x, EW4x]` written three times. | checked | fixed (split into 6 functions, `RE_READS`, `relative_line`) |
| H11 | 4 | `values`, `seed_value`, `gate_reading`, `reference_value` vs `SeedRecord::reference` / `part_reference` / `reference_of` | The tool does not copy `max(quoted, spread/sqrt(n))`: it calls `seeds::combine`. It does copy the selection around it (estimator by part by budget seeds, then `combine`). `reference_of` is private, fixes the record's estimator, panics below `MIN_SEEDS = 5` and on a missing part, so the tool cannot call it as is. | checked | Found |
| H12 | 3 | whole file | Binary crate, every item private; clippy and `cargo doc --document-private-items` clean. | checked | no finding |
| H13 | 2 | `run`, unmatched cells | A cell that matches no record is printed, not counted or failed on. A drifted `MATCH_REL` or a re-banked record turns rows into "not read" lines and the population shrinks by lines nobody reads. | checked | Found |

Not reproduced: none. Filed items: the two backlog items naming `seed_record.rs`
or `seeds.rs` (`cli-integrate-dy13-skips-seed-record`,
`mlm-match-py-error-not-seed-policy`) concern other files; neither is a finding
here.

## Fixed (commit `9cc7296`)

- **H1, H2, H3, H7, H8** loud. `Cell::parse` returns `Result<Cell, _>` and names the missing field; `read_cells` propagates entry errors and parse errors with the path; `check_reproduction` is `!(|gate - pull| <= 1e-9)`; `with_this_side` returns `Err` for a sweep record no row reads or with one seed, `parse_this_side` for a repeated `(record, seed)`; `read_records` for a record outside its row's directory. Every refusal is asserted by its message.
- **H3, H4** `join_rows` and `check_reproduction` split out of `run` so the tests reach them. `run` is 40 lines.
- **H5, H6** tests 7 to 20. Statistics are pinned to hand values (sample sd 0.908295 of `[1,1,-1,.5,-.5]`, median, positive count, n < 2 and one-cluster `NaN`; `unweighted_se`, `weighted_err`, `chi2_per_dof` of `[(0.01,0.01),(0,0.001)]`; `ew_shift` 0.002 ± sqrt(1e-7), `draws`, `mg_variance_share` 1/6, `heavy`, the EW median and its sqrt(pi/2) error, R4x error, EW4x mean, `rel`). Clustering is tested at RMS 0.2 (joins), 0.5 (does not), a disjoint channel, and two-seed records of one record and of two. The join is tested at 1e-13 and 1e-10, with a twin record (`several records`), and with a `by_lprup` share. Cell, report-directory and record-directory reads use `tempfile`.
- **H9, H10** doc; `print_population` split.

## Mutations

Driver: `hy1/sigmalean-HY1-mut.py` with the list in `sigmalean-HY1-muts.py`
(applies a replacement, runs `cargo test -p vibegraph-validation-report --bin sigma-pulls`, reverts). Old = the 7 tests at `7e92ac2`; new = `9cc7296`. Logs: `sigmalean-HY1-mut-old.log`, `-mut-new.log`, `-mut-new2.log`..`-new4.log`.

| id | mutation | old | new |
|---|---|---|---|
| M01 | sd divides by n | survived | killed (clustered-error test) |
| M02-M05 | se = sd; clustered drops G/(G-1), 1/N; sums uncentred | killed | killed |
| M06 | median even branch takes `v[n/2]` | survived | killed |
| M07-M10 | chi2 dof n; `weighted_err = sqrt(w)`; `unweighted_se = sd`; un_sd over n | survived | killed |
| M11 | `MATCH_REL` 1e-7 | survived | killed |
| M12 | ambiguous join takes the first hit | survived | killed |
| M13 | reproduction tolerance 1.0 | survived | killed |
| M14, M15 | `SAME_DRAW_RMS` 0.6, 0.1 | survived | killed |
| M16, M17 | channel guard dropped; same-record join dropped | survived | killed |
| M18 | `ew_shift` error without /n | survived | killed |
| M19, M21, M25, M28, M34 | EW diff sign; pull denominator a sum; 4x reads budget seeds; draws unstandardized; EW reads reported | killed | killed |
| M20, M22, M23 | heavy ratio 0.5; variance share linear; median error factor pi | survived | killed |
| M24 | `xqcut` not MLM | survived | killed |
| M26, M27, M29, M30 | EW4x reads budget seeds; `rel` inverted; correlation unnormalised; `by_lprup` part ignored | survived | killed |
| M31 | sweep mean over the first two seeds | survived | survived with a 2-seed sweep, killed with 3 |
| M32, M33 | sweep accepts one seed; sweep drops this-side error | survived | killed |
| M35 | positive count `>=` | survived | killed (needs a zero) |
| M36 | `EW_SHIFT_RESOLVED` 2.0 to 20.0 | survived | **survived**: only the printed split reads it; no unit test reaches it |
| M38-M41 | record directory unchecked; unknown sweep record; duplicate sweep seed; `> 1e-9` form (NaN passes) | n/a (new code) | killed, each by the test of its refusal |

Old tests: 9 of 36 killed. New: 35 of 36, plus M38-M41. M37 (`gate.is_finite() && !agrees`) was written to model the NaN pass and did not: it still fails on a NaN cell pull, so it is an equivalent mutant and is not counted; M41 is the real form. Blind spots that remain: M36, and the formatting of every printed line (covered only by the output diff).

Test power was set from the tests' own errors: all bounds are hand values at 1e-12 to 1e-15, no sampled quantity enters.

## Gate

- `cargo fmt --all --check`: `FMT-OK`.
- `cargo clippy --workspace --all-targets -- -D warnings`: `Finished dev profile [optimized] target(s) in 2m 38s`, no warnings.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: `Finished ... in 31.92s`, no warnings.
- `cargo test -p vibegraph-validation-report`: `38 passed` (seed-record), `20 passed` (sigma-pulls, was 7), `4 passed` (validation-report). The 13 added tests are the 20 minus the original 7: all in `sigma-pulls`. `cargo test --workspace` was not run: the brief scopes tests to this crate while two sibling sessions hold the CPU, and the diff touches one binary of this crate.
- `cargo doc -p vibegraph-validation-report --no-deps --document-private-items`: `Finished`, no warnings.
- Banked targets: none touched (no library, gate or reference file changed).
- **Output identity**, the brief's command with the V1 inputs read-only: `diff hy1/sigmalean-HY1-before.txt hy1/sigmalean-HY1-final.txt` prints nothing (293 lines, exit 0, before from the `7e92ac2` binary, after from `9cc7296`; `IDENTICAL` also after the first refactor).
- Loud failure on real data: the V1 report with one `pull` nulled, base binary: exit 0, 47 rows. New binary: exit 1, `sigma-pulls: .../ee_to_mumu.json: "pull" is missing or not a finite number`.

## Seed-policy duplication, and what would conflict with #43

Reused, not copied: `seeds::combine` (policy), `SeedRecord` and its `at_budget` / `screening` / `check`, `SeedRun::value`, `RECORD_DIR`, `RECORD_FILE` (both files path-included). The tool never recomputes `max(quoted, spread/sqrt(n))`. Copied: the record directory walk (`read_records`; `seed_record::read` takes a row name and has no listing) and the estimator/part/budget selection (H11).

Reusing the selection means a `SeedRecord` method taking `(part, estimator, budget)` and returning an `Option` instead of panicking. That edits `seed_record.rs`, which the `seed-record-tooling` branch (PR #43) also edits (`git diff main..seed-record-tooling --stat` lists `vibegraph-lib/tests/common/seed_record.rs` and `validation-report/src/bin/seed-record.rs`), so this change is held until #43 merges and goes in as a rebase on its `seed_record.rs`. Nothing in `9cc7296` touches either file.

## Method

About 40% mutation batches (old, new, repair), 25% reading the tool against `seed_record.rs`, `seeds.rs` and the report writer, 15% hand arithmetic for expected values and the two bounds (H6's cancelling shift, H9's 1.06%), 10% a real-data run for loud-failure and output identity, 10% refactor. The productive lead was "which one-line mutation does this assert not see": 27 of 36 survivors. Arithmetic on the existing cluster test found that its perturbation cancels. Did not hold up: a duplicated policy copy (none), visibility (nothing public), history comments (grep empty), `Class::of` ordering (tested, correct). A first transitive-cluster mistake in my own new test (a 0.5 shift chained through "near") shows the cluster test needs a perturbation in the opposite direction for "far".

## Found

1. **`SeedRecord` selection method** (H11): add `SeedRecord::read(part, estimator, screening) -> Option<SeedReference>` and have `reference()`, `part_reference()` and `sigma-pulls` call it; add a `records(output_dir)` lister beside `read`. Closes the copied selection and directory walk. Waits for #43 (conflicts in `seed_record.rs`).
2. **Unmatched cells are not counted** (H13): `sigma-pulls` prints `not read from a seed record` per cell and exits 0. Closing it: take the expected controls (the four named in V1's report) as an allowlist and fail on any other, or print the joined/unmatched counts on one line.
3. **The sweep file has no committed producer**: `--this-side` reads `record seed sigma_pb err_pb` lines; V1 made them with a scratchpad awk script over the probe's log (`v1/headroom_to_tsv.sh`). Commit the converter or have the probe write the file.
4. **`EW_SHIFT_RESOLVED` and every printed number's format are checked only by an output diff** (M36). A golden-output test on the small synthetic population of the unit tests would pin them.

## Brief corrections

- "The tool's population output unchanged before/after (diff it)": satisfied, but the brief's sweep path is `v1/sigmalean-V1-sweep.tsv` (it exists, 170 lines) and the unmatched controls are four, not the three the first lines of output show before `head`.
- The brief asks whether the tool re-implements "the equal-weight value" or the seed policy: it does neither; the duplicated part is the selection around them (H11).
- The sessions index (`sessions/index.md`) is generated and `pixi run kb-lint` reports it stale once this report exists; I ran `pixi run kb-index` (one file, 2 lines added) and committed it. After that `kb-lint` still prints the pre-existing `mg-single-helicity-bench-no-consumer.md` warning only.
