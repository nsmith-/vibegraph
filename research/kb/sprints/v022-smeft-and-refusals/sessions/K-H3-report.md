---
type: Session Report
title: "K-H3 report: hygiene pass on the run-card, CLI and batch PRs"
description: "Findings of the hygiene review of #143, #152 and #156 (written before any edit), the fixes with their mutations, the bit-for-bit oracles and what is left for Found."
status: draft
tags: [hygiene, cli, run-card, cuts, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-19}
---
Branch `sprintk-hyg-cuts` off main b3bc5399 ([brief](K-H3.md)). Reviewed:
`cce68116` (#143), `f78fb60b` (#152), `b3bc5399` (#156) and their direct callers
and tests. This section was committed before any code edit.

## Findings (before any edit)

Points: 1 maintainability, 2 non-vacuity, 3 visibility, 4 abstractions.
*checked* means the evidence below settles it.

| id | pt | site | evidence | confidence | disposition |
|---|---|---|---|---|---|
| H1 | 2 | `vibegraph-lib/src/cuts.rs` `a_nonpositive_xpt_compiles_to_the_default_cuts`, `a_nonpositive_rank_minimum_compiles_to_the_default_cuts`, `an_unbindable_or_negative_ht_maximum_compiles_to_the_default_cuts` | The same 12-line loop body is pasted into three tests. In the HT copy the `misset` / `-100` skip iterates `HT_MAXIMA` and can never fire (dead branch). In the other two the skip accepts a `ValueRefused` for `misset = -100` without asserting that the refusal happens, so deleting the `refuse_misset_floor` call leaves both green (to be shown by mutation M-H1). | checked (M-H1: the old tests pass the mutation, the new ones fail it) | fixed (`ca416bd9`) |
| H2 | 1/4 | `cuts.rs` `fn describe` and `runcard/report.rs` `fn describe` | Two byte-identical `match v { Float.. Int.. Bool.. Str\|Opaque => format!("'{s}'") }` bodies (`grep -n "^fn describe" vibegraph-lib/src/cuts.rs vibegraph-lib/src/runcard/report.rs`) | checked | fixed (`ca416bd9`) |
| H3 | 1 | `report.rs` `describe_card` | 163 lines (379-541), added by #152, no named reason | checked | fixed (`ca416bd9`) |
| H4 | 1/4 | `report.rs` `scan` and `runcard.rs` `RunCard::parse` | `scan` repeats `parse`'s line loop (`split_line`, `canonical_name`, `parse_value`, the `auto` list rule, `written`), differing only by collecting errors instead of returning the first | checked | fixed (`ca416bd9`) |
| H5 | 2 | `vibegraph-cli/tests/cli_check_only.rs` | Every `--check` test uses fixed-energy beams (`grep -rn '"--check"' vibegraph-cli/tests`: four files, all `lpp = 0`). The `if args.check { return Ok(None) }` in `integrate_hadronic` has no test: deleting it passes the suite | checked | fixed (`363a3c68`) |
| H6 | 2 | `cli_check_only.rs` `the_parity_cases_span_distinct_refusal_kinds` | Re-lists three of the four parity cards by hand (the forbidden-propagator case is missing) and hard-codes the count 3; a card added to the parity list is not spanned | checked | fixed (`363a3c68`) |
| H7 | 2 | `cli_run_card.rs` `the_arguments_are_checked` | Two of three cases assert only `!status.success()` | checked | fixed (`363a3c68`) |
| H8 | 3/bug | `vibegraph-lib/src/generate.rs` `generate_sample` (`emit_to` then `refuse_sampled_fault`, line 655) and `generate_proton_sample` (line 1227) | `emit_to` renames the file onto `options.out` when the write succeeds; a sampled-point fault is only read afterwards, so a refused run (an error exit) leaves a complete event file, drawn from points the fault weighted zero, under the final name, where a batch wrapper that reads the file instead of the exit status takes it for a result. The audit-failure path removes the file; this one does not | checked (by reading; no card reaches a sampled fault, so the test drives the helper) | fixed (`387891f4`) |
| H9 | 4 | `proton.rs:3744` and `hadronic.rs:2780` (`fault` / `record_fault`), `vibegraph-cli/src/integrate.rs:1646` and `vibegraph-lib/src/generate.rs:276` (`refuse_sampled_fault`) | The same one-slot `OnceLock<SampledPointFault>` with its accessor and setter, and the same `fault -> refusal` conversion, in two integrands and in two callers | checked | Found |
| H10 | 1 | `cli_run_card.rs` `trait StatusOf` | A one-use extension trait for `Value["status"].as_str()` | checked | fixed (`363a3c68`) |
| H11 | 1 | `report.rs` `classification_table` | A `BTreeSet` of the unimplemented names is built per call and read only by a `debug_assert!` | checked | fixed (`ca416bd9`) |
| H12 | 2 | `cuts.rs` `a_negative_misset_is_refused_where_its_square_is_the_lepton_floor` doc | A 130-column doc line run together by hand (`a tie (...) leaves the floor where it was, and every`) | checked | fixed (`ca416bd9`) |
| H13 | 2 | K-N1's three `refuse_sampled_fault(integ.fault())?` call sites in `integrate_fixed_energy` and `generate_sample` | The only test drives the integrand seam; no card reaches a sampled fault (the test's own doc says so), so deleting a call site passes every test | checked | leave (no mutation reaches it) |
| H14 | 1 | `cli_batch_stop.rs` `the_first_term_stops_a_plain_text_run_too` | Inlines the `kill` shell-out that `Run::signal` already holds | checked | leave |
| H15 | 4 | `vibegraph-cli/src/generate.rs` `output_encoding` and `vibegraph-lib/src/generate.rs` `OutputEncoding::of_path` | The lower-cased extension is read twice, once to refuse and once to map | checked | leave |
| H16 | 1 | `diagrams/mod.rs` `generate_sets_inner` (240 lines), `generate.rs` `generate_sample` / `generate_proton_sample` (232 / 233), `cuts.rs` `compile_exempting` (109, 108 before) | Each already past 100 lines before the PRs and grew by a few | checked | leave |
| H17 | 2 | `report.rs` `VALUE_REFUSALS` | A hand-kept list of the refusals no field class records; `every_listed_parameter_has_an_entry` checks it one way (every listed name has an entry), nothing can check that a new `RunCardError` refusal has a row | checked | Found |

