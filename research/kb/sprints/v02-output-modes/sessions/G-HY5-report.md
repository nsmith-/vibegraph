---
type: Session Report
title: "G-HY5 report: hygiene review of the output modes; a decay's TUI width, the survey's error kind, the recorded-card announcement"
description: "Review of G-F6 (a0be63f..c76c3de): the TUI shows a width in eV, vg_run stores the refusal's error kind (database schema 3), only a card read from disk is announced at INFO, a declined systematics download is a network refusal, the version rule sits on SCHEMA_VERSION and in the user docs, and a decay's ldjson result is tested. Fifteen mutations, all as expected; every gate green."
status: draft
tags: [hygiene, cli, output, ldjson, tui, cms-survey, sprint-g, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
measured:
  host: "cloud container, linux-64, 4 cores, 2026-11-05"
  commit: "5575ff1 on 5602ce9"
  command: "G-HY5-gates.sh and G-HY5-mutate.py, logs G-HY5-* in the session scratchpad"
sources:
  - {id: brief, resource: "G-HY5.md", title: "G-HY5 brief"}
  - {id: f6, resource: "G-F6-report.md", title: "G-F6 report"}
---
Scope: G-F6's change, `a0be63f..c76c3de` (feature `75e6854`, test fix
`e06b541`): `output.rs`, `error.rs`, `logging.rs`, `inspect.rs`, the
`integrate`/`generate`/`check`/`main` edits, `cli_output_ldjson.rs`, the
banked-test additions, the survey's parser and `vgrun.py`, and the CLI
overview. Worktree `/home/user/wt/G4-HY5`, branch `sprintg-output-modes-hy5`.

## Findings (written before any edit)

| id | pt | site | evidence | confidence | disposition |
|---|---|---|---|---|---|
| F1 | 1 | `vibegraph-cli/src/tui/state.rs:129-131` | `state.sigma_pb = Some(sigma * GEV2_TO_PB)` for every progress event; a decay's integrand is in GeV (`Observable::per_natural_unit` is 1 for a width) | checked | **fixed** (brief 1) |
| F2 | 1 | `validation/cms_gridpacks/schema.sql:293-333`, `vgrun.py:276-286` | `parse_vg_ldjson` sets `res["error_kind"]`; `vg_run` has no column and the insert names none | checked | **fixed** (brief 2) |
| F3 | 1 | `vibegraph-lib/src/diagrams/mod.rs:229`, reached from `integrate.rs:1060` (`recorded_observable`) and `generate.rs:382` | `info!("generate {process} @{}")` inside `parse_proc_card`; settled by M4 below: with that line at INFO, `inspect --output-ldjson` streams one `generate e+ e- > mu+ mu- @1` log object and `generate` another | checked (by M4) | **fixed** (brief 3) |
| F4 | 4 | `generate.rs:539-543`, `integrate.rs:1066-1070` | the same `if parsed.is_decay() { PartialWidth } else { CrossSection }` twice in the change; F1 needs a third | checked | **fixed** (`integrate::observable_of`) |
| F5 | 1 | `output.rs:232-235` | `IntegrateResult`: "Every field is read from the artifact", but `integrate.rs:1001` passes `output.observable`, the integrand's | checked | **fixed** (doc) |
| F6 | 2 / kinds | `generate.rs:497-503` | every `systematics::plan::ResolveError` is classed `pdf`; `ResolveError::Declined` (`plan.rs:52-58`, built at `plan.rs:299` when the fetch callback refuses) is a download the policy did not allow, which `ErrorKind::Network` names | checked | **fixed**, unit test |
| F7 | kinds | `vibegraph-lib/src/generate.rs:233`, CLI `generate.rs:345` | `GenerateError::Other` → `cli_error`'s `other => ErrorKind::Run` carries `cannot create {out}` (761), `the artifact banks no channel grids` (1405), the channel count/key/dimension mismatches (820-845) and the channel-map digest mismatch (856): `io` and `artifact` failures streamed as `run` | checked | **Found** (library API) |
| F8 | 3 | `output.rs:119-123` | `grep -rn 'Refusal\b' vibegraph-cli/src` outside `output.rs` finds only a comment in `network.rs` | checked | **fixed** (private) |
| F9 | 1 | `output.rs:50`, `docs/src/cli/overview.md:225,251` | the field rule (adding a field keeps the version) is in `output.rs`'s module docs and the kb concept only; the user page states only the kind rule, and gives no width unit for progress | checked | **fixed** (doc) |
| F10 | 2 | `tests/cli_output_ldjson.rs:284` | `inspect`'s printed σ held to `<= a.sigma_err_pb`; it prints to the error's second figure, so its rounding is under 0.05 σ. Mutation: print σ + 0.5·err | suspected → checked (M7, M7old) | **fixed** |
| F11 | 2 | `tests/` | `grep -rn 'partial_width\|"GeV"' vibegraph-cli/tests` finds nothing: no test reads a decay's result object. Mutation: `unit: "pb"` in `result_object` | checked (M8old) | **fixed** (new test) |
| F12 | — | `output.rs:18-19` | a panic leaves the stream without a final object | checked | **leave**: documented; the survey reads the panic off stderr |
| F13 | — | ldjson stdout writers | `grep -rn 'println!\|print!(\|stdout()' vibegraph-cli/src`: `tui/mod.rs:273,282` behind `output::text_line` / the held-lines path, `check.rs:301-319` (`check-events` refuses the flag in `main`), `output.rs:87` the writer itself | checked | no finding |
| F14 | — | `inspect.rs:35` | `mode() == Ldjson` duplicates the gating in `output::result`/`text_line` | checked | **leave**: it skips formatting the text lines |

