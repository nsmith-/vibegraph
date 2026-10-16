---
type: Session Report
title: "HY1 report: hygiene review of the seeded banking driver"
description: "Review of F1 and F2's seeded banking driver, seed-record writer and seed-record reader. Findings written before any edit: five refusals no test reached (record non-finite values, reference and read re-checks, two partitioned-seed refusals), any-panic and any-refusal asserts, an untested screen statistic, a vacuous launch-block check and launch cut in the shell, a 137-line reader, two in-file duplicates, a swallowed read_dir error and a hand-written temp directory."
status: draft
tags: [madevent-reference, sigma, seeds, hygiene, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-16}]
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-10-16"
  command: "see each section; scripts and logs under the scratchpad's refshy1/ (refshy1-*)"
sources:
  - {id: brief, resource: "HY1.md", title: "HY1 brief"}
  - {id: f1, resource: "F1-report.md", title: "F1 report"}
  - {id: f2, resource: "F2-report.md", title: "F2 report"}
---
Review of `git diff origin/main...HEAD -- . ':!research'` at `5cd754b`. The
findings table below was written, and its pre-fix mutations run, before any
code edit; the dispositions column was filled in after the fixes.

## Findings (written before the edits)

Sites are at `5cd754b`. `REC` is `vibegraph-lib/tests/common/seed_record.rs`,
`BIN` is `validation-report/src/bin/seed-record.rs`, `MG` is
`validation/madgraph/`. A pre-fix mutation result reads "survived" when the
whole `seed-record` test binary (25 tests) passed with the mutation applied
(`refshy1-mutate.py`, logs `refshy1-mut-before.log` and
`refshy1-mut-before-named.log`). The pre-fix refusals that a test did reach:
`read_run`'s event count (killed by two tests), its σ refusal, `check`'s event
count, schema, row and missing-process refusals.