## What changed

- `runcard/scan.rs` (new, private): the line scanner. `RunCard::parse` returns its first error, the report lists all of them.
- `runcard/report.rs`: `describe_card` is five functions (`scan_of_default`, `resolve`, `stage_refusal`, `key_report`, `process_defaults`); one `describe` (now `pub(crate)` in `cuts.rs`).
- `generate.rs`: `refuse_emitted_fault` removes the event file with the refusal; new test `a_fault_after_the_file_is_in_place_removes_the_file`.
- tests: one `assert_compiles_to_default` helper that requires the `misset = -100` refusal exactly on sets with a counted lepton; `cli_check_only` gains the hadronic `--check` test (gated on `extended-validation`, it needs the PDF set, and calls `validation::require` when the set is absent) and one shared case list; `the_arguments_are_checked` names the clap refusals.

## Oracles

**Refactor identity** (`describe_card` split, `parse` merge): a temporary test dumped `RunCard::parse` (Debug or error) and `describe_card` (4 decay/process-default combinations) for the 21 committed `*run_card.dat`, 18 inline cards (unknown name, bad value, `$` placeholder, ignored physics, xqcut, ickkw, `auto` lists, comments) and the two no-card forms, before and after: `cmp` of the two 12,221,046-byte dumps exits 0, sha256 `f7a78ffef529392836d26d0db08c09a8dc74c7ed3dcffd992416f8d916152dee` for both (`KH3_OUT=... cargo test -p vibegraph-lib --test kh3_oracle`, file deleted before commit).

**σ and grids against main b3bc5399** (built from `git archive` in its own target dir, deleted after), `integrate <proc> --run-card <card> --fixed-budget --neval 20000 --niter 3 --seed 7 -j 2 --no-tui --no-network`, `LHAPDF_DATA_PATH=validation/pdf`:

| run | σ (both binaries) | grid.bin.zst sha256 (both, `cmp` exits 0) |
|---|---|---|
| `e+ e- > mu+ mu-`, `config_share_ee_mumu` card | 419.72 ± 0.75 fb | 934120b117f2e527edb2559840fb60bdf3b5843742aed0d169b732d00a72b29b |
| `p p > e+ e-`, `dy13_default` | 935 ± 11 pb | 9f51be30c2db15b8ebf5bae6ace83c49d64c8162b81aae8193df22e717b7af50 |
| `p p > e+ e- j`, `pp_to_llj_mlm` | 267.0 ± 2.5 pb | 95f0c1bf6b50921b733af23cd76202ecb85a9e0f21a322ca3916ca84b81d084d |

