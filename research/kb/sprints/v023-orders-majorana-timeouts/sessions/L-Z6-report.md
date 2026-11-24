---
type: Session Report
title: "L-Z6 report: hygiene pass on phase timing records and the run-global progress position"
description: "Findings, fixes with mutation evidence, and the identity oracle (σ, grid sha256 and the masked ldjson stream against main) for sprint L's phase instrument (#183) and progress position (#186)."
status: draft
tags: [hygiene, cli, ldjson, progress, phase-timing, non-vacuity, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-hyg-progress` off main 8b3af5ab. Scope: the squash commits
`abc3b31e` (#183, phase timing) and `b4e284a3` (#186, progress position), read
at the branch's base, where #185 has since reworked the surrounding compile
code in `multiplicity.rs` and `integrate.rs`.

## Findings

Written and committed before any code edit. Ranked by what a fix buys.
*Suspected* findings name the run that settles them; the results are under
Mutations.

| id | point, class | site | what is wrong | evidence | confidence | fix here? |
|---|---|---|---|---|---|---|
| Z6.1 | 2, vacuous test | `vibegraph-cli/src/phase.rs:257` (`elapsed_time_never_runs_backwards`), `vibegraph-cli/tests/cli_phase_timing.rs:94-96` | Nothing pins `elapsed_s`/`wall_s` to a real clock. The unit test asserts `b >= a && a >= 0.0`, which a constant passes; the CLI order test asserts `t0 >= last && t1 >= t0` and `wall == t1 - t0`, which a constant 0 also passes. | quoted lines; mutation M1 (`elapsed()` returns `0.0`) is expected to pass the whole phase suite | suspected, settled by M1 | yes: delete the unit test (the CLI test owns the property), and assert in the fixed-beam test that the `vegas` phase's `wall_s` is positive and at least its own `points` × 1 ns |
| Z6.2 | 2, untested call-site argument | `vibegraph-cli/src/integrate.rs:1026-1030` | `phase::plan(&integrate_phases(sets.is_none(), …))`: no test integrates a diagram artifact under `--output-ldjson`, so the `enumerates` argument is never exercised; the unit test exercises `integrate_phases(false, …)` but not the call site. | `grep -rn '"enumerate"' vibegraph-cli/tests/cli_phase_timing.rs vibegraph-cli/tests/cli_progress_position.rs` shows only proc-card runs; mutation M2 (`sets.is_none()` → `true`) is expected to pass | suspected, settled by M2 | yes: a CLI test that enumerates `e+ e- > mu+ mu-` to an artifact, integrates it with `--check`, and checks the position against the phase records (4 phases, no `enumerate`) |
| Z6.3 | 4, missing abstraction | `vibegraph-lib/src/multiplicity.rs:287`, `:359`, `:849` | A card's groups are numbered three times by hand: `Group { index: k as u64 + 1, count, label: labels[k], channels }` in `multiplicity_groups_of_with` and `MultiplicitySum::adapt_alphas`, and `Group { index: 1, count: 1, label: group_labels(..).remove(0), .. }` in `sole_group`. | quoted lines | checked | yes: one `numbered_groups(keys)` that labels and numbers, used by all three |
| Z6.4 | 3, public item that should be private | `vibegraph-lib/src/progress.rs:104-107` | `Group`'s four fields are `pub`, but every constructor and reader is inside the library (`multiplicity.rs`, `progress.rs`); the CLI only passes a `Group` from `sole_group` into `enter_group`. | `grep -rn 'Group {' vibegraph-cli/ vibegraph-lib/` (outside `progress.rs` and `multiplicity.rs`: none) | checked | yes: `pub(crate)` fields |
| Z6.5 | 1, dead or duplicated code | `vibegraph-cli/src/integrate.rs:1587` | `part_process_ids(groups)` is a one-line forward to `FlavorGroups::process_ids`, which #186 added with the same doc; one caller (`:1831`). | `grep -rn part_process_ids` returns the definition and `:1831` | checked | yes: call `groups[k].process_ids()` and delete the wrapper |
| Z6.6 | 1, duplicated code | `vibegraph-cli/src/phase.rs:180-184`, `:229-239` | `start` and `finish` each build the `event`/`phase`/`elapsed_s` map before `emit_object("phase", …)`. | quoted lines | checked | yes: one `record(event, name, elapsed_s)` |
| Z6.7 | 4, hand-kept list (the brief's lead) | `vibegraph-cli/src/phase.rs:125-150` | `integrate_phases`/`generate_phases` restate the order of the `phase::start` calls. It cannot be derived from the phases the run enters: `phases` is written on progress objects *before* those phases run, so it is a forecast by construction. What a derivation would buy, a forecast that cannot drift from the calls, is had instead from the CLI tests that compare `phases` with the start records the run goes on to write (fixed-energy, `--check`, proton two-group, `generate`; Z6.2 adds the artifact input). `plan` reads only the list's length; the order is pinned by `cli_phase_timing.rs`. | quoted lines; `cli_progress_position.rs:180-215` | checked | leave (Z6.2 closes the uncovered argument) |
| Z6.8 | 1, duplicated code (the brief's second lead) | `vibegraph-cli/src/phase.rs`, `vibegraph-cli/src/logging.rs:263-291` | Not reproduced as duplication between phase and progress: a `progress` object is the tracing JSON formatter's event re-parsed in `LdjsonEvent::drop` and stamped by `phase::annotate`; a `phase` object is built by hand. The two share `output::emit_object` and nothing else. The duplication there is is inside `phase.rs` (Z6.6). | quoted lines | checked | not reproduced |
| Z6.9 | 2, blind spot | `vibegraph-cli/src/output.rs:104`, `cli_phase_timing.rs:252` | The kill test cannot see the explicit `flush()` in `emit_object`: `std::io::Stdout` is line-buffered whatever it points at, so a `\n`-terminated `write_all` reaches the file without it. | mutation M3 (drop the `flush`) is expected to pass | suspected, settled by M3 | leave: named under "What each test cannot detect" |
| Z6.10 | 2, blind spot | `vibegraph-cli/src/phase.rs:294-312` (`phase_names_are_distinct`) | The test lists every variant by hand, so a variant added and left out of the list is unchecked. Enumerating an enum's variants needs a derive crate for one test. | quoted lines | checked | leave |
| Z6.11 | 1, duplicated code across test files | `cli_phase_timing.rs:66`, `cli_progress_position.rs:22`, `cli_run_card.rs:21`, `cli_check_only.rs:65`, `cli_enumerate_progress.rs:26` | Five copies of "parse every stdout line as JSON". | `grep -rn 'serde_json::from_str(l)' vibegraph-cli/tests` | checked | Found (cross-file) |
| Z6.12 | 1, duplicated code across test files | `cli_progress_position.rs:67` (`write_pdf_set`), `cli_generate_vanishing_group.rs:56` (`synthetic_pdf_set`), `vibegraph-lib/src/systematics/plan.rs:702` | Three hand-written `lhagrid1` stand-in sets. | `grep -rn lhagrid1 --include=*.rs` | checked | Found; the filed item `proton-phase-timing-test-needs-fetched-pdf` is the natural place to share one |
| Z6.13 | — | `multiplicity.rs` (compile `groups`) | Known gap (a) of the L-Y1 report, filed as `compile-and-alpha-survey-group-counts-differ`. Closing it means knowing before `derive_flavor_groups_with` runs whether it will refuse with `OneBodyCutToZero`: not a few lines. | filed item | checked | leave |

Comment rules: `git show abc3b31e b4e284a3 -- '*.rs' docs | grep '^+' | grep -E '\bT[0-9]\b|\bC[0-9]\b|Phase [0-9]|note [0-9]+|used to|no longer|sprint|PR [0-9]|session|backlog|item'`
returns nothing. No `unwrap`/`expect` was added on a fallible path:
`group_labels(..).remove(0)` takes the one label of a one-element input.

**Backlog check.** Items naming a reviewed path:
`hygiene/proton-phase-timing-test-needs-fetched-pdf` (filed, its site is
right), `hygiene/budget-test-keeps-private-capture-subscriber` (names
`progress.rs` as already fixed; true), and the gap (a) item above. None is a
finding.

**Identity oracle**, chosen before the first edit: σ, its error and its bits,
and `sha256sum out/grid.bin.zst`, at `--fixed-budget --neval 3000 --niter 3
--seed 5 -j 1`, on `e+ e- > mu+ mu-` (45.6 GeV beams) and the MLM card
(`p p > e+ e-` + `add process p p > e+ e- j`, `pp_to_llj_mlm_run_card.dat`,
model `sm`), each run plain and under `--output-ldjson`; and the
`--output-ldjson` stream with `timestamp`, `elapsed_s` and `wall_s` deleted
at every depth (`jq -c 'walk(if type=="object" then del(.timestamp,
.elapsed_s, .wall_s) else . end)'`), compared byte for byte.

