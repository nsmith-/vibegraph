---
type: Session Report
title: "F1 report: the run card split by stage, GenerateSettings, artifact version 13 and the library generation entry point"
description: "FIELD_CLASSES carries a stage (14 Generate fields); RunCard::split narrows the card and yields GenerateSettings; iseed seeds integrate; artifact v13 records proc_card, the narrowed card and the generate settings, with the v3-v12 readers and guards deleted; the sampling driver moved to vibegraph::generate::generate. Event files are byte-identical to the baseline on five fixed-energy and decay runs."
status: draft
tags: [gridpack-generate, artifact, run-card, generate, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-21}
---
Worktree `/home/user/wt/H1`, branch `sprintd-gridpack-generate`, from
`c34b693`. The design is
[integrate and generate split by data dependency](../../../pipeline/integrate-generate-split.md).

## Implemented

1. **Stage column.** `FIELD_CLASSES` rows are `(name, Stage, FieldClass)`, and
   `Stage` is `Integrate` or `Generate` (`runcard/classes.rs`). The 14 Generate
   rows follow the §1 table: `nevents`, `event_norm`, `use_syst`, the two
   `systematics_*` and five `sys_*` fields, `clusinfo`, `time_of_flight`,
   `boost_event` and `lhe_version`. `iseed` is `Integrate` and is now `Consumed`
   (`R_ISEED`). The `B_JOB` sentence saying the seed comes from the CLI is gone.
2. **`RunCard::split(self) -> (RunCard, GenerateSettings)`** (`runcard.rs`).
   - It removes every Generate field from the card.
   - `nevents` is no longer a struct field, and `RunCard::event_norm()` is gone.
   - `use_syst` goes to `GenerateSettings::systematics.enabled`. `alpsfact`
     stays as resolved.
   - The three Generate fields that are refused off their default are dropped.
   - `GenerateSettings { nevents: u64, event_norm, systematics: Systematics,
     clustering_info }` lives in `vibegraph::generate`. `Systematics` is
     `{ enabled, fields: Vec<(name, card text)> }`: verbatim strings, as N6
     decides.
3. **Seed (N4).** `RunCard::integration_seed(flag)` returns the flag, else a
   non-zero `iseed`, else `DEFAULT_INTEGRATION_SEED` (20260719, the old CLI
   default). `integrate --seed` is now `Option<u64>`. The artifact's `seed` is
   the seed actually used.
   - A negative `iseed` is refused at the parse (`RunCardError::NegativeSeed`).
     The brief did not cover it.
4. **Artifact version 13** (`artifact.rs`).
   - It adds `proc_card: String` (verbatim, read through the new
     `read_proc_card_text`) and `generate: GenerateSettings`, and `run_card` is
     the narrowed card.
   - `OLDEST_READABLE_VERSION = FORMAT_VERSION = 13`. The reader decodes 13 and
     refuses anything else with the existing `UnsupportedVersion`.
   - Deleted: the `v3`/`v4`/`v5`/`v7`/`v9` readers, `version_for`,
     `refuse_unmerged_grids`, `MAP_CHOICES_VERSION`, `MULTIPLICITY_VERSION`,
     `MERGED_CHANNEL_VERSION`, `GRID_RULE_VERSION`, `SCALE_DRAW_VERSION`,
     `UnrecordableGridRule`, and the CLI's
     `refuse_stale_artifact_on_clustering_scale` and
     `refuse_stale_artifact_on_mixed_multiplicity`.
   - `RunCard::banner_overrides` is now serialized; it was `#[serde(skip)]`
     before. See "Deviations".
5. **Library entry point (N10).** `vibegraph::generate::generate(&IntegrateArtifact,
   &GenerateAssets, &GenerateSettings, &GenerateOptions)` (`vibegraph-lib/src/generate.rs`,
   with `generate/source.rs` and `generate/audit.rs`). It takes no `RunCard`; it
   reads `artifact.run_card`.
   - `GenerateAssets` holds the parsed proc card, the model, the evaluated
     model and, on proton beams, `(PdfSet, PdfMember)`.
   - `GenerateOptions` holds the seed, the strategy, the scan budget, the
     max rule, the reweight launches, the enumeration pool, the output path,
     the artifact label for the header, and an `on_scan` efficiency callback
     (the TUI hook).
   - The CLI keeps its argument surface. It still loads the cards, splits the
     run card, compares the narrowed card and the generate settings with the
     artifact, resolves the reweight card, applies `--nevents` to
     `artifact.generate`, and calls the library.
   - To make the move possible, `multiplicity_groups`, `multiplicity_maps`,
     `MultiplicityGroups` and `refuse_mixed_multiplicity` moved to
     `vibegraph::multiplicity`. `initial_state` became
     `InitialState::from_run_card`, and `is_decay` became
     `SupportedCard::is_decay`.