## Mutations

Driven by one script: a temporary `kh3(id)` switch read from `KH3_MUT` at each site, one build, then each test run alone under its id (`none` rows are controls: 2 of them and a full run of six CLI test files all pass without a switch). The patch is not committed. All 72 rows below fail under their switch (rc 101, `test result: FAILED`); the controls pass.

| id | test | mutation |
|---|---|---|
| c01 | the_rank_base_card_admits_every_point | `RankCut::of` always active |
| c02 | jet_rank_cuts_read_the_pt_ordering_not_the_leg_order | insertion sort off |
| c03 | fewer_members_than_the_rank | existence test off |
| c04 | only_four_ranks_carry_bounds | rank 4 not checked |
| c05 | the_jet_family_follows_maxjetflavor | b legs ranked as jets |
| c06 | lepton_rank_cuts | lepton rank dropped |
| c07 | rank_cuts_apply_to_exempt_decay_products | members need `do_cuts` |
| c08 | a_rank_minimum_over_a_split_exemption_is_refused | overcount refusal off |
| c09 | cutuse_is_refused_only_with_a_jet_rank_bound_on | max ceiling ignored |
| c10 | a_rank_floor_on_every_member_regulates_its_limits | soft-limit regulation by rank floor off |
| c11 | a_single_jet_rank_cut_is_its_single_leg_cut | `<=` at the minimum |
| c12 | the_shat_floor_reads_the_rank_minima | rank sum left out of the floor |
| c13 | a_nonpositive_rank_minimum_compiles_to_the_default_cuts | negative minimum active |
| c14 | a_positive_unimplemented_minimum_is_refused_by_name | `x <= 1e9` is off |
| c15 | an_unbindable_or_negative_ht_maximum_compiles_to_the_default_cuts | negative HT max refused |
| c16 | a_bindable_ht_maximum_is_refused_by_name | threshold halved |
| c17 | the_ht_minima_stay_refused | `ht2min` read as an off sentinel |
| c18 | the_inert_value_description_follows_the_rules | HT rule text dropped |
| c19 | the_lepton_floor_term_is_the_smin_madevent_prints | last leg counted |
| c20 | a_negative_misset_is_refused_where_its_square_is_the_lepton_floor | `>=` for `>` (tie refused) |
| c21 | each_threshold_withdraws_the_claim_that_nothing_is_rejected | rank filters ignored by `admits_every_point` |
| c22 | a_nonpositive_xpt_..., a_nonpositive_rank_minimum_..., a_negative_misset_... | `refuse_misset_floor` call skipped: all three fail. The pre-change `a_nonpositive_xpt_...` and `a_nonpositive_rank_minimum_...` (copied back as `old_*`) both pass it (H1) |
| c23 | a_refused_scale_on_a_sampled_point_is_a_fault_and_not_a_panic | `record_fault` a no-op |
| r01-r12 | the 12 `report.rs` tests, in order | Ignored read as Honoured; inert branch off; note keeps no period; `moved_by_resolution` forced; resolved read from the scan; `$` placeholder dropped; ignored-physics refusal off; detector's key refusal off; warnings dropped; process defaults empty; process default ignored; resolved read as the default |
| r13-r18 | the 6 content `cli_run_card` tests | `misset` refused as a sentinel; cutuse key refusal off; unknown line off by one; decay process default dropped; `overridden_by_card` false; `misset` words dropped |
| a01 | the_arguments_are_checked | clap group not required |
| e01, e02 | enumeration_carries_... (lib), enumerate_progress_carries_... (cli) | `line` reports `lines`; `total` is a sum |
| k01-k05 | the 5 `cli_check_only` tests | `budget` moved after the check; refusal kind changed (k02, which the parity test does not see and the span test does); early return off (k03, k05); `check ok` line dropped (k04) |
| k06-k10 | the 4 stop-reason tests and `the_stop_reasons_serialize_by_name` | one mapping arm each |
| k11, k12, s01, s02 | plain-text stop, stop before kept, second signal, first signal | `wrote` line dropped; refusal off; second signal also a stop request (also fails the unit test); `sampling_started` a no-op |
| p01-p05 | `the_default_is_the_smallest_limit...` and the 4 `cli_default_threads` tests | `nth(1)`; available fixed at 8; comparison inverted; comparison always true; explicit count logged |
| g01-g07 | the 6 `cli_generate_output` tests and the CLI unit test | default encoding gzip; gz as zstd; zst as gzip; bz2 not refused; copy for rename; no partial name; case kept |
| g08 | a_fault_after_the_file_is_in_place_removes_the_file | file not removed |