| id | pt | site | evidence | mutation (pre-fix result) | conf. | disposition |
|---|---|---|---|---|---|---|
| H1 | 2 | `REC:424` `check`, non-finite refusal | No test builds a record with a NaN or infinite value; `NaN <= 0.0` is false, so the positive-σ refusal does not stand in | `if false && values…is_finite()` (survived) | checked | fixed: `a_record_refuses_a_non_finite_value` |
| H2 | 2 | `REC:480` `reference_of` re-check | No unit test calls `reference()` on a record that fails `check` | delete the `check()` call (survived) | checked | fixed: `a_record_that_fails_its_check_is_no_reference` |
| H3 | 2 | `REC:654` `read`'s `check` | `reference_of` re-checks, so deleting `read`'s check only drops the record's path from the panic; no test reads the message | delete the call (survived) | checked | fixed: `a_work_area_without_a_record_reads_none` asserts the path in the panic |
| H4 | 2 | `BIN:448` windows that ran different `nevents` | No test; the seed takes `runs[0].nevents`, and `check` cannot see a window's own count | `if false && …` (survived) | checked | fixed: `a_partitioned_seed_refuses_windows_at_other_budgets_or_with_several_processes` |
| H5 | 2 | `BIN:460` partitioned seed with several processes | No test | `if false && …` (survived) | checked | fixed: same test |
| H6 | 2 | `BIN:349` no channel listed | Unreachable as a defect: with no channel the sum is 0, and `read_run` has already refused σ ≤ 0, so the channel-sum refusal fires. Only the message differs | `if false && …` (survived) | checked | fixed: the message is asserted (`a_run_missing_a_channel_is_refused`); the defect class stays unreachable |
| H7 | 2 | `BIN:522` `parse_args` | No test of any argument refusal (unknown flag, missing value, missing flag, non-integer `--budget`/`--hard-survey`) | `--hard-survey` parse error → `unwrap_or(2)` (survived); unknown flag ignored (survived) | checked | fixed: `the_arguments_refuse_what_they_cannot_read` |
| H8 | 2 | `REC:535` screen statistic | `the_screen_flags…` uses pulls that are all far off (≈ 40) or all zero, so the flag reads the same on the 4× mean's pull, the first seed's, or any seed's | `worst = pulls.first()` (survived) | checked | fixed: `the_screen_reads_the_screening_seeds_mean_not_any_one_seed` |
| H9 | 2 | `REC:515` paired EW − reported error | Only pinned at 0 (constant offset), where any normalisation gives 0 | `/ (n-1) / n` → `/ n` (survived) | checked | fixed: `the_paired_shift_carries_the_standard_error_of_its_mean` |
| H10 | 2 | `BIN:1144`, `BIN:1155` | `catch_unwind(…).is_err()` discards the payload: any panic passes (missing-process and wrong-row refusals) | — (the refusals themselves are reached: dropping the row assert is killed by `a_work_area_without_a_record_reads_none`, reading another process for a missing one by `one_process_is_read_from_its_own_share`) | checked | fixed: payload matched (`panic_message`) |
| H11 | 2 | `BIN:1107`, `:865`, `:1174`, `:1217` | Any-refusal asserts: `check().is_err()` for the schema, `parse_symfact("1\n").is_err()`, `parse_run_config(…).is_err()`, the missing `results.dat` | schema check removed (killed by `a_record_holding_a_seed_at_a_third_budget_or_twice_is_refused`: the record is otherwise valid, so the bare `is_err` holds today by construction, not by assertion) | checked | fixed: each matches its message |
| H12 | 2 | `MG/madevent_seeds.sh` `mes_check_card` | Green on an empty set: with every launch setting skipped, or a script whose block starts `launch -f` (the sed range needs a bare `launch`), nothing is compared and it returns 0. `refshy1-shcheck.sh`: `check_card a.mg5 b_card.dat [ebeam1 nevents]: rc=0`, `check_card nolaunch.mg5 b_card.dat []: rc=0`. Today every one of the 64 scripts keeps ≥ 2 settings after `iseed nevents` are skipped | — | checked | fixed: refuses with no setting compared |
| H13 | 2 | `MG/build.sh` `script_driver … no-launch` | A non-bare `launch` line is not cut, so the "never-run" template would be launched at generation: a `launch -f` script came back with its launch block, rc 0. All 64 committed scripts use a bare `launch` today | — | checked | fixed: refuses a non-bare launch line |
| H14 | 2 | `MG/madevent_seeds.sh` `mes_check_run` | No committed test (shell + Python, outside the hermetic layer). Each refusal does fire on a hand-made run (`refshy1-shcheck.sh`: short, empty, NaN, zero, truncated gzip all rc 1; good rc 0) | see Mutations | checked | Found (no committed test); refusals shown by scratch mutation |
| H15 | 1 | `BIN:270-407` `read_run` | 137 lines, past the ~100-line rule with no named reason; the channel walk (`:308-363`) and the `<init>` split (`:365-392`) are separable | — | checked | fixed: `read_channels`, `split_by_lprup` (`read_run` 137 → 65 lines; longest function now `read_seed`, 89) |
| H16 | 1, 4 | `BIN:86-92`, `BIN:134-141` | `Channel::parse`'s first line and `parse_total` are the same parse (first two Fortran reals of the first line) | — | checked | fixed: one `value_and_error` |
| H17 | 1, 4 | `BIN:412-415`, `:424-430`, `:562` | The `s<digits>`/`w<digits>` directory test is written three ways (`strip_prefix`+`parse`, `strip_prefix`+`is_some_and`, `starts_with`+slice) | — | checked | fixed: `numbered`, `dir_name` |
| H18 | 2 | `BIN:263` `sorted_dirs` | `.filter_map(\|e\| e.ok()…)` swallows a `read_dir` entry error, which would drop a seed or a channel directory silently | no mutation can provoke an entry error in a test | checked | fixed: entry errors propagate (no mutation reaches) |
| H19 | 1 | `BIN:790` `scratch` | Hand-written temp directory (`temp_dir()` + pid), removed by hand at each test's end, so a failing test leaks it. `tempfile` is already in the workspace (`vibegraph-cli` dev-dependency) | — | checked | fixed: `tempfile` dev-dependency |
| H20 | 1 | `REC:53` module doc | Says the file depends on `std`, `serde`, `puruspe` and `seeds` only; `read` uses `serde_json` | — | checked | fixed |
| H21 | 1 | `REC:390` `check` doc | Lists schema, distinct seeds, budgets, finite values; the body also refuses σ ≤ 0, a short event count and another `hard_survey` | — | checked | fixed |
| H22 | 1 | `MG/bank_seeded_sigma.sh:239` | `run_fresh ROW SEED NEVENTS CARD PARAM` omits the sixth argument, `OVERRIDES` | — | checked | fixed |
| H23 | 3 | `REC:208`, `REC:293` | `DEFAULT_RUN_CONFIG` and `SCREEN_PULL_LIMIT` are `pub` with no user outside the module (`grep` over `BIN`, `validate_sigma.rs`, `validate_hadronic.rs`: 0) | — | checked | fixed: private |
| H24 | 4 | `MG/gen_pta_windows.sh:229-234` | The new `hard_survey` replace-or-append duplicates `mes_install_card` (`madevent_seeds.sh:85-97`), which the script already sources; `nevents`/`iseed` were already a second copy | — | checked | fixed: `install_cards` calls `mes_install_card` |
| H25 | 1, 4 | `REC` vs `BIN` (the brief's question) | One schema: `BIN` includes `REC` and `seeds.rs` by `#[path]`, so types, `check`, `combine` and the reading rule have one copy. `read_run`'s σ and event refusals overlap `check` for a one-window seed but are per window (a negative window can hide in a positive sum) and carry the path | — | checked | leave: one copy already; the overlap is finer-grained, see below |
| H26 | 1 | `BIN` size, 1337 lines | 766 lines of code (parsers ~230, run/seed reader ~270, args ~50, printing ~180) and 571 of tests | — | checked | leave: after H15 no function passes 100 lines; see below |
| H27 | 1 | shell: `set -euo pipefail`, quoting, the main guard, dead options | `bank_seeded_sigma.sh` and `gen_pta_windows.sh` set `-euo pipefail`; `madevent_seeds.sh` is sourced; `build.sh` keeps `set -e` and its guard `[ "${BASH_SOURCE[0]}" = "$0" ]` holds (`bash -c '. ./build.sh; type build_main'` printed nothing of the build). Unquoted `$SEEDS`/`$ROWS`/`$(group_rows …)` are deliberate word splits. Every documented variable is read | — | checked | leave: nothing wrong |
| H28 | 1 | `MG/build.sh` `script_driver` | `$(mktemp -t vg_build_XXXXXX).mg5` leaves the empty `vg_build_XXXXXX` behind on every call (moved, not added, by this PR) | — | checked | fixed: `mktemp` then `mv` to `.mg5` |
| H29 | 4 | cross-module | Second copies of the writer's parsers outside the change (see Found) | — | checked | Found |

Backlog check: `grep -rlE 'bank_seeded_sigma|seed-record|seed_record|madevent_seeds\.sh|madgraph/build\.sh|gen_pta_windows|time_stages|gen_mlm_references\.sh|gen_onshell_veto\.sh|assemble_bundle' research/kb/backlog`
returns `validation/reference-sigma-seed-scatter-untested.md` (the item this PR
claims) and `validation/uux-wbwb-onshell-veto-tt-2pct-off.md` (names
`gen_onshell_veto.sh` for its row's physics). Neither names a site that is
wrong or already fixed.

The comment grep (`\bT[0-9]\b|\bC[0-9]\b|Phase [0-9]|note [0-9]+|used to|no longer`,
plus `D1|amendment|sprint|session`) over the added lines finds two hits, both
benign: "no longer sums to the total" describes the test's own setup, and "the
session's history" is MadGraph's session.

## Fixed

Commits are listed in Gate. Nothing in `validate_sigma.rs`,
`validate_hadronic.rs` or `helas/` changed; in `REC` only two doc comments and
two `pub`s (H20, H21, H23). The gates read `REC` through `read`, `reference` and
`part_reference`, whose bodies did not change.

- **Writer tests (H1–H11).** Six new tests in `BIN`, and the tightened asserts.
  The suite went from 25 to 31 tests. The new ones:
  `a_record_refuses_a_non_finite_value`,
  `a_record_that_fails_its_check_is_no_reference`,
  `a_partitioned_seed_refuses_windows_at_other_budgets_or_with_several_processes`,
  `the_arguments_refuse_what_they_cannot_read`,
  `the_screen_reads_the_screening_seeds_mean_not_any_one_seed`,
  `the_paired_shift_carries_the_standard_error_of_its_mean`. The screen test
  pins the two cases where the 4× mean and the single pulls disagree. In the
  first, each seed sits at 102.2, pull 1.80 alone and 2.20 as a mean, so it is
  flagged. In the second, the seeds sit at 103 and 97.6, so the first alone
  pulls 2.45 but the mean pulls 0.11, and it is not flagged. Every expected
  number is worked by hand in the test's comments, not taken from a run.
- **Shell refusals (H12, H13).** `mes_check_card` exits with "no launch-block
  setting to compare" when nothing was compared. `script_driver … no-launch`
  refuses a launch line that is not bare `launch`. Before and after, from
  `refshy1-shcheck.sh` and the commands under Mutations:

  | input | before | after |
  |---|---|---|
  | settings all skipped (`ebeam1 nevents`) | rc 0 | rc 1, "no launch-block setting to compare" |
  | `launch -f` script | rc 0 | rc 1, same |
  | real disagreement (`ebeam1` 45.6 vs 50.0) | rc 1 | rc 1 |
  | `script_driver no-launch` on a `launch -f` script | rc 0, block kept | rc 1, "has a launch line that is not a bare 'launch'" |
  | every committed `scripts/*.mg5`, `script_driver no-launch` | — | 64 ok, 0 refused, no launch/set/done line left |
  | every committed script against its banked `output/<row>/Cards/run_card.dat` (or `<row>_run_card.dat` for `built-by` rows), `iseed nevents` skipped | — | 64 ok, 0 failed |

- **`read_run` split (H15), merged duplicates (H16, H17), `sorted_dirs`
  (H18).** The identity oracle was chosen before the edit:
  `refshy1-oracle.sh`. It runs `seed-record` on fixed copies of F2's dry-run
  kept files: `ee_to_mumu` and `pp_to_bb_fixed`, each with and without
  `--validated-ew`, and a partitioned row built from them, each `ee_to_mumu` seed
  as two windows. It keeps the 10 outputs, stdout and `seed_record.json` for
  each case. `diff -r oracle/before oracle/after` printed nothing
  (`ORACLE-IDENTICAL`). One behaviour change sits outside the oracle: a
  directory named bare `s` under the row is now skipped. It used to fail
  `read_seed` with "not an s<iseed> directory".
- **`gen_pta_windows.sh` (H24).** `install_cards` sets `nevents`, `iseed` and
  `hard_survey` through `mes_install_card`, and keeps its own Python only for
  the window's `pta`/`ptamax`. Identity oracle `refshy1-ptacards.sh`: the
  installed run card on the banked `output/ee_to_mumua/Cards/run_card.dat`
  under four settings (plain, `HARD_SURVEY=2`, the window cuts, both).
  `diff -r ptacards/before ptacards/after` was empty (`PTA-CARDS-IDENTICAL`),
  for example `7ef13fc1…` for `hs2-window`. A side effect: `run_pta`'s stamp
  hashes `gen_pta_windows.sh`, so a pta seed kept before this commit re-runs.
  None is banked.
- **Docs and visibility (H20–H23, H28).** As in the table.

## Mutations

Rust: `refshy1-mutate.py`. It applies one exact-string replacement, runs
`cargo test -p vibegraph-validation-report --bin seed-record`, restores the
file and logs the outcome. The source files were `cmp`-identical to the fixed
copies after each batch.

| id | mutation | old tests (`5cd754b`) | new tests |
|---|---|---|---|
| P1 | `check`: skip the non-finite refusal | survived | killed: `a_record_refuses_a_non_finite_value` |
| P2 | `check`: skip the schema refusal | killed: `…_third_budget_or_twice_is_refused` (bare `is_err`) | killed: same test, now on the message |
| P3 | `reference_of`: drop `check()` | survived | killed: `a_record_that_fails_its_check_is_no_reference` |
| P4 | `read`: drop `check()` | survived | killed: `a_work_area_without_a_record_reads_none` |
| P5 | `read`: row assert compares `row` with itself | killed: `a_work_area_without_a_record_reads_none` (any panic) | killed: same test, now on the payload |
| P6 | `read_seed`: skip the windows-`nevents` refusal | survived | killed: `a_partitioned_seed_refuses_windows_…` |
| P7 | `read_seed`: skip the partitioned-multi-process refusal | survived | killed: same |
| P8 / P8b | skip the no-channel refusal (`channels == 0`, then `c.count == 0`) | survived | killed: `a_run_missing_a_channel_is_refused` |
| P9 | `--hard-survey` bad value → `unwrap_or(2)` | survived | killed: `the_arguments_refuse_what_they_cannot_read` |
| P10 | screen flag on the first seed's pull | survived | killed: `the_screen_reads_the_screening_seeds_mean_not_any_one_seed` |
| P11 | paired shift error `/ (n-1) / n` → `/ n` | survived | killed: `the_paired_shift_carries_the_standard_error_of_its_mean` |
| P12 | unknown argument ignored | survived | killed: `the_arguments_refuse_what_they_cannot_read` |
| P13 | `read_run`: skip the event-count refusal | killed: 2 tests | killed: 2 tests |
| P14 | `read_run`: skip the σ refusal | killed: `a_short_event_file_or_a_non_numeric_sigma_is_refused` | killed: same |
| P15 | `check`: skip the event-count refusal | killed: `a_record_refuses_a_short_seed_and_another_hard_survey` | killed: 3 tests |
| P16 | `part_reference`: a missing process reads another | killed: `one_process_is_read_from_its_own_share` (any panic) | killed: same, on the payload |

P6 and P7 ran on `read_seed`, which the split did not touch. P8 had to be
re-run as P8b because the split moved its site.

Shell: `refshy1-shcheck.sh` builds hand-made runs and runs `mes_check_run` on
copies of `madevent_seeds.sh` with one refusal disabled each. These copies live
in the scratchpad and are not committed tests (see Found):

| mutation of `mes_check_run` | case that flips | unmutated | mutated |
|---|---|---|---|
| `if n < nev:` → `if False:` | 3 events, card asks 5 | rc 1, "holds 3 events, the run card asked for 5" | rc 0 |
| positive finite σ → `if False:` | NaN σ; zero σ | rc 1 each | rc 0 each |
| gzip error → count as complete | truncated gzip | rc 1, "cannot read the event file" | rc 0 |

`mes_check_card` and `script_driver`: their before/after table is under Fixed.
The before column is the unmutated old code failing to refuse, so for these
two the old code is itself the mutation.

**Sites no mutation reaches:**
- `sorted_dirs`' entry error (H18). A test cannot make `read_dir` yield an
  entry error.
- The no-channel refusal (H6/P8) as a defect. A run with no channel already
  fails the channel-sum refusal, because σ > 0 is required first. P8b is killed
  only through the message the new assert pins.
- The overlap between `read` and `reference_of` (H2, H3). Each is now pinned by
  its own test, but at the gates' call sites (`reference`, `part_reference`)
  one would cover the other.

## Gate

Commits on `sigma/reference-scatter`:
- `5fd18e6`: the findings table, committed before the fixes.
- `9dcf9c7`: shell (H12, H13, H22, H24, H28).
- `9a54c7a`: writer and reader (H1–H11, H15–H21, H23, plus the ambiguous
  `[reference]` doc link `cargo doc` warned about at `REC:670`, which predates
  this session).
- This report.

| command | tail |
|---|---|
| `cargo fmt --all --check` | clean, rc 0 |
| `cargo clippy -j 2 --workspace --all-targets -- -D warnings` | `Finished`, rc 0 (the first run failed on 22 `needless_borrow`s in the new tests, which `clippy --fix` applied) |
| `cargo clippy -j 2 --workspace --all-targets --all-features -- -D warnings` | `Finished`, rc 0 |
| `CARGO_PROFILE_DEV_DEBUG=0 cargo test -j 2 --workspace -- --test-threads 2` | 39 suites, 1433 passed, 0 failed, 16 ignored, rc 0. `seed-record`: 31 passed (25 before; the 6 new tests are listed under Fixed). No test was deleted |
| `cargo test -p vibegraph-validation-report` | `seed-record` 31 passed; `validation-report` 4 passed |
| `cargo test -p vibegraph-lib --test seed_combination` | 4 passed (within the workspace run) |
| `cargo doc -p vibegraph-validation-report --no-deps --document-private-items` | 0 warnings (1 before, fixed) |
| `sigma_bb_fixed_scale_vs_mg`, `sigma_gate_matches_madgraph` (`--profile release-debug --features extended-validation`), no record present | `MG σ = 2143300.0 ± 3899.1 pb, pull = +1.16`; `ee_to_mumu MG = 2.025000e3 +- 8.427e-1, pull = +0.21`; both `ok`. These match F1's after-validate readings exactly: the inert path |
| the same, F2's dry-run records copied into `output/seeded_sigma/` | `MadEvent reference from seeded_sigma/pp_to_bb_fixed/seed_record.json … 2.148120e6 +- 4.753e3`, pull −0.01; `ee_to_mumu … 2.024640e3 +- 9.014e-1`, pull +0.40; both `ok`, identical to F2's report. The copies were removed after (`ls output/seeded_sigma`: no such file) |
| `python3 scripts/kb.py index --check`, `lint` | clean; lint `0 errors, 2 warnings`, both on unrelated performance backlog items |

The two σ gates are the banked targets this change reaches. Both read `REC`,
which changed only in docs and visibility. `validate_hadronic`'s other record
readers (`banked`, `seeded_reference`) and the grammar gate's are untouched,
and no record exists for their rows. `target/debug` was deleted after.

## Leave, with the reasons

- **H25: who owns the format.** The question was whether the reader's file or
  the writer should own the schema, checks and combination. It is already one
  copy: `BIN` includes `REC` and `seeds.rs` by `#[path]`, as the CLI tests
  include `seeds.rs`. To move it into `validation-report` as a library,
  `vibegraph-lib` would need a dev-dependency on the collator crate, for no
  gain over `#[path]`. `read_run`'s refusals overlap `check` for a one-window
  seed, but they are per window and name the file, so both stay.
- **H26: `seed-record.rs` size.** It is 1568 lines after this session: code
  766 → 804 lines (the `#[cfg(test)]` line moved from 767 to 805), the rest tests. It would split cleanly into parsers, run and seed
  reading, and printing. Each piece has one user, though, and after H15 no
  function passes 100 lines. A split moves code without removing any.
- **H27: shell structure.** Nothing wrong was found (see the table row).
- **`#![allow(dead_code)]` at `REC:56`.** This is the established pattern for
  every `vibegraph-lib/tests/common/*.rs`: `mod.rs`, `seeds.rs`, `leshouche.rs`,
  `report.rs` and `manifest.rs` all carry it. Each test target uses a subset of
  the module. It is not new debt from this PR.
- **`card_int` reads the banner through `f64`** (`BIN` `read_run`), so
  `hard_survey = 2.7` would read 2. MadGraph writes these keys as integers.
  Making it stricter would change a refusal's input domain with no case to
  motivate it.
- **`SeedRun::value` and `PartValues::value`** are the same four lines on two
  types. A trait would add more than it removes.

## Method

| technique | share | yield |
|---|---|---|
| Name each refusal and run a mutation that removes it (batch driver, before and after) | 45% | H1–H7, H11. 7 of 16 mutations survived the old tests |
| Grep each refusal's message against the tests | 10% | Found the same gaps; settled by the mutations |
| Hand arithmetic on the screen and summary tests' inputs (which pulls are all far or all zero) | 5% | H8, H9 |
| Hand-made shell inputs (`refshy1-shcheck.sh`), including the empty-set case | 15% | H12, H13, H14 |
| Read for length and in-file duplication, then grep workspace-wide for copies of each parser | 10% | H15–H17, H29 |
| `grep` for `filter_map(…ok())`, `catch_unwind`, `temp_dir()` | 5% | H10, H18, H19 |
| Identity oracles and the gates | 10% | — |

**Leads that did not hold up:**
- `build.sh`'s source guard and `set -e` when sourced: correct.
- An empty-set `mes_check_card` on today's scripts: every script compares at
  least 2 settings. The defect is latent, not live.
- `read_run`'s refusals as a duplicate of `check`: they are finer-grained, so
  they stay.
- The visibility point was low-yield, as `hygiene-review.md` predicts after a
  mechanical pass: two constants.

The first version of the shell harness printed `rc=0` for a refusing call. It
read `$?` after a `$(basename …)` in the same `echo`. It was caught because the
refusal message printed beside `rc=0`, and fixed before any reading was taken.
Exit status in shell harnesses needs capturing on its own line.

## Brief corrections

- **"`cargo test -p vibegraph-lib --test seed_combination`" does not test the
  reader.** The target compiles `common::seed_record` but has no test of it.
  Its four tests are `combine`'s. The reader's tests are in the `seed-record`
  binary (`cargo test -p vibegraph-validation-report`). F2's gate table implies
  the opposite ("hermetic (lib, includes `common::seed_record`)").
- **"add the test where none does" for `mes_check_run`.** The hermetic layer
  has no Python, and the repository has no shell-test harness. The refusals were
  shown by scratch mutation instead, and a committed test is in Found.
- **"`seed-record.rs` (1.3k lines)".** It was 766 lines of code and 571 of
  tests.

## Found

1. **No committed test for the driver's shell refusals**: `mes_check_run`,
   `mes_check_card` and `script_driver … no-launch` (H14).
   - Evidence: the scratch harness above is the only exercise of these
     refusals, and F2's forced failures were manual too.
   - To close it: a `validation/madgraph/tests/` script run in the `madgraph`
     pixi environment (and in CI where that environment exists). Or move the
     result and event-count check into `seed-record` or another Rust tool, where
     the hermetic layer tests it.
2. **Hand-written temp directories across `vibegraph-lib`** (`std::env::temp_dir()`
   plus pid, removed by hand, leaking on a failed test).
   - Sites: `vibegraph-lib/src/cache/resolve.rs:134`, `cache/pinned.rs:223`,
     `cache/store.rs:195`, `ufo/mod.rs:808`, `:845`, `:871`, and
     `artifact.rs:885`, `:926`, `:1072`, `:1113`.
   - To close it: a `tempfile` dev-dependency on `vibegraph-lib`, as
     `vibegraph-cli` and now `validation-report` have.
3. **Second copies of the writer's MadGraph-file parsers** (H29). `seed-record`
   is deliberately free of vibegraph crates, so none could be reused here.
   - Fortran `D` exponents: `BIN` `fortran_float` and
     `vibegraph-lib/src/lhef/parse.rs:136-141`.
   - `<MGRunCard>` `value = key` reader: `BIN` `banner_value` and
     `vibegraph-cli/tests/cli_generate_proton.rs:1143` `mg_run_card_fields`.
   - `results.dat` first line:
     - `BIN` `value_and_error`;
     - `vibegraph-lib/tests/validate_hadronic.rs:217` `banked_llj_sigma`;
     - `madevent_seeds.sh` `mes_run_seed` (awk) and `mes_check_run` (Python);
     - `gen_pta_windows.sh` `run_one`.
   - `symfact.dat` positive multiplicity: `BIN` `parse_symfact` and
     `bank_seeded_sigma.sh` `keep_run` (`awk '$2 > 0'`). The two must agree on
     which channels are kept and summed.
   - To close it: a small shared parser module (a `tests/common` file the
     writer includes by path, as it does `seed_record.rs`) for the Rust copies.
     The shell copies stay, but each needs the same rule.
4. **A further instance for the already-planned `mes_run_seed` filing.**
   `gen_pta_windows.sh` `run_one` refuses only a missing or empty
   `results.dat` (`[ -s … ]`), not a non-numeric first line or a short event
   file, when it is run on its own stages. Run through `bank_seeded_sigma.sh`,
   each window goes through `mes_check_run`.