Brief points of attention, as found: the ldjson writer holds the stdout lock
per object, and every stdout writer is gated (F13). Kinds are set at each
`err(kind, …)` site except the two library catch-alls (F6 fixed, F7 Found).
`inspect_emits_the_object_integrate_emitted` already existed; F11 adds the
decay case. The version rule existed in the module docs; F9 puts it on the
constant and in the user docs.

Backlog check: `grep -rlE` over `research/kb/backlog` for the reviewed paths
(`cli/src/(output|inspect|error|logging|integrate|generate|main).rs`,
`tui/state.rs`, `cms_gridpacks/…`) names
`multiplicity-sum-construction-copied`, `library-generate-accepts-zero-events`,
`interned-model-predicate-written-three-times`, `diagram-inspection-command-missing`
and three performance items; none covers a finding here, and none is made
stale by these commits.

## Fixed

| finding | commit | change |
|---|---|---|
| F2 | `83488d4` | `vg_run.error_kind TEXT CHECK (… ten kinds …)`, `CHECK (error_kind IS NULL OR outcome = 'refused')`, the `comparison` view carries it, `vgrun.py` inserts `r["error_kind"]`; `SCHEMA_VERSION`/`user_version` 2 → 3. Test `Schema.test_a_refusal_keeps_its_error_kind` (the parser's kind round-trips; an unknown kind and a kind on an `ok` run are refused) |
| F3 | `95b6504` | `parse_proc_card` logs each process at DEBUG; `read_proc_card_text` (the card `integrate` reads) logs it at INFO under the `proc_card` span. `cli_output_ldjson`: one announcement in `integrate`'s stream, none in `inspect`'s or `generate`'s |
| F1, F4 | `5f3c4f0` | `UiState::{estimate, estimate_err}` in the reported unit plus `observable`; `describe_process(process, observable)`; the fold multiplies by `observable().per_natural_unit()`; `footer::measured` prints `σ … b` or `Γ … eV`, used by the pane and the closing summary. `integrate::observable_of` replaces the copy in `generate`. Tests `a_width_is_taken_in_gev`, `a_width_renders_in_electronvolts` |
| F6 | `0b457c8` | `error::systematics_error_kind`: `Declined` → `network`, the rest `pdf`. Test `a_declined_systematics_download_is_a_network_refusal` |
| F5, F8, F9 | `0d55e89` | docs on `SCHEMA_VERSION`, `IntegrateResult` and the CLI overview; `Refusal` private |
| F10, F11 | `5575ff1` | `inspect_prints_the_recorded_results_as_text` holds value and error to 0.06 σ (two-figure rounding is < 0.05 σ; the rest is headroom for the pb → b conversion); `a_decay_result_is_a_width_in_gev` (`t > b e+ ve`, `--neval 2000 --niter 3`) pins `observable`, `unit`, `sigma`, `mu_f` and `inspect`'s object equal to `integrate`'s |

The commits split one gated tree; only the final tree (`5575ff1`) ran the
gate.

## Mutations

`G-HY5-mutate.py`, log `G-HY5-mutations.log`: apply, run, revert; all
fifteen gave the expected result, and the tree was checked clean afterwards.

| id | mutation | command | result |
|---|---|---|---|
| M1 | fold converts by `GEV2_TO_PB` always | `cargo test -p vibegraph --bin vibegraph -- tui::` | FAILED `a_width_is_taken_in_gev` |
| M2 | footer: width under `(PICOBARN, "b")` | same | FAILED `a_width_renders_in_electronvolts` |
| M3 | `describe_process` drops the observable | same | passes: **unreachable** (below) |
| M4 | library parser back at INFO | `cargo test -p vibegraph --test cli_output_ldjson` | FAILED `inspect_emits_…` (line 274), `the_generate_result_…` (line 344) |
| M5 | CLI reader announces at DEBUG | `… -- inspect_emits` | FAILED (line 274) |
| M6 | `Declined` → `Pdf` | `cargo test -p vibegraph --bin vibegraph -- error::` | FAILED `a_declined_systematics_download_is_a_network_refusal` |
| M7 | `inspect` prints σ + 0.5·err | `… --test cli_output_ldjson -- inspect_prints` | FAILED (line 300) |
| M7old | M7 against the old 1σ bound | same | passes: the old test was blind to it |
| M8 | `result_object` `unit: "pb"` | `… -- a_decay` | FAILED (line 408) |
| M8old | M8 against every pre-existing ldjson test | `… -- --skip a_decay` | passes 9/9: the old suite was blind to it |
| M9 | `observable_of` always a cross section | `… -- a_decay` | FAILED (line 423, inspect's object ≠ integrate's) |
| P1 | `error_kind TEXT` without the kind list | `python3 -m unittest discover -s validation/cms_gridpacks -p 'test_*.py'` | FAIL `test_a_refusal_keeps_its_error_kind` |
| P2 | drop `CHECK (error_kind IS NULL OR outcome = 'refused')` | same | FAIL `test_a_refusal_keeps_its_error_kind` |
| P3 | `schema.sql` left at `user_version = 2` | same | FAIL `test_fresh_database_matches_schema_version` |
| P4 | parser drops `error_kind` | same | FAIL `test_a_refusal_keeps_its_error_kind`, `test_refused` |

Sites no mutation reaches:

- `describe_process`'s write of the observable (M3): it goes through the
  `DISPLAY` `OnceLock`, which no test installs (installing it would put every
  later test in the binary under a live display). The fold and the footer are
  tested from a state that holds the observable; the call sites pass
  `observable_of(&parsed)`.
- `vgrun.py`'s insert of `r["error_kind"]`: nothing runs `run_gridpack`
  without a binary. The column, its checks, the view and the parser are
  tested; the binding was checked by counting 33 columns, 33 placeholders and
  33 values.
- `systematics_error_kind` is tested on the variants directly, not through a
  `generate --systematics` run that declines a download.

## Routed

None: no finding moves a number, a convention or a hot path.

## Gate

`G-HY5-gates.sh` on the final tree, with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

| command | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets --jobs 3 -- -D warnings` | exit 0, `Finished` |
| `cargo clippy --workspace --all-targets --features vibegraph/extended-validation --jobs 3 -- -D warnings` | exit 0, `Finished` |
| `cargo clippy --workspace --all-targets --all-features --jobs 3 -- -D warnings` | exit 0, `Finished` |
| `cargo test --workspace --jobs 3` | exit 0; 55 result lines, 1935 passed, 0 failed, 25 ignored (G-F6: 1931; +4: `a_width_is_taken_in_gev`, `a_width_renders_in_electronvolts`, `a_declined_systematics_download_is_a_network_refusal`, `a_decay_result_is_a_width_in_gev`; `cli_output_ldjson` 10/10) |
| `cargo doc -p vibegraph --no-deps --document-private-items` | exit 0, no warnings |
| `cargo doc -p vibegraph-lib --no-deps --document-private-items` | exit 0, 7 warnings, all in files this branch does not touch (`coupling/cluster/configs.rs`, `phasespace/channel.rs`, `coupling/alphas.rs`, `coupling/scales.rs` ×2, `lhef/mod.rs` ×2) |
| `cargo test -p vibegraph --profile release-debug --features extended-validation --test cli_integrate` (reads `inspect`'s object; F3 changes its log stream) | `ok. 6 passed; 0 failed` |
| `… --test cli_generate_proton` (the same, on the mixed-multiplicity run) | `ok. 11 passed; 0 failed; 2 ignored` |
| `python3 -m unittest discover -s validation/cms_gridpacks -p 'test_*.py'` | `Ran 31 tests … OK` (+1: `test_a_refusal_keeps_its_error_kind`) |
| `python3 scripts/kb.py lint` | `0 errors, 5 warnings` (the five pre-existing backlog `blocked_by` warnings) |

## Method

Reading the G-F6 diff against its report (about 45%); workspace greps for
stdout writers, `err(` kinds, catch-all mappings, visibility and duplicated
mappings (about 25%); the mutation batch (about 20%); test runs (10%). The
error-kind catch-alls were found by following each library error type into
its `map_err` and listing its variants, not by reading the CLI sites, which
all name a kind. Leads that did not hold: a non-JSON stdout line in ldjson
mode (every writer is gated or refused); a missing `inspect` = `integrate`
test and a missing version rule (both existed; extended rather than added).

## Found

- **`generate`'s library refusals other than the four named variants stream
  as `run`.** `vibegraph::generate::GenerateError::Other` carries an IO failure
  (`cannot create {out}`, `generate.rs:761`) and the artifact disagreements
  (no channel grids, `:1405`; channel count, key and grid dimension, `:820-845`;
  channel-map digest, `:856`), and `cli_error` maps `Other` to `run`, so a
  consumer branching on `artifact` misses them. Close: library variants (or a
  kind on `Other`) for IO and artifact disagreement, mapped in `cli_error`,
  with a refusal test on a stale channel-map digest.
- **Two systematics refusals are classed `pdf` though they are about the
  request.** `ResolveError::NominalNotFirst` and the request-content cases of
  `ResolveError::Entry` (a member id outside the set, `plan.rs:370-400`) come
  from `--systematics` or the recorded card's `systematics_arguments`, and
  `Entry` also wraps a set that failed to open, as a string. The map site does
  not know which. Close: split `Entry` by cause and carry the request's origin
  so the CLI can say `usage` or `card`.
- **`research/kb/tooling/cli-output-modes.md` predates these fixes**: it
  names `error::pdf_error_kind` as the only classifier (now also
  `error::systematics_error_kind`) and does not say that only the card
  `integrate` reads is announced at INFO, or that `vg_run` stores the kind.
- **A panic ends an ldjson stream without a final object.** Documented, and
  the survey reads it from stderr, but a consumer reading stdout alone sees a
  truncated stream. Close, if a consumer needs it: a panic hook that emits a
  `refusal` of kind `run` in ldjson mode.

## Brief corrections

- The scope `sprintg-cli-output..sprintg-output-modes` includes the merge of
  `main` (cuts, proton, diagrams, the H-sprint reports); G-F6's own change is
  `a0be63f..c76c3de`, which is what was reviewed.
- The hygiene agent definition forbids edits under `research/kb/`; the brief
  asks for this report there, and the brief was followed. The sessions index
  was regenerated with `scripts/kb.py index`.
- The brief's clippy feature run is `--features vibegraph/extended-validation`;
  the definition's is `--all-features`. Both were run.