The findings table above is as committed in 7a1bf90b, before any code edit.
The oracle's mask grew twice after the choice, both times because two runs of
the *same* baseline binary differed, never to hide a difference between base
and fix: see Oracles.

## Final dispositions

| id | disposition | evidence |
|---|---|---|
| Z6.1 | fixed | M1 passed every phase test before, fails `a_fixed_beam_integration_streams_its_phases_in_order` after |
| Z6.2 | fixed | M2 passed both CLI files before, fails the new `an_artifact_run_plans_no_enumeration` after |
| Z6.3 | fixed | identity oracle; M9, M12, M16 sites now go through `numbered_groups` and the tests that caught them pass |
| Z6.4 | fixed | builds; no reader outside the crate |
| Z6.5 | fixed | identity oracle (`process_ids` in the result is in the masked stream) |
| Z6.6 | fixed | identity oracle (the `phase` records are in the masked stream) |
| Z6.7 | leave | the forecast is pinned end to end on five run shapes after Z6.2 |
| Z6.8 | not reproduced | as stated in the table |
| Z6.9 | leave | M3 passes (checked): see What each test cannot detect |
| Z6.10 | leave | as stated |
| Z6.11, Z6.12 | Found | below |
| Z6.13 | leave | filed item |

## What changed

- `vibegraph-cli/src/phase.rs`: `record(event, name, elapsed_s)` builds the
  three fields every `phase` record shares (Z6.6). The unit test
  `elapsed_time_never_runs_backwards` is deleted (Z6.1): it passed a clock
  stuck at 0.
