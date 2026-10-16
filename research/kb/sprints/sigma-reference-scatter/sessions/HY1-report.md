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
| H1 | 2 | `REC:424` `check`, non-finite refusal | No test builds a record with a NaN or infinite value; `NaN <= 0.0` is false, so the positive-σ refusal does not stand in | `if false && values…is_finite()` (survived) | checked | PENDING |
| H2 | 2 | `REC:480` `reference_of` re-check | No unit test calls `reference()` on a record that fails `check` | delete the `check()` call (survived) | checked | PENDING |
| H3 | 2 | `REC:654` `read`'s `check` | `reference_of` re-checks, so deleting `read`'s check only drops the record's path from the panic; no test reads the message | delete the call (survived) | checked | PENDING |
| H4 | 2 | `BIN:448` windows that ran different `nevents` | No test; the seed takes `runs[0].nevents`, and `check` cannot see a window's own count | `if false && …` (survived) | checked | PENDING |
| H5 | 2 | `BIN:460` partitioned seed with several processes | No test | `if false && …` (survived) | checked | PENDING |
| H6 | 2 | `BIN:349` no channel listed | Unreachable as a defect: with no channel the sum is 0, and `read_run` has already refused σ ≤ 0, so the channel-sum refusal fires. Only the message differs | `if false && …` (survived) | checked | PENDING |
| H7 | 2 | `BIN:522` `parse_args` | No test of any argument refusal (unknown flag, missing value, missing flag, non-integer `--budget`/`--hard-survey`) | `--hard-survey` parse error → `unwrap_or(2)` (survived); unknown flag ignored (survived) | checked | PENDING |
| H8 | 2 | `REC:535` screen statistic | `the_screen_flags…` uses pulls that are all far off (≈ 40) or all zero, so the flag reads the same on the 4× mean's pull, the first seed's, or any seed's | `worst = pulls.first()` (survived) | checked | PENDING |
| H9 | 2 | `REC:515` paired EW − reported error | Only pinned at 0 (constant offset), where any normalisation gives 0 | `/ (n-1) / n` → `/ n` (survived) | checked | PENDING |
| H10 | 2 | `BIN:1144`, `BIN:1155` | `catch_unwind(…).is_err()` discards the payload: any panic passes (missing-process and wrong-row refusals) | — (the refusals themselves are reached: dropping the row assert is killed by `a_work_area_without_a_record_reads_none`, reading another process for a missing one by `one_process_is_read_from_its_own_share`) | checked | PENDING |
| H11 | 2 | `BIN:1107`, `:865`, `:1174`, `:1217` | Any-refusal asserts: `check().is_err()` for the schema, `parse_symfact("1\n").is_err()`, `parse_run_config(…).is_err()`, the missing `results.dat` | schema check removed (killed by `a_record_holding_a_seed_at_a_third_budget_or_twice_is_refused`: the record is otherwise valid, so the bare `is_err` holds today by construction, not by assertion) | checked | PENDING |
| H12 | 2 | `MG/madevent_seeds.sh` `mes_check_card` | Green on an empty set: with every launch setting skipped, or a script whose block starts `launch -f` (the sed range needs a bare `launch`), nothing is compared and it returns 0. `refshy1-shcheck.sh`: `check_card a.mg5 b_card.dat [ebeam1 nevents]: rc=0`, `check_card nolaunch.mg5 b_card.dat []: rc=0`. Today every one of the 64 scripts keeps ≥ 2 settings after `iseed nevents` are skipped | — | checked | PENDING |
| H13 | 2 | `MG/build.sh` `script_driver … no-launch` | A non-bare `launch` line is not cut, so the "never-run" template would be launched at generation: a `launch -f` script came back with its launch block, rc 0. All 64 committed scripts use a bare `launch` today | — | checked | PENDING |
| H14 | 2 | `MG/madevent_seeds.sh` `mes_check_run` | No committed test (shell + Python, outside the hermetic layer). Each refusal does fire on a hand-made run (`refshy1-shcheck.sh`: short, empty, NaN, zero, truncated gzip all rc 1; good rc 0) | see Mutations | checked | PENDING |
| H15 | 1 | `BIN:270-407` `read_run` | 137 lines, past the ~100-line rule with no named reason; the channel walk (`:308-363`) and the `<init>` split (`:365-392`) are separable | — | checked | PENDING |
| H16 | 1, 4 | `BIN:86-92`, `BIN:134-141` | `Channel::parse`'s first line and `parse_total` are the same parse (first two Fortran reals of the first line) | — | checked | PENDING |
| H17 | 1, 4 | `BIN:412-415`, `:424-430`, `:562` | The `s<digits>`/`w<digits>` directory test is written three ways (`strip_prefix`+`parse`, `strip_prefix`+`is_some_and`, `starts_with`+slice) | — | checked | PENDING |
| H18 | 2 | `BIN:263` `sorted_dirs` | `.filter_map(\|e\| e.ok()…)` swallows a `read_dir` entry error, which would drop a seed or a channel directory silently | no mutation can provoke an entry error in a test | checked | PENDING |
| H19 | 1 | `BIN:790` `scratch` | Hand-written temp directory (`temp_dir()` + pid), removed by hand at each test's end, so a failing test leaks it. `tempfile` is already in the workspace (`vibegraph-cli` dev-dependency) | — | checked | PENDING |
| H20 | 1 | `REC:53` module doc | Says the file depends on `std`, `serde`, `puruspe` and `seeds` only; `read` uses `serde_json` | — | checked | PENDING |
| H21 | 1 | `REC:390` `check` doc | Lists schema, distinct seeds, budgets, finite values; the body also refuses σ ≤ 0, a short event count and another `hard_survey` | — | checked | PENDING |
| H22 | 1 | `MG/bank_seeded_sigma.sh:239` | `run_fresh ROW SEED NEVENTS CARD PARAM` omits the sixth argument, `OVERRIDES` | — | checked | PENDING |
| H23 | 3 | `REC:208`, `REC:293` | `DEFAULT_RUN_CONFIG` and `SCREEN_PULL_LIMIT` are `pub` with no user outside the module (`grep` over `BIN`, `validate_sigma.rs`, `validate_hadronic.rs`: 0) | — | checked | PENDING |
| H24 | 4 | `MG/gen_pta_windows.sh:229-234` | The new `hard_survey` replace-or-append duplicates `mes_install_card` (`madevent_seeds.sh:85-97`), which the script already sources; `nevents`/`iseed` were already a second copy | — | checked | PENDING |
| H25 | 1, 4 | `REC` vs `BIN` (the brief's question) | One schema: `BIN` includes `REC` and `seeds.rs` by `#[path]`, so types, `check`, `combine` and the reading rule have one copy. `read_run`'s σ and event refusals overlap `check` for a one-window seed but are per window (a negative window can hide in a positive sum) and carry the path | — | checked | PENDING |
| H26 | 1 | `BIN` size, 1337 lines | 765 lines of code (parsers ~230, run/seed reader ~270, args ~50, printing ~180) and 570 of tests | — | checked | PENDING |
| H27 | 1 | shell: `set -euo pipefail`, quoting, the main guard, dead options | `bank_seeded_sigma.sh` and `gen_pta_windows.sh` set `-euo pipefail`; `madevent_seeds.sh` is sourced; `build.sh` keeps `set -e` and its guard `[ "${BASH_SOURCE[0]}" = "$0" ]` holds (`bash -c '. ./build.sh; type build_main'` printed nothing of the build). Unquoted `$SEEDS`/`$ROWS`/`$(group_rows …)` are deliberate word splits. Every documented variable is read | — | checked | PENDING |
| H28 | 1 | `MG/build.sh` `script_driver` | `$(mktemp -t vg_build_XXXXXX).mg5` leaves the empty `vg_build_XXXXXX` behind on every call (moved, not added, by this PR) | — | checked | PENDING |
| H29 | 4 | cross-module | Second copies of the writer's parsers outside the change (see Found) | — | checked | PENDING |

Backlog check: `grep -rlE 'bank_seeded_sigma|seed-record|seed_record|madevent_seeds\.sh|madgraph/build\.sh|gen_pta_windows|time_stages|gen_mlm_references\.sh|gen_onshell_veto\.sh|assemble_bundle' research/kb/backlog`
returns `validation/reference-sigma-seed-scatter-untested.md` (the item this PR
claims) and `validation/uux-wbwb-onshell-veto-tt-2pct-off.md` (names
`gen_onshell_veto.sh` for its row's physics). Neither names a site that is
wrong or already fixed.

The comment grep (`\bT[0-9]\b|\bC[0-9]\b|Phase [0-9]|note [0-9]+|used to|no longer`,
plus `D1|amendment|sprint|session`) over the added lines finds two hits, both
benign: "no longer sums to the total" describes the test's own setup, and "the
session's history" is MadGraph's session.