6. **`<MGRunCard>` (N9).** It is now `mg_run_card(card, settings, seed)`, built
   from `RunCard::banner_with`: the integrate card's MadGraph record, then the
   settings as used, then the refused Generate fields at their defaults. `iseed`
   is the generation seed when it fits an `i32`, and the card's `iseed`
   otherwise.

## Deviations, and where the brief or the design is silent or wrong

- **`banner_overrides` is serialized.** The design does not mention it. The old
  generate wrote `<MGRunCard>` from a card it had just parsed, which carried
  MadGraph's record (`ptj`/`mmjj` under `xqcut`, `alpsfact`). The library reads
  the deserialized card, which carried none. Without the change, a matched
  sample's banner would have reported the rewritten `ptj = xqcut`.
  `a_split_card_keeps_madgraphs_record_through_an_encoding` pins it.
- **`for_decay` on a narrowed card.** `use_running_coupling` derives a decay's
  card through `from_values`, which reads `event_norm` and `use_syst`. On the
  narrowed card both are absent, and it would have panicked. `for_decay` now
  fills the missing names from the defaults and then keeps only the names the
  input had.
- **N9 was implemented here.** The brief does not list it, but the library
  entry point cannot write the banner from a `RunCard` it no longer has.
  `validate_mlm_dumps.rs` now passes seed 0, and its doc no longer says
  vibegraph records the card's `iseed`.
- **The CLI still compares the generate settings** (`settings_mismatches`),
  because F1 keeps the CLI surface. `an_event_norm_that_did_not_train_the_grid_is_refused`
  and the other card tests therefore still pass. F2 removes the comparison
  together with the card arguments.
- **Refusal messages moved into the library unchanged.** The two
  stochastic-rounding refusals in `vibegraph::generate` still name
  `--strategy`, because the CLI tests match that text. A non-CLI caller sees
  flag names.
- **The CLI's v12 test** (`a_version_12_artifact_is_refused`) needs `zstd` as a
  `vibegraph-cli` dev-dependency, to stamp `run()`'s artifact down to version
  12.
- **Brief and design §5 agree.** I found no conflict.

## What moved where

- `vibegraph-cli/src/generate.rs` keeps the args, card comparison,
  `settings_mismatches` (L349), `reweight_launches` (L390) and `run` (L425).
  Everything else moved:
  - into `vibegraph-lib/src/generate.rs`: `generate` (L165),
    `generate_sample` (L222), `report_scan` (L516), `file_header` (L579),
    `check_channel_keys` (L663), `generate_proton_sample` (L704),
    `check_alphas` (L1019) and the rest of the driver;
  - into `generate/source.rs`: `SampleSource` (L32), `ResonanceTally`
    (L194), `PartRecords` (L355), `ProtonSampleSource` (L370) and the record
    builders;
  - into `generate/audit.rs`: `ReweightAudit` (L32).
- `vibegraph-lib/src/runcard.rs`: `DEFAULT_INTEGRATION_SEED` L238,
  `integration_seed` L341, `split` L362, `banner_with` L394, `for_decay` L548,
  `NegativeSeed` L181.
- `vibegraph-lib/src/runcard/classes.rs`: `Stage` L214, `FIELD_CLASSES` L225,
  `generate_fields` L438.
- `vibegraph-lib/src/multiplicity.rs`: `multiplicity_groups` L88,
  `multiplicity_maps` L117, `refuse_mixed_multiplicity` L137.
- `vibegraph-lib/src/hadronic.rs`: `InitialState::from_run_card` L904.
- `vibegraph-lib/src/diagrams/check.rs`: `SupportedCard::is_decay` L66.
- `vibegraph-lib/src/lhef/write.rs`: `mg_run_card` L288.
- `vibegraph-cli/src/integrate.rs`: split and seed at L637-638; `proc_card` and
  `generate` recorded at L760.
- `vibegraph-cli/src/main.rs`: `read_proc_card_text` L38.
- `docs/src/cli/reference.md` was regenerated by `scripts/gen-cli-docs.sh`,
  since the `--seed` help changed.

## Equivalence with the baseline

The baseline binary was built from `c34b693` sources. Each binary ran
`integrate --fixed-budget --neval 20000 --niter 4`, then
`generate --nevents 2000 --seed 20260728`, from the same cards
(`scratchpad/gpH1-F1-compare.sh`):

```
ee_mumu (buffer)              σ = 2024.195880 ± 2.546394 pb both; event files IDENTICAL (2000 events)
ee_mumu (stochastic-rounding) σ = 2024.195880 ± 2.546394 pb both; event files IDENTICAL (2000 events)
ee_mumu (event_norm = sum)    σ = 2024.195880 ± 2.546394 pb both; event files IDENTICAL (2000 events)
e+ e- > u u~ g (250 GeV)      σ = 0.345104 ± 0.001432 pb both;    event files IDENTICAL (2000 events)
t > b e+ ve (decay)           Γ = 1.572714e-1 ± 2.723842e-4 GeV;  event files IDENTICAL (2000 events)
```

