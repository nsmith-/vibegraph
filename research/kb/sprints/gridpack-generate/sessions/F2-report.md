---
type: Session Report
title: "F2 report: generate takes the artifact and no card, checks the recorded proc card and the model digest, and finds PDF sets through LHAPDF_DATA_PATH"
description: "generate <ARTIFACT> --seed S with no card or PDF argument; the recorded proc card re-parsed and checked against process; the model checked by digest, the interned SM included; LHAPDF_DATA_PATH (else LHAPATH) replaces VIBEGRAPH_PDF_DIR and the validation/pdf fallback; --event-norm; negative nevents refused; the rounding refusals phrased in library terms. All 21 mutations killed."
status: draft
tags: [gridpack-generate, generate, cli, pdf, model-identity, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
---
Worktree `/home/user/wt/H1`, branch `sprintd-gridpack-generate`, from
`96175fb`. The design is
[integrate and generate split by data dependency](../../../pipeline/integrate-generate-split.md)
§3–§5, with
[run inputs](../../../pipeline/run-inputs-in-the-artifact.md) §5 and §7.

## The command line

Before:

```
vibegraph generate [OPTIONS] <ARTIFACT> <PROC_CARD>
  --run-card F  --ufo-dir D  --nevents N  -o FILE  --force  --strategy …
  --seed S (default 20260728)  --pdf-set NAME (default NNPDF23_lo_as_0130_qed)
  --pdf-dir D  --scan-points …  --max-truncation …  --reweight-card F
  --reweight-exact  --reweight-couplings …  [parallel, log, network options]
```

After:

```
vibegraph generate [OPTIONS] <ARTIFACT>
  --seed S (required)  --nevents N  -o FILE  --force  --strategy …
  --event-norm average|sum|unity (default: the recorded value)  --ufo-dir D
  --scan-points …  --max-truncation …  --reweight-card F  --reweight-exact
  --reweight-couplings …  [parallel, log, network options]
```

`integrate` keeps `--pdf-set` and `--pdf-dir` (H2 removes them); only the
`--pdf-dir` help changed.

## What `generate` does, in order (`vibegraph-cli/src/generate.rs`, `run`)

1. A missing `--seed` is refused first: "generate needs --seed: each
   generation job needs its own seed, since two jobs drawing from one grid
   with the same seed write the same events".
2. The artifact is read (the floor stays 13).
3. `artifact.proc_card` is parsed, and `vibegraph::generate::check_process`
   compares its process string with `artifact.process`
   (`GenerateError::ProcessReparsed`, ending "re-integrate").
4. The model is resolved through the UFO search path, loaded, and
   `vibegraph::generate::check_model` compares its digest with
   `model.digest` (`GenerateError::ModelChanged`). The text is §3's verbatim;
   `{dir}` is `<search path>/<name>`, or `(built in)` for the interned SM,
   which goes through the same check.
5. The settings are `artifact.generate`, with `--nevents` and `--event-norm`
   applied. The zero-events message now reads "--nevents is absent and
   {artifact} recorded nevents = 0".
6. The reweight card is resolved against the model. The forbidden s-channels
   come from the recorded proc card, because that is the card `parsed` holds.
7. On proton beams, `artifact.pdf_set` is located by name with no flag, and
   `artifact.pdf_member` is loaded. Before this change the CLI used its
   `PDF_MEMBER` constant.
8. `vibegraph::generate::generate` runs, and its error goes through
   `cli_error`, which phrases the two rounding refusals with the CLI's flags.

Deleted: `CardMismatch`, `card_mismatches`, `pdf_mismatches`,
`refuse_on_mismatch`, `settings_mismatches`, their seven unit tests, and the
`read_proc_card` wrapper in `main.rs`, whose only caller was `generate`.

## PDF location (C7, C8; H1's part)

- `vibegraph-lib/src/cache/resolve.rs`:
  - `pdf_data_path(lhapdf_data_path, lhapath)` uses `LHAPDF_DATA_PATH` when it
    is set, even to the empty string, and `LHAPATH` otherwise. It splits the
    value as a path list and skips empty entries.
  - `locate_pdf_set(name, flag, data_path, cache_root)` tries the flag
    (trusted, as before), then the first data-path entry that holds the set,
    then the cache. A data-path entry that lacks the set is passed over.
  - `Source::DataPath` is new.
  - `locate_from_env` is deleted. It existed only for the PDF path.
  - `locate` (UFO) keeps its dev fallback (`.`).
- `vibegraph-lib/src/cache/mod.rs`:
  - `AssetKind::Pdf.env_var()` is `LHAPDF_DATA_PATH`.
  - `LHAPATH_VAR` and `RETIRED_PDF_DIR_VAR` are new.
- `vibegraph-cli/src/assets.rs`:
  - `DEV_PDF_FALLBACK` is gone.
  - `resolve_pdf_set_dir` reads both variables and calls `locate_pdf_set`.
  - `warn_on_retired_pdf_dir` warns once per process (a `Once`) when
    `VIBEGRAPH_PDF_DIR` is set. The warning names `$LHAPDF_DATA_PATH`.
  - The no-pin refusal and the no-home refusal now name `$LHAPDF_DATA_PATH`
    instead of `--pdf-dir` / `$VIBEGRAPH_PDF_DIR`.
- CVMFS, the index and unpinned downloads are not built.

## F1's Found items

- **Negative `nevents`.** It is refused at the parse
  (`RunCardError::NegativeEventCount`). `split` now `expect`s a non-negative
  value instead of clamping it to 0.
- **Rounding refusals.**
  - `GenerateError` is now an enum. It has `RoundingOffAverage { norm }`,
    `RoundingOnMixedMultiplicity { parts }`, `ProcessReparsed`, `ModelChanged`
    and `Other(String)`.
  - The library text says "the stochastic-rounding strategy … use the buffer
    strategy".
  - The CLI's `cli_error` maps both rounding variants back to
    `--strategy stochastic-rounding …; use --strategy buffer`. The off-average
    message adds "or --event-norm average".

## Deviations, and where the brief or the design is wrong or silent

- **Design §5 against the brief.** I found no conflict between them.
- **`a_different_pdf_set_is_refused`.** Split §4 says it "becomes the checksum
  refusal". No checksum is recorded until version 14 (H2), so that test cannot
  exist in H1. I replaced it with
  `cli_generate_proton::generate_locates_the_recorded_pdf_set_by_name`: the
  recorded set, missing from the data path and the cache, is refused by name.
  The checksum test is H2's.
- **The `cli_hard_errors.rs` call count is wrong in split §4.** It lists one
  `generate` call (L231). There were three:
  - L221, the zero-events test, which I migrated;
  - L568, `cli_generate_refuses_an_unimplemented_cut_naming_it_and_its_value`,
    which I deleted (the integrate counterpart stays);
  - L602, `cli_generate_refuses_the_run_card_before_reading_the_artifact`
    (#59's order test), which I deleted.
- **The migration list misses some sites.** `README.md` (two `generate`
  examples), `model/model-identity-digest.md` (it described `card_mismatches`)
  and `tooling/asset-resolution.md` are not on split §4's list; the last is on
  run inputs §7's. I migrated all three. The rendered `assets/demo_*.gif`
  still show the old command line, because the tapes were edited but not
  re-rendered.
- **`--pdf-dir` in tests.** The brief says integrate's flag stays, and it
  does. Following split §6 ("moving the tests once"), every test that passed
  `--pdf-dir validation/pdf` to `integrate` now sets `LHAPDF_DATA_PATH`
  through a new `common::PdfDataPath` helper. Two exceptions remain:
  - `cli_pdf_cache_fetched::an_explicit_pdf_dir_still_takes_precedence`
    tests the flag itself;
  - `cli_onshell_veto`'s `pp_z` row passes it inside an argument array, and
    H2 must change that row.
- **`common::vibegraph` also removes `LHAPDF_DATA_PATH` and `LHAPATH`.**
  Without that, a developer's own LHAPDF setting would reach the hermetic
  tests.
- **`cli_ufo_model`'s digest test.** It copies the vendored SMEFT model into
  a tempdir, integrates against the copy, checks that the unchanged copy
  generates, then overwrites `restrict_massless.dat` with
  `restrict_SMlimit_massless.dat` and expects the refusal. Appending a comment
  would not do: the digest is over the parsed model.

## Tests: new, changed, removed (file:line in the commit)

New refusals and their tests:

| refusal | test |
|---|---|
| a proc card, `--run-card`, `--pdf-set`, `--pdf-dir` passed to `generate` | `cli_generate.rs:598` `a_card_or_a_pdf_choice_passed_to_generate_is_an_argument_error` |
| missing `--seed` | `cli_generate.rs:634` `a_missing_seed_is_refused` |
| recorded proc card re-parses to another process | `cli_generate.rs:655`; lib `generate::tests::a_proc_card_that_reparses_to_another_process_is_refused` |
| model digest moved, interned SM | `cli_generate.rs:675`; lib `generate::tests::a_model_whose_digest_moved_is_refused` |
| model digest moved, UFO restrict card edited | `cli_ufo_model.rs:126` `a_restrict_card_edited_after_integrate_is_refused_by_digest` |
| negative `nevents` | lib `runcard::tests::a_negative_nevents_is_refused` |
| `$VIBEGRAPH_PDF_DIR` alone resolves nothing and warns once | `cli_pdf_cache.rs:234` |
| no cwd `validation/pdf` fallback | `cli_pdf_cache.rs:260` `a_checkouts_validation_pdf_is_not_a_fallback` |
| empty `LHAPDF_DATA_PATH` shadows `LHAPATH` | `cli_pdf_cache.rs:209`; lib `cache::resolve::tests::lhapath_is_read_only_when_lhapdf_data_path_is_unset` |
| rounding refusals name no flag (lib) / name `--strategy` (CLI) | lib `generate::tests::the_rounding_refusals_name_no_flag`; bin `generate::tests::a_rounding_refusal_names_the_strategy_flag`; `cli_generate.rs` `stochastic_rounding_refuses_an_event_norm_off_average` |

Other new tests:

- `cli_generate.rs:832` `the_event_norm_flag_overrides_the_recorded_value`:
  the average artifact under `--event-norm sum` writes the same `<init>` and
  events as the sum artifact.
- `cli_generate.rs:695` `an_existing_event_file_is_left_alone`: split out of
  the deleted card test.
- `cli_pdf_cache.rs:192`: the first data-path entry holding the set wins.
- `cache::resolve` unit tests:
  - `the_data_path_is_a_colon_list_without_empty_entries`;
  - `the_first_data_path_entry_holding_the_set_wins`;
  - `a_data_path_without_the_set_falls_through_to_the_cache`.
- `assets::tests::the_retired_pdf_dir_warning_names_lhapdf_data_path`.
- `cli_pdf_cache_fetched.rs:147`
  `lhapdf_data_path_serves_a_run_without_touching_the_cache` (extended
  validation).

The `generate` call sites, migrated to take no card (`--pdf-dir` → `LHAPDF_DATA_PATH`):

- `cli_generate.rs:149` (`generate_cmd`), `:209` (`generate_on`), `:637`
- `cli_generate_proton.rs:203`, `:646`, `:880`, `:1007`
- `cli_decay_chain_events.rs:210`, `:1121`, with the helper setting the data path at `:160`
- `cli_first_run.rs:210`
- `cli_decay.rs:442`
- `cli_reweight_proton.rs:174`
- `cli_ufo_model.rs:155`
- `cli_hard_errors.rs:221`
- `validate_samples_proton.rs:277`
- `cli_reference_docs.rs` follows the regenerated `docs/src/cli/reference.md`

Integrate `--pdf-dir` → `LHAPDF_DATA_PATH`: `cli_integrate.rs:68,255,301`,
`cli_decay_chain.rs:103`, `cli_generate_proton.rs:169,707,832,995`,
`cli_reweight_proton.rs:162`, `validate_samples_proton.rs:249`.

Removed:

- `cli_generate`: `a_card_that_did_not_train_the_grid_is_refused`,
  `an_event_norm_that_did_not_train_the_grid_is_refused` and the `refuse`
  helper.
- The dynamical-card `generate` leg of `cli_generate_proton`'s dynamical-scale
  test. Its `integrate` half stays.
- `cli_hard_errors`: the two tests above.
- The `generate.rs` unit tests of the comparison.

Scripts:

- `scripts/acceptance.sh:183,234`
- `scripts/demo_ee.tape:44`
- `scripts/demo_pp.tape:29,40`
- `validation/pythia/generate_samples.sh`
- `validation/pythia/generate_mlm_samples.sh`
- `validation/madgraph/gen_reweight_oracle.py`: a new `vibegraph_env()`
- `validation/madgraph/bench_reweight.py`
- `validation/manifest.toml`: the `cli-pdf-cache-fetched` rationale

Docs: `docs/src/cli/overview.md`, `docs/src/cli/reference.md` (regenerated by
`scripts/gen-cli-docs.sh`) and `README.md`.

Concepts:

- `events/generate.md`
- `pipeline/integrate-artifact.md`: the version was stale at 11; the contents
  table and the checks section are rewritten.
- `pipeline/artifact-format-versioning.md`: the floor paragraph.
- `tooling/release-acceptance.md`
- `tooling/asset-resolution.md`
- `model/model-identity-digest.md`

## Proton tests that could not run here

No PDF set and no MadGraph output exist in this container (`validation/pdf`
holds only scripts; `validation/madgraph/output` and `~/.vibegraph` are
absent). The following compiled and pass clippy under
`--features vibegraph/extended-validation`, but were not run:

- `cli_generate_proton`, including the new
  `generate_locates_the_recorded_pdf_set_by_name`;
- `cli_reweight_proton`;
- `cli_decay_chain_events`;
- `cli_decay_chain`;
- `cli_integrate`;
- `validate_samples_proton`;
- `cli_pdf_cache_fetched`, including the new `LHAPDF_DATA_PATH` test;
- `cli_onshell_veto::heavy`.

Nothing here exercised generation on proton beams end to end: neither the
`artifact.pdf_member` change nor generation from a set reached through
`LHAPDF_DATA_PATH`. The CI banked layer runs them.

## Mutations

The driver is `scratchpad/gpH1-F2-mutate.py`. It applies each mutation, runs
the named test, and restores the file. All 21 failed the test.

```
[0]  negative nevents check removed                       runcard::tests::a_negative_nevents_is_refused                          FAILED runcard.rs:1249
[1]  check_process always passes (lib)                    generate::tests::a_proc_card_that_reparses_to_another_process...       FAILED generate.rs:1441
[2]  check_process always passes (CLI)                    cli_generate::a_recorded_proc_card_that_parses_to_another_process...   FAILED cli_generate.rs:662
[3]  check_model always passes (lib)                      generate::tests::a_model_whose_digest_moved_is_refused                 FAILED generate.rs:1463
[4]  check_model always passes (CLI, interned SM)         cli_generate::a_model_whose_digest_moved_is_refused                    FAILED cli_generate.rs:682
[5]  check_model always passes (CLI, UFO restrict card)   cli_ufo_model::a_restrict_card_edited_after_integrate_is_refused...    FAILED cli_ufo_model.rs:177
[6]  --seed falls back to 20260728                        cli_generate::a_missing_seed_is_refused                                FAILED cli_generate.rs:643
[7]  generate accepts --pdf-set again                     cli_generate::a_card_or_a_pdf_choice_passed_to_generate_is_an_argument_error FAILED cli_generate.rs:626
[8]  generate accepts a positional proc card again        same                                                                   FAILED cli_generate.rs:626
[9]  --event-norm ignored                                 cli_generate::the_event_norm_flag_overrides_the_recorded_value         FAILED cli_generate.rs:837
[10] library rounding text names --strategy again         generate::tests::the_rounding_refusals_name_no_flag                    FAILED generate.rs:1419
[11] CLI passes the library's rounding text through       bin generate::tests::a_rounding_refusal_names_the_strategy_flag        FAILED generate.rs:425
[12] same, end to end                                     cli_generate::stochastic_rounding_refuses_an_event_norm_off_average    FAILED cli_generate.rs:862
[13] empty LHAPDF_DATA_PATH treated as unset (unit)       cache::resolve::tests::lhapath_is_read_only_when_lhapdf_data_path_is_unset FAILED resolve.rs:362
[14] same, CLI                                            cli_pdf_cache::lhapath_is_read_only_when_lhapdf_data_path_is_unset     FAILED cli_pdf_cache.rs:224
[15] empty data-path entries kept                         cache::resolve::tests::the_data_path_is_a_colon_list_without_empty_entries FAILED resolve.rs:341
[16] only the first data-path entry probed                cli_pdf_cache::lhapdf_data_path_resolves_the_first_entry_holding_the_set FAILED cli_pdf_cache.rs:203
[17] data path ignored by the CLI                         same                                                                   FAILED cli_pdf_cache.rs:203
[18] VIBEGRAPH_PDF_DIR read as a data-path entry          cli_pdf_cache::vibegraph_pdf_dir_resolves_nothing_and_draws_a_warning  FAILED cli_pdf_cache.rs:243
[19] no warning for VIBEGRAPH_PDF_DIR                     same                                                                   FAILED cli_pdf_cache.rs:252
[20] cwd validation/pdf fallback restored                 cli_pdf_cache::a_checkouts_validation_pdf_is_not_a_fallback            FAILED cli_pdf_cache.rs:265
```

## Gates

```
$ cargo fmt --all --check
exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 18.31s
exit 0
$ cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
    Finished `dev` profile [optimized] target(s) in 23.75s
exit 0
$ cargo test --workspace
exit 0
42 result lines: 1606 passed, 0 failed, 18 ignored
$ python3 scripts/kb.py lint
885 concepts, 293 backlog items; 0 errors, 4 warnings
```

The four lint warnings are the existing `state: blocked` backlog items. After
the gates ran, the two `vibegraph::ufo` imports in `generate.rs` were merged
into one line. Re-checked afterwards:

- `cargo fmt --all --check`: clean;
- extended clippy: `Finished`, no warnings;
- `cargo test -p vibegraph --bin vibegraph`: 98 passed.

## Found

- **The demo GIFs show the old command line.** `scripts/demo_{ee,pp}.tape`
  are migrated, but `assets/demo_ee.gif` and `assets/demo_pp.gif` (in the
  README) were rendered from the old tapes. To close it, re-render them with
  vhs.
- **`cli_onshell_veto`'s `pp_z` row still passes `--pdf-dir`**
  (`cli_onshell_veto.rs:324`). It works while `integrate` keeps the flag, and
  breaks when H2 removes it. To close it, switch the row to
  `LHAPDF_DATA_PATH` in H2.
- **A worker with `LHAPDF_DATA_PATH` set and no home directory is refused.**
  `resolve_pdf_set_dir` computes the cache root before it looks at the data
  path, so a host with no `$HOME` and no `$VIBEGRAPH_HOME` is refused even
  when the set is on the data path. To close it, resolve the cache root only
  when the cache step is reached.
- **The recorded `pdf_member` is now read, and nothing tests it.** `generate`
  loads `artifact.pdf_member`, which is always 0 today. The H2 test of a
  non-zero member covers it.
- **The design's checksum test for `generate` belongs to H2.** H1 tests
  location by name only.
- **The proton generate path is still unexercised end to end** (F1's Found
  item carries over). To close it, run the extended layer above with the
  fetched set before H1 merges.
