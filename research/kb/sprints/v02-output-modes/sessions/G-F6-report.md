---
type: Session Report
title: "G-F6 report: output modes, --output-ldjson schema 1, ten error kinds, artifact format 20, vibegraph inspect"
description: "One output mode per run (plain, rich, ldjson) formats logs, progress and the result or refusal. --output-ldjson streams kind-tagged objects with schema_version 1 on every line; CliError carries one of ten stable kinds; artifact format 20 records per-multiplicity totals and refuses 19; integrate, generate and inspect emit final objects (inspect's equal to integrate's on a run); check-events refuses the flag; the survey reads the stream and still parses text. Every gate green; ten of ten mutations caught; the new tests fail on the pre-change tree."
status: draft
tags: [cli, output, ldjson, tui, tracing, artifact, cms-survey, sprint-g, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
measured:
  host: "cloud container, linux-64, 2026-11-05, one job at a time"
  commit: "75e6854 (feature) + e06b541 (banked-test fix), on a0be63f"
  command: "gate scripts and logs under the session scratchpad, prefixed G-F6-"
sources:
  - {id: brief, resource: "G-F6.md", title: "G-F6 brief"}
  - {id: design, resource: "../../../tooling/cli-output-modes.md", title: "CLI output modes, as built"}
  - {id: output, resource: "../../../../../vibegraph-cli/src/output.rs", title: "the mode, the schema and the final objects"}
  - {id: tests, resource: "../../../../../vibegraph-cli/tests/cli_output_ldjson.rs", title: "the schema tests"}
---
## What was built

Commits on `sprintg-output-modes-f6`: `75e6854` (the feature, docs, survey,
design concept), `e06b541` (the two banked tests read `inspect`'s result off
its last line), and this report.

1. **One mode per run.** `main` picks `output::Mode` (`Plain`, `Rich`,
   `Ldjson`) once; `logging::init` takes a `logging::Sink` built from it.
   `--output-ldjson` is a global `LogArgs` switch, `conflicts_with = "tui"`;
   `wants_tui` is false under it. Text result lines go through
   `output::text_line`, which drops them in ldjson mode; `output::result`
   emits the final object only in ldjson mode; the refusal replaces the
   `error!` line in ldjson mode.
2. **The stream** (schema in `output.rs`'s module docs, the CLI overview's
   "Output modes" section, and the design concept). Every stdout line is a JSON
   object with `kind` (`log` | `progress` | `result` | `refusal`) and
   `schema_version: 1`. `log` and `progress` are `tracing-subscriber`'s `json()`
   formatter output (`timestamp`, `level`, `target`, `fields`, `span`),
   restamped by `logging::LdjsonEvent` via `serde_json`; nothing is hand-written.
   The version is on every line rather than on a header: a filtered, truncated
   or mid-stream read still carries it on each object it holds.
3. **Error kinds** (`error::ErrorKind`, serialized snake_case): `usage`,
   `card`, `model`, `process`, `pdf`, `network`, `support`, `artifact`, `io`,
   `run`. Set at every `err(...)` site (the signature is now
   `err(kind, msg)`); PDF failures are classified from `SelectFailure` /
   `LookupError` by `error::pdf_error_kind`; `generate`'s library refusals map
   by variant (rounding → `usage`; reparsed process, changed model, changed
   collider energy → `artifact`; `Other` → `run`). Stability rule: a kind is
   never renamed or re-meant; adding or removing one bumps `SCHEMA_VERSION`.
4. **Artifact format 20**: `IntegrateArtifact::multiplicities:
   Vec<MultiplicityTotal>` (`final_state`, `process_ids`, `final_masses`,
   `channels`, `sigma_pb`, `sigma_err_pb`, `chi2_per_dof`). Version 19 is
   refused by version, tested on its real field layout
   (`a_version_19_artifact_is_refused`, with a byte-length check that the
   legacy shape differs from 20 by exactly the empty list's `u64` length).
5. **Final objects.** `integrate`: `output::IntegrateResult` — `process`,
   `model {label, digest}`, `observable`, `unit`, `sigma`, `sigma_err`,
   `chi2_per_dof` (null below two iterations), `mu_f` (tagged:
   `no_densities` | `constant {gev}` | `per_beam {gev: [a, b]}` | `per_event
   {dynamical_scale_choice}`), `seed`, `multiplicities`, `channels` (`key`
   tagged by `type`, `alpha`, `neval`, `sigma`, `sigma_err`, `chi2_per_dof`),
   `artifact`, `format_version`. Built from the artifact after it is written
   (`integrate::result_object`). `generate`: `output::GenerateResult` —
   `events_written`, `events_drawn`, `output`, `artifact`, `observable`,
   `unit`, `sample_sigma`, `sample_sigma_err`, `integration_sigma`,
   `integration_sigma_err`, `pull` (`vibegraph::generate::sample_pull`, which
   the library's own log line now uses too; `EmitSummary::sample_sigma_pb` is
   now `pub`).
6. **`vibegraph inspect <grid.bin.zst>`** (`inspect.rs`): text lines (process,
   model, the `σ = …` result line, χ²/dof, scale, seed, the per-multiplicity
   lines when there are several, one line per channel, path and format), or
   under `--output-ldjson` the same `result` object.
7. **`check-events --output-ldjson`** is a clap `ArgumentConflict` usage error
   (exit 2) raised in `main` before anything runs.
8. **The survey**: `vgrun.py` passes `--output-ldjson` in place of
   `--no-tui`; `parse.parse_vg_output` reads the objects when the output holds
   any (`parse_vg_ldjson`: σ, χ²/dof, channel count, multiplicities and model
   from `result`; outcome, `error_kind` and reasons from `refusal`; evaluation
   and diagram counts from the `log` messages; a panic from the non-JSON lines
   of the merged stderr; an unknown `schema_version` is an `error` outcome),
   and the summary lines otherwise. Tests cover both, including that one run
   reads alike in both formats.
9. **Docs**: `docs/src/cli/overview.md` (four commands; "Output modes"),
   `docs/src/cli/reference.md` regenerated with `inspect` added to
   `scripts/gen-cli-docs.sh` and `cli_reference_docs`'s command list, and
   `research/kb/tooling/cli-output-modes.md` rewritten from proposal to what
   was built (plus its regenerated tooling index).

## Deviations, and corrections to the brief

- **`serde_json` was not "already a dependency of the CLI"**: it was a
  dev-dependency. It is now a normal dependency, with `float_roundtrip`. Without
  that feature serde_json's *reader* is off by one ulp on some values: the first
  schema run read `2025.6705859029714` for the written `2025.6705859029712`
  (`G-F6-t2.log`). The writer was always shortest-round-trip; the feature makes
  a Rust consumer (the tests) read back the bits that were written. Python's
  `json` is exact already.
- **Progress units.** The brief says values in pb. The final objects are; the
  `progress` objects carry the field contract verbatim, so VEGAS `sigma`/`err`
  are in natural units (GeV⁻², or GeV for a width). Converting needs the
  observable, which the progress layer does not have. Documented in the schema.
- **Per-multiplicity totals are recorded for every proton run**, one entry for a
  single phase space, though `integrate` computed and printed them only for
  two or more; fixed-energy and decay runs record none. The text modes still
  print the lines only for two or more, and the survey records them only then,
  so its rows match older campaigns.
- **μF in the final object is reconstructed**, not newly recorded: from the
  artifact's `inputs.pdf` and `run_card` by `FactorizationScale::of`, the
  function the run used (`FactorizationScale::recorded_in`). The banked DY test
  pins all three non-trivial shapes through `inspect`.
- **`inspect --output-ldjson` streams `log` objects too** (parsing the recorded
  proc card logs at INFO); its `result` object is the one integrate emitted.

## Gates (final state, commit e06b541)

Commands run by `G-F6-gates.sh` and `G-F6-gates2.sh` with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

| command | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets --jobs 3 -- -D warnings` | exit 0 |
| `cargo clippy --workspace --all-targets --features vibegraph/extended-validation --jobs 3 -- -D warnings` | exit 0 |
| `cargo test --workspace --jobs 3` | exit 0; 55 result lines, 1931 passed, 0 failed, 25 ignored (incl. `cli_output_ldjson` 9/9); rerun on the clean tree after the mutations (`G-F6-final-test.log`), same counts |
| `cargo test -p vibegraph --profile release-debug --features extended-validation --test cli_integrate` | `ok. 6 passed; 0 failed` |
| `… --test cli_generate_proton` | `ok. 11 passed; 0 failed; 2 ignored` |
| `… --test cli_hard_errors` | `ok. 13 passed; 0 failed` |
| `… --test cli_param_card` | `ok. 8 passed; 0 failed` |
| `python3 -m unittest discover -s validation/cms_gridpacks -p 'test_*.py'` | `Ran 30 tests … OK` |
| `python3 scripts/kb.py lint` | `0 errors, 5 warnings` (the five are pre-existing backlog warnings) |

The first banked pass failed `cli_integrate` and `cli_generate_proton` on the
tests' own reading of `inspect` (they parsed the whole stdout as one object;
`trailing characters`); `e06b541` reads the last line, and both pass. The
multiplicity assertions ahead of that line had already passed in the first run.

## Mutation evidence

`G-F6-mutate.py` (log `G-F6-mutations.log`): apply, run the catching test,
revert. All ten caught.

| mutation | caught by |
|---|---|
| M1 no `schema_version` on a line | 7 of 9 `cli_output_ldjson` tests |
| M2 ldjson streams no progress | `integrate_streams_logs_progress_and_one_result`, `progress_is_streamed_whatever_the_level`, `the_generate_result_carries_the_sample_and_its_pull` |
| M3 result drops χ²/dof | `the_integrate_result_carries_the_recorded_values` |
| M4 ldjson refusal printed as an `error!` line | `a_refusal_is_the_final_object_with_its_kind`, `refusals_carry_the_kind_of_what_failed` |
| M5 missing `--seed` classed `run` | `refusals_carry_the_kind_of_what_failed` |
| M6 `check-events` accepts the flag | `the_flag_is_refused_with_tui_and_by_check_events` |
| M7 text result lines kept in ldjson | 5 tests (non-JSON line on stdout) |
| M8 pull without the integration's error | `the_generate_result_carries_the_sample_and_its_pull` |
| M9 oldest readable version lowered to 19 | `artifact::tests::a_version_19_artifact_is_refused` |
| M10 progress filtered by the visible level | the three progress-reading tests |

Pre-change: `cli_output_ldjson.rs` copied onto a `git archive a0be63f` tree
and run there (`G-F6-pre.log`): `test result: FAILED. 0 passed; 9 failed`, each on `error: unexpected argument '--output-ldjson' found` (or the missing `inspect` subcommand).

## For downstream sessions

- `#89` also bumps the artifact format; this branch is at 20 with 19 refused.
  Renumber at merge: the new `MultiplicityTotal` field and the
  `a_version_19_artifact_is_refused` test move with whichever lands second.
- A consumer of `progress` must not read VEGAS `sigma` as pb.
- `err(kind, msg)` is the constructor now; a new error site must pick a kind.

## Found

- **The TUI shows a decay's running width as picobarns.**
  `vibegraph-cli/src/tui/state.rs:129-131` multiplies the VEGAS progress
  `sigma`/`err` by `GEV2_TO_PB` unconditionally, but a decay's integrand is in
  GeV, so the pane reads Γ × 3.894e8 labelled pb (code reading; not run on a
  terminal). Close: carry the observable into `UiState` (as
  `describe_process` carries the process) and convert only a cross section,
  with a state test on a width.
- **The survey's database has no column for the error kind.** `parse_vg_ldjson`
  returns `error_kind`, but `schema.sql`'s `vg_run` and `vgrun.py`'s insert do
  not store it, so reasons are still grouped by message template only. Close: a
  `vg_run.error_kind` column with the schema-version bump the database's own
  test requires.
- **The ldjson stream is dominated by per-block progress.** A 4000-point,
  four-iteration fixed-energy run emits 450 `progress` objects (440 of them
  `vegas`) beside 239 `log` objects; a large proton run will emit many more.
  Nothing rate-limits them. Close, if it matters to a consumer: a minimum
  interval per stage in the ldjson progress layer, documented in the schema.
- **Re-parsing a recorded proc card logs at INFO.** `generate` and `inspect`
  parse the artifact's proc card and so print `generate … @1` (the parser's
  own INFO line) as if a card had been read from disk. Close: parse the recorded
  card at a quieter level, or say in the line that it is the recorded card.
