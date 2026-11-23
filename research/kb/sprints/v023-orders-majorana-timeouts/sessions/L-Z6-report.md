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