The proton path, and so the `<MGRunCard>` banner of a matched sample, could not
be run. No PDF set exists in this container (`validation/pdf` and
`~/.vibegraph` are absent), so the extended layer was type-checked and linted
but not run.

## New tests and mutation evidence

Each mutation was applied, the named test run, and the file restored
(`scratchpad/gpH1-F1-mutate.py`).

| test | mutation | result |
|---|---|---|
| `classes::every_run_card_field_has_one_stage_and_fourteen_are_generate` | `clusinfo` row moved to `Integrate` | FAILED, classes.rs:640 `left == right` |
| `runcard::the_integration_seed_is_the_flag_then_iseed_then_the_default` | `iseed` ignored (`None if false && …`) | FAILED, runcard.rs:1221 |
| same | `iseed` beats the flag | FAILED, runcard.rs:1224 |
| `runcard::a_negative_iseed_is_refused` | `NegativeSeed` check removed | FAILED, runcard.rs:1230 |
| `runcard::split_hands_the_generate_fields_to_the_settings` | `remove` → `get().cloned()` | FAILED, runcard.rs:1165 |
| `runcard::a_split_card_keeps_madgraphs_record_through_an_encoding` | `#[serde(skip)]` restored on `banner_overrides` | FAILED, runcard.rs:1205 |
| `runcard::a_decay_card_of_a_split_card_holds_the_same_parameters` | `retain` removed from `for_decay` | FAILED, runcard.rs:1246 |
| `artifact::a_version_12_artifact_is_refused` | reader arm `12 \| FORMAT_VERSION` | FAILED, artifact.rs:714 |
| `lhef::write::mg_run_card_records_the_generation_as_drawn` and `mg_run_card_carries_what_set_mad_reads` | banner `iseed` set to 0 | both FAILED, write.rs:868 and :838 "iseed is the generation's seed" |
| `lhef::write::mg_run_card_records_the_generation_as_drawn` | banner `event_norm` ignores the settings | FAILED, write.rs:866 |
| `generate::a_proton_artifact_without_a_pdf_set_is_refused` | a missing PDF set falls to the fixed-beam path | FAILED, generate.rs:1368 |
| CLI `generate::tests::a_changed_generation_setting_is_refused` | `settings_mismatches` compares the artifact with itself | FAILED, generate.rs:681 `left == right`: `[]` |
| `cli_generate::an_event_norm_that_did_not_train_the_grid_is_refused` (existing) | the `settings_mismatches` call removed from `run` | FAILED, cli_generate.rs:806 |
| `cli_generate::a_version_12_artifact_is_refused` | reader arm `12 \| FORMAT_VERSION` | FAILED, cli_generate.rs:703 |

The moved tests run in the library unchanged:
`a_refused_reweighting_removes_its_file`,
`stochastic_rounding_is_refused_on_a_sum_over_multiplicities`,
`channel_weights_must_be_a_selection_distribution` and
`restarting_the_source_replays_the_same_events`. Deleted with their code: the
v3/v4/v5/pre-maps upgrade tests, the v10/v11/v12 write-down tests,
`write_refuses_a_rule_its_version_cannot_record`, and the two CLI
pre-draw-artifact tests. The two `cli_generate_proton` stale-version blocks
were also removed.

## Gates

```
$ cargo fmt --all --check
(no output, exit 0)

$ cargo clippy --workspace --all-targets -- -D warnings
exit 0
$ cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
(no warnings)
$ cargo check --workspace --all-targets --features vibegraph/extended-validation
exit 0

$ cargo test --workspace
exit 0; 42 result lines: 1597 passed, 0 failed, 18 ignored
  (vibegraph-lib unit tests: 1048 passed, 12 ignored)

$ python3 scripts/kb.py index
wrote 1 index files
$ python3 scripts/kb.py lint
884 concepts, 293 backlog items; 0 errors, 4 warnings
  (the 4 warnings are pre-existing backlog items with `state: blocked` and no `blocked_by`)
```

## Found

- **The proton generate path was not exercised end to end in this session.**
  No PDF set is reachable here, so `generate_proton_sample` was checked only
  for compiling and by its unit-level guards. That includes the matched
  `<MGRunCard>` now read from the serialized `banner_overrides`. To close it,
  run `cli_generate_proton`, `validate_samples_proton` and
  `cli_reweight_proton` with the fetched set before H1 merges, and
  `validate_mlm_dumps::mg_run_card_matches_madevents_banner` against the
  oracle runs.
- **The library's refusal messages name CLI flags** (`--strategy`, in
  `refuse_rounding_*`). To close it, phrase them in library terms and let the
  CLI add the flag name. F2 rewrites the CLI tests anyway.
- **The `nevents` refusal message lost its value.** The CLI's "nevents = N"
  message now always prints 0, because `GenerateSettings::nevents` clamps a
  negative card value to 0. To close it, refuse a negative `nevents` at the
  parse, as `iseed` now is.