- `vibegraph-cli/tests/cli_phase_timing.rs`: the fixed-beam test asserts the
  `vegas` phase's `wall_s` is at least its own `points` × 1 ns. The floor is
  a physical one (no integrand evaluation takes under a nanosecond), not a
  reading: the dev-profile runs here log 973–44 653 ns/eval, so it sits three
  orders of magnitude below the slowest real run's need and cannot flake.
- `vibegraph-cli/tests/cli_progress_position.rs`: new
  `an_artifact_run_plans_no_enumeration`, enumerate `e+ e- > mu+ mu-` to a
  diagram artifact, integrate it with `--check`: `phases` is 4 and the phases
  started are exactly `input`, `amplitude_probe`, `compile`, `channels`
  (Z6.2).
- `vibegraph-cli/src/integrate.rs`: `part_process_ids` deleted; the one
  caller reads `groups[k].process_ids()` (Z6.5).
- `vibegraph-lib/src/multiplicity.rs`: `numbered_groups(keys)` labels and
  numbers a card's groups; `multiplicity_groups_of_with`,
  `MultiplicitySum::adapt_alphas` and `sole_group` use it (Z6.3).
- `vibegraph-lib/src/progress.rs`: `Group`'s fields are `pub(crate)` (Z6.4).

Tests: +1 (`an_artifact_run_plans_no_enumeration`), −1
(`phase::tests::elapsed_time_never_runs_backwards`); the suite count is
unchanged.

## Mutations

Each mutation was a gate `std::env::var("LZ6_MUT").as_deref() == Ok("<id>")`
inserted by an assert-checked patch script, so one build served the batch;
the driver ran each id's tests with `LZ6_MUT=<id>` under the heavy lock and
logged the `test result` line (`/home/user/wt/logs/lz6-mut-pre3.log`,
`lz6-mut-pre4.log` after a container restart, `lz6-mut-post.log`). Every gate
was removed before any commit: `grep -rn LZ6_MUT --include=*.rs vibegraph-cli
vibegraph-lib` returns nothing (rc 1).

