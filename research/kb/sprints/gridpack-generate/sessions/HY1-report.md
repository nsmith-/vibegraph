---
type: Session Report
title: "HY1 report: hygiene review of the gridpack-style generate"
description: "Twenty findings on F1 and F2's diff: one parton-density predicate and one card-value formatter instead of two, two new rustdoc warnings and four doc/comment slips fixed, the argument-error test tightened to name the refused argument, the --seed help pinned to DEFAULT_INTEGRATION_SEED; the warn-once guard is unreachable by any mutation; six cross-module or design findings go to Found."
status: draft
tags: [gridpack-generate, hygiene, review, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
---
Worktree `/home/user/wt/H1`, branch `sprintd-gridpack-generate`, from
`74f8805`. The diff reviewed is `git diff origin/main...HEAD` (F1 `723f6d1`,
F2 `39a1a25`), against the [hygiene review](../../../workflow/hygiene-review.md)
checklist.

## Findings

The table was written before the first edit. Confidence: every row is
*checked*; the two *suspected* rows (HY1.6, HY1.7) were settled by the
mutations below.

| id | point | site | evidence | disposition |
|---|---|---|---|---|
| HY1.1 | 1 | `vibegraph-lib/src/generate.rs:1148` | `has_decay_chains`'s doc ("Whether any process line of the card carries decays…") sat on `forbidden_onshell`, stacked over that function's own doc; `has_decay_chains` had none. On main it was on `has_decay_chains` (CLI `generate.rs:739`), so the move displaced it | fixed |
| HY1.2 | 1, 4 | `vibegraph-lib/src/runcard.rs:244` `card_text` | the same five-arm `ParamValue` match as `lhef/write.rs:290-296` (`grep -rn '"True".to_string()'` returns both) | fixed: `mg_run_card` calls `card_text` |
| HY1.3 | 1, 4 | CLI `generate.rs:371`, lib `generate.rs:250` | the identical predicate `!card.is_decay() && artifact.run_card.beam_mode() == BeamMode::Proton` in both | fixed: `vibegraph::generate::reads_parton_densities`, used by both |
| HY1.4 | 1 | lib `generate.rs:1452` ("the design's message"), `runcard/classes.rs:631` ("written from the design's classification") | references to a planning document in comments (AGENTS.md comment rule) | fixed |
| HY1.5 | 1 | CLI `integrate.rs:435` | `--seed`'s help spells `20260719`, a second copy of `runcard::DEFAULT_INTEGRATION_SEED` that nothing ties to it; the constant had no user outside `runcard.rs` | fixed: a unit test pins the help to the constant |
| HY1.6 | 2 | `vibegraph-cli/tests/cli_generate.rs:625` | `expect_refusal(out, what, "unexpected argument")` matches clap's word for any unknown token, not the argument under test | fixed: matches `unexpected argument '<arg>'` |
| HY1.7 | 2 | CLI `assets.rs:148` (`Once` in `warn_on_retired_pdf_dir`) | each command resolves one PDF set (`integrate.rs:807`, `generate.rs:373` are the only callers of `load_pdf_set`), so `vibegraph_pdf_dir_resolves_nothing_and_draws_a_warning`'s one-warning assert cannot see the `Once` | leave: the guard is cheap and a per-beam set would reach it; the blind spot is named below |
| HY1.8 | 1 | CLI `assets.rs:5`, `:7` | `cargo doc -p vibegraph --no-deps --document-private-items`: "unresolved link to `cache::resolve::locate`" (:5, already a site of `rustdoc-warnings-ungated`) and "unresolved link to `cache::resolve::locate_pdf_set`" (:7, new) | fixed, both |
| HY1.9 | 1 | lib `generate.rs:118`, `:142-144`, `:194` | `GenerateOptions::strategy`, `GenerateAssets::{card, model, evaluated}` and `GenerateError::Other` were undocumented `pub` items of the new module | fixed |
| HY1.10 | 1 | CLI `generate.rs:415` | `use vibegraph::runcard::EventNorm;` in the test module repeated what `use super::*` already brings in | fixed |
| HY1.11 | 1 | lib `generate.rs:184-187` | `ModelChanged` says "its files changed after vibegraph integrate ran; re-integrate, or restore them" also for location `(built in)`, where there are no files and the build changed. The text is the design's verbatim | Found |
| HY1.12 | 1 | lib `generate.rs:186`, `:1096` | library messages name `vibegraph integrate`; `artifact.rs:50` (`UnsupportedVersion`) already does. No flag name is left in the library | leave |
| HY1.13 | 3 | CLI `generate.rs:358` | `nevents == 0` is refused by the CLI only; `vibegraph::generate::generate` scans and emits on it | Found |
| HY1.14 | 4 | `config.rs:101`, CLI `assets.rs:210`, CLI `generate.rs:281` | three `import.name == "sm"` predicates for "the interned model"; `model_location` also rebuilds `config.rs:120`'s path | Found |
| HY1.15 | 4 | banked test files | the "AWAITING BUNDLE" escape for an absent run; the PR adds none (`cli_generate_proton` uses `banked_present` and `validation::require`) | Found |
| HY1.16 | 4 | eight CLI test files | the same `fn pdf_dir()` (`../validation/pdf`) | Found |
| HY1.17 | 1 | lib `generate_sample` (~220 lines), `generate_proton_sample` (~190) | moved from the CLI as they were, not grown | leave |
| HY1.18 | 2 | the deleted `card_mismatches` tests | covered `ebeam1`, `ptl`, an omitted card (`lpp1`), `event_norm`, the process and the restrict variant. The first four can no longer arise: `generate` accepts no card (`a_card_or_a_pdf_choice_passed_to_generate_is_an_argument_error`). The process is `a_recorded_proc_card_that_parses_to_another_process_is_refused` (CLI and lib); the model is the digest tests (lib, CLI interned SM, CLI UFO restrict card); the PDF set is `generate_locates_the_recorded_pdf_set_by_name`. The two deleted `cli_hard_errors` tests check a generate-time run card, which no longer exists. No gap | leave |
| HY1.19 | 1 | backlog sites | four items name sites the move made stale (below) | Found |
| HY1.20 | 1 | lib `multiplicity.rs:38` | `cargo doc -p vibegraph-lib`: "redundant explicit link target", new: the change imported `derive_flavor_groups` into the module | fixed |

Points checked with nothing found:
- **Visibility.** Every new `pub` has a user outside its module: `check_process`,
  `check_model`, `GenerateAssets`, `GenerateOptions`, `GenerateError` (CLI
  `generate.rs`); `multiplicity_groups`, `multiplicity_maps`,
  `refuse_mixed_multiplicity`, `MultiplicityGroups` and `integration_seed`
  (CLI `integrate.rs`); `InitialState::from_run_card` (CLI and ten lib test
  targets); `SupportedCard::is_decay` (three CLI modules); `pdf_data_path`,
  `locate_pdf_set`, `LHAPATH_VAR`, `RETIRED_PDF_DIR_VAR` (CLI `assets.rs`);
  `mg_run_card` (two test targets); `GenerateSettings` and `Systematics` are the
  type of the `pub` field `IntegrateArtifact::generate`. `DEFAULT_INTEGRATION_SEED`
  had none and now has HY1.5's test. `source.rs` and `audit.rs` are `pub(super)`
  throughout.
- **One place per rule.** The stage table is `FIELD_CLASSES` alone, read through
  `generate_fields` and `systematics_request_fields`; `split` and
  `integration_seed` have one definition and one production caller each
  (`integrate.rs:638-639`); the `LHAPDF_DATA_PATH`/`LHAPATH` parse is
  `pdf_data_path` alone (every other mention is a test setting the variable or a
  validation script exporting it). The design is cited by no code comment.
- **Comments.** `grep -nE '\bT[0-9]\b|\bC[0-9]\b|Phase [0-9]|note [0-9]+|used to|no longer'`
  over the added lines finds only runtime text (the retired-variable warning)
  and conditions (`check_process`'s "no longer parses"), besides HY1.4.
- **`common::PdfDataPath`.** Every test that needs a PDF set sets it through
  `pdf_data_path`, except `cli_pdf_cache_fetched::an_explicit_pdf_dir_still_takes_precedence`
  (it tests the flag) and `cli_onshell_veto.rs:324` (F2's Found). The tests that
  build on `vibegraph_ambient()` still read the developer's `VIBEGRAPH_HOME` and
  `VIBEGRAPH_UFO_DIR`, but each puts the set on the data path, which is searched
  before the cache, and uses the interned SM, so neither variable can change
  what they read.

## Fixed

| finding | commit | change |
|---|---|---|
| HY1.1-HY1.4, HY1.8-HY1.10, HY1.20 | `4199955` | `reads_parton_densities`; `card_text` shared; docs and links |
| HY1.5, HY1.6 | `699b392` | the `--seed` help test; the argument-error assert |

Refactor oracles (HY1.2, HY1.3), chosen before the edit: the existing unit
tests of `mg_run_card` (`mg_run_card_carries_what_set_mad_reads`,
`mg_run_card_records_the_generation_as_drawn`, which compare the block line by
line) and the CLI decay and fixed-energy targets, which reach the predicate's
two branches. All pass after the change, and the two mutations below show each
oracle reaches the shared code.

## Mutations

Driver `scratchpad/gpH1-HY1-mutate.py` (logs `gpH1-HY1-mut-pre.log`, `gpH1-HY1-mut-post.log`): apply, run the named test, restore,
log. `git diff --stat` was empty after each batch.

| id | mutation | before the fix | after |
|---|---|---|---|
| HY1.6 | `--pdf-set` added to `GenerateArgs` as a `bool` flag, so the set name is the unexpected token | `a_card_or_a_pdf_choice_passed_to_generate_is_an_argument_error` **passed** (1 passed) | **FAILED** `cli_generate.rs:627`: "error: unexpected argument 'NNPDF23_lo_as_0130_qed' found" |
| HY1.7 | the `Once` removed from `warn_on_retired_pdf_dir` | `vibegraph_pdf_dir_resolves_nothing_and_draws_a_warning` passed | passed (no test changed): unreachable |
| HY1.5 | the help's `20260719` changed to `20260718` | no test | **FAILED** `integrate.rs:1068` |
| HY1.3 | `reads_parton_densities` drops `!card.is_decay()` | – | **FAILED** `cli_decay::a_top_decay_sample_follows_madevents_convention_and_shape` (`cli_decay.rs:450`) |
| HY1.2 | `card_text` writes `True` as `T` | – | **FAILED** `lhef::write::tests::mg_run_card_carries_what_set_mad_reads` (`write.rs:830`) |

Unreachable site: HY1.7's `Once`. No command resolves two PDF sets, so no
test can tell one warning per process from one per resolution.

## Gate

```
$ cargo fmt --all --check
exit 0

$ cargo clippy --workspace --all-targets -- -D warnings
    Checking vibegraph-lib v0.1.0 (/home/user/wt/H1/vibegraph-lib)
   Compiling vibegraph v0.1.0 (/home/user/wt/H1/vibegraph-cli)
    Finished `dev` profile [optimized] target(s) in 26.55s
exit 0

$ cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
    Checking vibegraph-lib v0.1.0 (/home/user/wt/H1/vibegraph-lib)
   Compiling vibegraph v0.1.0 (/home/user/wt/H1/vibegraph-cli)
    Finished `dev` profile [optimized] target(s) in 24.98s
exit 0

$ cargo clippy --workspace --all-targets --all-features -- -D warnings
    Checking vibegraph-lib v0.1.0 (/home/user/wt/H1/vibegraph-lib)
    Checking vibegraph v0.1.0 (/home/user/wt/H1/vibegraph-cli)
    Finished `dev` profile [optimized] target(s) in 32.76s
exit 0

$ cargo test --workspace
exit 0
42 result lines: 1607 passed, 0 failed, 18 ignored

$ cargo doc -p vibegraph --no-deps --document-private-items
    Finished `dev` profile [optimized] target(s) in 34.79s   (0 warnings; 2 before)

$ cargo doc -p vibegraph-lib --no-deps --document-private-items
warning: `vibegraph-lib` (lib doc) generated 8 warnings   (9 before; the 8 are rustdoc-warnings-ungated's sites)

$ python3 scripts/kb.py lint
886 concepts, 292 backlog items; 0 errors, 4 warnings
```

The suite count is F2's 1606 plus `integrate::tests::the_seed_help_names_the_default_integration_seed`;
no test was deleted. No banked target was run: the changes touch the
`<MGRunCard>` writer (`validate_mlm_dumps` reads it), but this container has no
MadGraph output and no PDF set, so the target would stop in
`validation::require`. Both clippy runs under the extended features compiled
it. The `<MGRunCard>` refactor is covered by the two unit tests above.

## Method

- Reading the diff of the moved and new code with the old CLI beside it
  (`git show origin/main:vibegraph-cli/src/generate.rs`): about 35%. Found
  HY1.1, HY1.3, HY1.9, HY1.17.
- Workspace greps for a distinctive line or name (`"True".to_string()`,
  `BeamMode::Proton`, `== "sm"`, `LHAPATH`, `integration_seed`, `fn pdf_dir`,
  `AWAITING BUNDLE`) and for each new `pub` item's readers: about 25%. Found
  HY1.2, HY1.3, HY1.5, HY1.14-HY1.16 and settled the visibility and one-place
  questions.
- `cargo doc` per package: 5%, two new warnings (HY1.8, HY1.20), which no other
  technique would have seen.
- Asking what each changed test's assert is blind to, then running it: about
  20%. HY1.6 held; HY1.7 held as a blind spot rather than a fix.
- Backlog path-tail greps: 10%, HY1.19.
- Leads that did not hold: the CLI `Strategy`/`EventNormArg` enums copying the
  library's (they carry clap's `ValueEnum` and help text, which the library does
  not depend on); the CLI's `reweight_launches` forbidden-s-channel refusal
  copying `ReweightPlan::new`'s (it predates the PR and refuses before
  compilation); `lhapdf_data_path_serves_a_run_without_touching_the_cache`'s
  "cache untouched" assert (vacuous under `VIBEGRAPH_NO_NETWORK`, but the run's
  success is the real check).
- A wait loop of mine that used `pgrep -f gpH1-HY1-mutate.py` matched its own
  command line and never exited; it was stopped by id.

## Found

1. **The model-digest refusal is wrong for the interned SM.** `GenerateError::ModelChanged`
   (lib `generate.rs:184-187`) tells a user whose build compiled in a
   different SM that "its files changed after vibegraph integrate ran; …
   restore them", with location `(built in)`. There are no files. The text is
   [integrate and generate split](../../../pipeline/integrate-generate-split.md)
   §3's verbatim. To close: a built-in variant of the message ("this build's
   built-in model differs from the integrating build's; re-integrate"), and a
   correction to §3.
2. **`vibegraph::generate::generate` accepts `nevents = 0`.** Only the CLI
   refuses it (CLI `generate.rs:358`); a library caller gets a full scan and an
   empty file. To close: refuse it at the library entry point and let the CLI
   keep its own wording, as `cli_error` does for the rounding refusals.
3. **The "interned model" predicate is written three times**:
   `vibegraph-lib/src/config.rs:101` (`load_ufo_with_identity`),
   `vibegraph-cli/src/assets.rs:210` (`resolve_ufo_search_path`) and
   `vibegraph-cli/src/generate.rs:281` (`model_location`, which also rebuilds
   `config.rs:120`'s `ufo_search_path.join(name)`). To close: one predicate on
   `ModelImport` (or `GlobalConfig` returning where it read the model), used by
   all three.
4. **The "AWAITING BUNDLE" absent-run escape is repeated across banked targets**:
   `vibegraph-cli/tests/validate_samples_proton.rs:1260`,
   `vibegraph-lib/tests/validate_hadronic.rs:1359`,
   `vibegraph-lib/tests/validate_sigma.rs:4850` and `:4946`, and the
   `unbundled_rows()` reads at `validate_sigma.rs:1319`, `:4490`, `:4531`,
   `:4668`, `:5493`, `validate_alphas.rs:126`, `validate_scales.rs:358`. This
   PR adds none. To close: one `common::manifest` helper that returns "run
   present", "awaiting bundle" or calls `require`.
5. **`fn pdf_dir()` is defined in eight CLI test files**:
   `cli_generate_proton.rs:127`, `cli_integrate.rs:36`, `cli_decay_chain.rs:83`,
   `cli_decay_chain_events.rs:132`, `cli_reweight_proton.rs:61`,
   `validate_samples_proton.rs:98`, `cli_onshell_veto.rs:281` and
   `cli_pdf_cache_fetched.rs:32` (`dev_pdf_dir`). To close: one helper in
   `tests/common/mod.rs` beside `PdfDataPath`.
6. **Backlog sites made stale by the move** (one entry each):
   - `lhef-idwtup-minus4-buffers-whole-sample` names `vibegraph-cli/src/generate.rs:97`,
     `:601`, `:1670`; `Strategy::Buffer` is now `vibegraph-lib/src/generate.rs:105`
     and the emit path is `emit_to` in the same file.
   - `stochastic-rounding-refuses-mixed-multiplicity` names `vibegraph-cli/src/generate.rs:436`;
     the refusal is `refuse_rounding_on_mixed_multiplicity`, `vibegraph-lib/src/generate.rs:280`.
   - `no-lazy-event-iterator` names `generate_sample` in `vibegraph-cli/src/generate.rs`;
     it is in `vibegraph-lib/src/generate.rs`.
   - `lib-tests-hand-roll-temp-dirs` names `cache/resolve.rs:134` (now `:169`)
     and `artifact.rs` lines the PR deleted; the PR adds five new uses of the
     hand-rolled helpers (four `cache::resolve` tests, `artifact::a_version_12_artifact_is_refused`).
   - `rustdoc-warnings-ungated`: its `vibegraph-cli/src/assets.rs:5` site is
     fixed by HY1.8.
7. **The warn-once guard is untested** (HY1.7). It becomes observable when a
   command resolves two PDF sets; that change should test it.

## Brief corrections

- The brief's gate names `--features vibegraph/extended-validation`; the
  hygiene definition names `--all-features`. Both were run.
- `cargo doc -p vibegraph-cli` is not a package: the CLI package is `vibegraph`
  and the library's is `vibegraph-lib`.
- "F2 found the 'absent run' escape duplicated elsewhere": F2's report does not
  record it, and no backlog item names it. I read it as the AWAITING BUNDLE
  escape (Found 4).
- The hygiene definition says not to edit `research/kb/`; the brief asks for
  this report there. The brief was followed, and the sessions index was
  regenerated with `python3 scripts/kb.py index`.