Sites no mutation reaches: the three `refuse_sampled_fault(integ.fault())?` calls K-N1 added to `integrate_fixed_energy` and `generate_sample` (a sampled point does not reach a scale refusal on any card, and the only test drives the integrand seam), and the same call on the proton paths; `the_stop_reasons_serialize_by_name` pins `constant` only by its serde name.

## What the tests cannot detect

- The identity oracle sees only the corpus above: a parse or report difference on a card no one wrote is unseen.
- `a_refused_scale_...` and the call-site gap above: a deleted `refuse_*` call passes.
- The hadronic `--check` test runs only under `extended-validation` (it needs the PDF set); the default suite has no hadronic `--check` test.

## Gate

- `cargo fmt --all --check`: `fmt-ok`
- `cargo clippy --workspace --all-targets -- -D warnings`: `Finished dev profile ... in 30.57s`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: `Finished dev profile ... in 38.79s`
- `cargo test --workspace`: 69 binaries, 2237 passed, 0 failed, 26 ignored, exit 0. Net tests added: `generate::tests::a_fault_after_the_file_is_in_place_removes_the_file` (default suite) and `cli_check_only::a_hadronic_card_reports_ok_without_sampling_or_an_artifact` (extended-validation only). None deleted.
- `cargo test -p vibegraph --features extended-validation --test cli_check_only --test cli_run_card`: 5 and 7 passed. These touch the hadronic `--check` path and the run-card report; no amplitude, colour, coupling or sampling code changed, so no `validate_*` gate applies.
- `cargo doc -p vibegraph-lib --no-deps --document-private-items`: 5 unresolved-link, 4 redundant-link and 1 module/macro warnings, none naming an item this pass touched (`graph::ChannelSet`, `ScaledChannel::ndim`, `parse_couplings`, `parse_particles`, `adapt_blocks_iteration_shared`); `-p vibegraph`: `Generated .../doc/vibegraph/index.html`, no warnings.
- `pixi run --skip-deps kb-lint`: `1301 concepts, 433 backlog items; 0 errors, 5 warnings` (blocked-without-`blocked_by` items, untouched).

## Method

Reading each diff against the checklist (about 40%), mutation-driven reading of each test (30%), grep for second copies (15%), the Debug-digest oracle and gates (15%). The one-build, env-switch mutation driver turned 72 mutations into one 5-minute build and a 15-minute run; it fits any tree of tests that share a crate. Leads that did not hold: no comment in the three diffs references a plan, session or history (the grep of the checklist returned nothing); no new `pub` lacks an outside user; `compile_exempting` was already 108 lines at the base.

## Found

1. **A shared sampled-point fault slot** (H9). `proton.rs` `ProtonIntegrand::fault`/`record_fault` and `hadronic.rs` `FixedBeamIntegrand::fault`/`record_fault` are the same `OnceLock<SampledPointFault>` with the same docs, and `refuse_sampled_fault` exists once in `vibegraph-cli/src/integrate.rs` and once in `vibegraph-lib/src/generate.rs`. Close with one `FaultSlot` type in `proton.rs` and one conversion on `SampledPointFault`.
2. **`VALUE_REFUSALS` is hand kept** (H17). A new `RunCardError` refusal for a consumed parameter has no row unless someone writes one, and no test can tell. Close by giving `RunCardError` a `parameter()` accessor and a test that every variant naming a parameter has a row.
3. **No test reaches the sampled-fault call sites** (H13). Closing it needs a card whose sampled point raises a scale fault, which does not exist yet; until then a deleted `refuse_*` call is invisible.
4. **A refused `generate` run leaves the file when the refusal is not a sampled fault** is not claimed: only the sampled-fault path was checked.
5. `generate_sets_inner` (240 lines), `generate_sample` and `generate_proton_sample` (about 230 each) were over 100 lines before these PRs and grew by a few lines each.

## Brief corrections

- The brief names run cards but not proc cards for the three bit-for-bit runs; `e+ e- > mu+ mu-`, `p p > e+ e-` (the dy13 proc card) and `p p > e+ e- j` were used.
- `K-H3.md` says `closes: []`, so no backlog item file was deleted.
- Disk: the brief gives about 24 GB free; the container had 9 GB at the start and 4 GB after the baseline build, which forced the baseline's target dir to be deleted right after use.