| id | mutation | guards | before fix | after fix |
|---|---|---|---|---|
| M0 | none | all targets | all pass (5, 5, 3, 7, 1) | all pass (5, 6, 2, 7, 15 in `multiplicity::tests`) |
| M1 | `elapsed()` returns `0.0` | Z6.1 | **passes** `phase::`, `cli_phase_timing`, `cli_progress_position` | fails `a_fixed_beam_integration_streams_its_phases_in_order` (`cli_phase_timing.rs:145`, the new floor) |
| M1b | `elapsed()` measured from a fresh `Instant` per call | the origin | fails the unit test and `a_fixed_beam` (`:94`, `t0 >= last`) | fails `a_fixed_beam` (`:94`) |
| M2 | `integrate_phases(sets.is_none() \|\| true, ..)` | Z6.2 | **passes** both CLI files | fails `an_artifact_run_plans_no_enumeration` ("the plan is the phases the run entered") |
| M3 | `emit_object` skips `flush()` | kill test | **passes** `a_run_killed_in_a_phase_leaves_its_start_record_on_disk` | not re-run (left, Z6.9) |
| M4 | `wall_s` = `now` | `a_fixed_beam` | fails (`:96`) | — |
| M5 | fixed-energy `--check` does not stop | `a_check_run_stops_after_the_channels` | fails (`:82`, record count) | — |
| M6 | a dropped guard ends `ok` | `a_refusal_ends_its_phase_as_failed` | fails (`:244`) | — |
| M7 | no `start` record for `vegas` | `a_run_killed_in_a_phase` | fails (60 s deadline) | — |
| M8 | proton `flavour_groups` size 0 | `a_proton_integration_streams_its_phases_in_order` (extended-validation) | fails | — |
| M9 | `sole_group` drops `channels` | `a_single_group_run_reports_group_one_of_one` | fails | — |
| M10 | plan ignores `--check` | `a_check_run_plans_the_phases_it_executes` | fails | — |
| M11 | `generate` never plans | `a_generate_run_carries_its_three_phases` | fails | — |
| M12 | survey `channels` + 1 | `a_two_group_proton_run_numbers_its_groups` | fails | — |
| M13 | `step` adds a field `extra` | `the_extras_leave_the_existing_fields_and_order_alone` | fails | — |
| M14 | a dropped `GroupScope` restores nothing | `progress::tests::a_step_in_a_group_carries_the_groups_fields` | fails | — |
| M16 | any non-empty tag set labels by tag | `multiplicity::tests::group_labels_are_tags_unless_the_tags_repeat` | fails | — |
| M17 | `integrate_phases` drops `pdf` | `phase::tests::the_planned_phases_follow_the_run_s_mode` | fails | — |
| M18 | `bank` prints as `vegas` | `phase::tests::phase_names_are_distinct` | fails | — |

No one-line mutation reaches `a_group_does_not_follow_work_onto_other_threads`:
what it pins is that `GROUP` is a `thread_local!`, a declaration, and the
other tests' mutations of `in_group` are caught by
`a_step_in_a_group_carries_the_groups_fields` first.

## Oracles

**Identity, main against the fixed branch.** The branch before the first
edit differs from main 8b3af5ab only by the brief and the findings
(`git diff --stat 8b3af5ab 7a1bf90b`: two `.md` files under `research/kb`),
so its dev build is main's code. It was built once, before any edit, and
kept as `/home/user/wt/lz6-main/vibegraph-base`; the fixed tree's build is
`vibegraph-fix`. Driver `/home/user/wt/lz6-main/oracle.sh <binary> <tag>`,
each card run plain (`--no-tui`) and under `--output-ldjson`, with
`--fixed-budget --neval 3000 --niter 3 --seed 5 -j 1 --no-network` and
`LHAPDF_DATA_PATH` at the worktree's `validation/pdf`:

```
base  ee  plain  grid_sha256=b7dc695591b2ac1569e1e1985bfc171daaa1a2870391283fd90343342cf921fe
base  ee  ldjson grid_sha256=b7dc6955…f921fe sigma=2029.545411444088 err=8.92604215347058
base  mlm plain  grid_sha256=3b2479a0243231886a9a9e160e30547c552b7cf0e21f3a9b160be909c60d70cf
base  mlm ldjson grid_sha256=3b2479a0…d70cf sigma=937.0178812779731 err=28.934146865202155
fix   ee  plain  grid_sha256=b7dc695591b2ac1569e1e1985bfc171daaa1a2870391283fd90343342cf921fe
fix   ee  ldjson grid_sha256=b7dc6955…f921fe sigma=2029.545411444088 err=8.92604215347058
fix   mlm plain  grid_sha256=3b2479a0243231886a9a9e160e30547c552b7cf0e21f3a9b160be909c60d70cf
fix   mlm ldjson grid_sha256=3b2479a0…d70cf sigma=937.0178812779731 err=28.934146865202155
$ diff <(base2 summary minus masked hash) <(fix summary minus masked hash)   # empty
sigma/err/grid identical base2==fix
```

σ is compared as its JSON shortest round-trip text, which is equal exactly
when the bits are (`2029.545411444088` is `0x409fb62e80566ce3`,
`937.0178812779731` is `0x408d48249ef080d7`). Both grid hashes are the ones
L-Y1 recorded at abc3b31, so #184/#185/#188 moved neither card.

**The masked stream.** The mask is `/home/user/wt/lz6-main/mask.jq`:

```
walk(
  if type == "object" then del(.timestamp, .elapsed_s, .wall_s, .ns_per_eval, .vibegraph_version)
  elif type == "string" then gsub("[0-9]+\\.[0-9]+ s\\b"; "<t> s") | gsub("[0-9]+ ns/eval"; "<n> ns/eval")
  else . end)
```

applied as `jq -c -f mask.jq stdout > <card>.masked.ldjson`. The brief's
three fields were not enough: two runs of the same baseline binary (`base`,
`base2`) still differed in the vegas progress field `ns_per_eval` and in log
messages carrying timings (`phase vegas: 0.057 s (...)`, `3000 points at 1031
ns/eval`). `vibegraph_version` was added when base and fix differed only in
it: `v0.2.2-29-g7a1bf90b` against `v0.2.2-29-g7a1bf90b-dirty`, the build
string of an uncommitted tree. With the mask:

```
ee  base==base2 masked 3bf3129dfbcbdaeb records=67
ee  base==fix   masked 3bf3129dfbcbdaeb records=67
mlm base==base2 masked 0da611fce17d8133 records=173
mlm base==fix   masked 0da611fce17d8133 records=173
```

(`cmp` of the files, then the first 16 hex digits of their sha256.) The
stream holds every `phase` record, every `progress` record with its group
fields and position, and the `result` with its `multiplicities`'
`process_ids`, which is what Z6.3, Z6.5 and Z6.6 could have moved.

## What each test cannot detect

- **The `vegas` wall floor** catches a stuck or per-call clock, not a clock
  that runs at the wrong rate (a `elapsed_s` in milliseconds would still clear
  1 ns per point). Nothing pins the origin to the top of `main`: without
  `mark_origin`, the first phase fixes it, and `input`'s `elapsed_s` reads a
  few milliseconds low.
- **The kill test** cannot see the explicit `flush()` in `emit_object` (M3
  passes): `std::io::Stdout` is a `LineWriter` whatever it points at, so each
  `\n`-terminated `write_all` reaches the file anyway. The flush is
  belt-and-braces for a future `Stdout` change, and no test of this file can
  hold it.
- **`phase_names_are_distinct`** is blind to a variant missing from its own
  list.
- **The artifact test** pins the plan for a fixed-energy artifact under
  `--check`; a proton artifact run (`enumerates` false, `proton` true) is the
  sum of two shapes each pinned elsewhere, not run.
- **The identity oracle** is two cards at one seed, `-j 1`, dev profile. It
  cannot see `generate`, decays or a dropped single-particle part; the edits
  touch none of their data, which is an argument, not a measurement.

## Gates

On the final tree (every gate removed), dev profile with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`,
each cargo command under `flock /home/user/wt/heavy.lock`:

```
$ /tmp/claude-0/lp/verify-hidden.sh /home/user/wt/sprintl-hyg-progress lz6
  # research/refs/mg5amcnlo, validation/madgraph/output and
  # validation/pdf/NNPDF23_lo_as_0130_qed moved aside, restored on exit
FMT_OK
CLIPPY_RC=0                       # cargo clippy --workspace --all-targets -- -D warnings
CLIPPY_EV_RC=0                    # same, --features vibegraph/extended-validation,vibegraph-lib/extended-validation
TEST_RC=0                         # cargo test --workspace --no-fail-fast
lines=79 passed=2352 failed=0     # an_artifact_run_plans_no_enumeration ... ok is in it
$ cargo doc -p vibegraph --no-deps --document-private-items
  Generated .../target/doc/vibegraph/index.html          # no warnings
$ cargo doc -p vibegraph-lib --no-deps --document-private-items
  warning: `vibegraph-lib` (lib doc) generated 13 warnings
  # all in coupling/, helas/eval/, lhef/, diagrams/squared.rs, phasespace/channel.rs:
  # none in a file this pass touched
$ cargo test -p vibegraph --features vibegraph/extended-validation,vibegraph-lib/extended-validation \
    --test cli_phase_timing --test cli_progress_position
test result: ok. 5 passed; 0 failed     # cli_phase_timing, the proton test included
test result: ok. 6 passed; 0 failed     # cli_progress_position
$ pixi run --skip-deps kb-lint
1394 concepts, 480 backlog items; 0 errors, 7 warnings   # warnings on other items
```

The two extended-validation targets are the ones these PRs added; the
proton phase test reads the fetched NNPDF set, so it runs only there.

## Long-run requests

None.

## Method

| technique | share | yield |
|---|---|---|
| Reading both squash diffs at the branch base (where #185 has moved the code) | 25% | Z6.3, Z6.5, Z6.6, Z6.7 |
| One mutation per new test, batched behind env-var gates, one build | 35% | Z6.1, Z6.2, Z6.9 confirmed; 15 tests shown non-vacuous |
| Grep a definition / pattern workspace-wide | 10% | Z6.4, Z6.5, Z6.11, Z6.12 |
| Identity oracle, including proving the mask on base against base | 20% | two mask fields the brief's mask missed |
| Backlog check by path tail | 5% | nothing new |

Leads that did not hold: "derive `phases` from the phases run" (Z6.7, it is a
forecast), "duplicated record-building between phase and progress" (Z6.8).
M1b, which looked like a second blind spot of the clock tests, is already
caught by `t0 >= last`: a per-call origin jitters backwards within
nanoseconds.

## Found

- **Five copies of an ldjson stdout parser** in the CLI tests (Z6.11):
  `cli_phase_timing.rs:66`, `cli_progress_position.rs:22`,
  `cli_run_card.rs:21`, `cli_check_only.rs:65`, `cli_enumerate_progress.rs:26`.
  Closing: one `common::ldjson_objects(stdout)` with its `allow(dead_code)`,
  used by all five.
- **Three hand-written `lhagrid1` stand-in sets** (Z6.12):
  `cli_progress_position.rs` (`write_pdf_set`),
  `cli_generate_vanishing_group.rs:56` (`synthetic_pdf_set`), and
  `vibegraph-lib/src/systematics/plan.rs:702`. Closing: one writer in
  `tests/common`, which is also what the filed
  `proton-phase-timing-test-needs-fetched-pdf` needs to move the proton phase
  test into the default suite.
- **The brief's mask is not an identity oracle on its own**: `ns_per_eval`,
  timing text in log messages and the `-dirty` version string differ between
  runs of one build. The kb's hygiene procedure, or the sprint's brief
  template, should carry the extended mask and the base-against-base check.

## Brief corrections

- **"Build main in a separate detached worktree."** Not needed here: the
  branch before the first edit is main's code plus two `.md` files, so the
  baseline is the branch's own pre-edit build, kept as a copy of the binary in
  `/home/user/wt/lz6-main`. That saved a full build on a contended lock.
- **The mask** of `timestamp` and `elapsed_s` alone does not make two runs of
  one binary identical (see Oracles).
- **The `phases` lead** describes a forecast as a derivable count (Z6.7).
- **The duplicated-record lead** is not reproduced (Z6.8).
- **Feature names**: `cargo test -p vibegraph-lib` takes
  `--features extended-validation`; the brief's
  `vibegraph/extended-validation,vibegraph-lib/extended-validation` form is
  for `-p vibegraph`.
