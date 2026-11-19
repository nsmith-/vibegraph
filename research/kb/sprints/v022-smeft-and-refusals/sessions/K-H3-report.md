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
| H1 | 2 | `vibegraph-lib/src/cuts.rs` `a_nonpositive_xpt_compiles_to_the_default_cuts`, `a_nonpositive_rank_minimum_compiles_to_the_default_cuts`, `an_unbindable_or_negative_ht_maximum_compiles_to_the_default_cuts` | The same 12-line loop body is pasted into three tests. In the HT copy the `misset` / `-100` skip iterates `HT_MAXIMA` and can never fire (dead branch). In the other two the skip accepts a `ValueRefused` for `misset = -100` without asserting that the refusal happens, so deleting the `refuse_misset_floor` call leaves both green (to be shown by mutation M-H1). | suspected until M-H1 runs | fix: one helper, which requires the refusal exactly on the sets with a counted lepton |
| H2 | 1/4 | `cuts.rs` `fn describe` and `runcard/report.rs` `fn describe` | Two byte-identical `match v { Float.. Int.. Bool.. Str\|Opaque => format!("'{s}'") }` bodies (`grep -n "^fn describe" vibegraph-lib/src/cuts.rs vibegraph-lib/src/runcard/report.rs`) | checked | fix: one copy |
| H3 | 1 | `report.rs` `describe_card` | 163 lines (379-541), added by #152, no named reason | checked | fix: split (identity oracle: a `Debug` digest of `describe_card` over every committed run card, the decay default and malformed cards, before and after) |
| H4 | 1/4 | `report.rs` `scan` and `runcard.rs` `RunCard::parse` | `scan` repeats `parse`'s line loop (`split_line`, `canonical_name`, `parse_value`, the `auto` list rule, `written`), differing only by collecting errors instead of returning the first | checked | fix: `parse` reads the scan and returns its first error (identity oracle: digest of `parse` over the same corpus) |
| H5 | 2 | `vibegraph-cli/tests/cli_check_only.rs` | Every `--check` test uses fixed-energy beams (`grep -rn '"--check"' vibegraph-cli/tests`: four files, all `lpp = 0`). The `if args.check { return Ok(None) }` in `integrate_hadronic` has no test: deleting it passes the suite | checked | fix: add a hadronic `--check` test |
| H6 | 2 | `cli_check_only.rs` `the_parity_cases_span_distinct_refusal_kinds` | Re-lists three of the four parity cards by hand (the forbidden-propagator case is missing) and hard-codes the count 3; a card added to the parity list is not spanned | checked | fix: both tests read one case list |
| H7 | 2 | `cli_run_card.rs` `the_arguments_are_checked` | Two of three cases assert only `!status.success()` | checked | fix: assert the clap refusal each case must raise |
| H8 | 3/bug | `vibegraph-lib/src/generate.rs` `generate_sample` (`emit_to` then `refuse_sampled_fault`, line 655) and `generate_proton_sample` (line 1227) | `emit_to` renames the file onto `options.out` when the write succeeds; a sampled-point fault is only read afterwards, so a refused run (an error exit) leaves a complete event file, drawn from points the fault weighted zero, under the final name, where a batch wrapper that reads the file instead of the exit status takes it for a result. The audit-failure path removes the file; this one does not | checked (by reading; no card reaches a sampled fault, so the test drives the helper) | fix with a test failing before and passing after |
| H9 | 4 | `proton.rs:3744` and `hadronic.rs:2780` (`fault` / `record_fault`), `vibegraph-cli/src/integrate.rs:1646` and `vibegraph-lib/src/generate.rs:276` (`refuse_sampled_fault`) | The same one-slot `OnceLock<SampledPointFault>` with its accessor and setter, and the same `fault -> refusal` conversion, in two integrands and in two callers | checked | Found (a shared type) |
| H10 | 1 | `cli_run_card.rs` `trait StatusOf` | A one-use extension trait for `Value["status"].as_str()` | checked | fix: a closure |
| H11 | 1 | `report.rs` `classification_table` | A `BTreeSet` of the unimplemented names is built per call and read only by a `debug_assert!` | checked | fix |
| H12 | 2 | `cuts.rs` `a_negative_misset_is_refused_where_its_square_is_the_lepton_floor` doc | A 130-column doc line run together by hand (`a tie (...) leaves the floor where it was, and every`) | checked | fix: reflow |
| H13 | 2 | K-N1's three `refuse_sampled_fault(integ.fault())?` call sites in `integrate_fixed_energy` and `generate_sample` | The only test drives the integrand seam; no card reaches a sampled fault (the test's own doc says so), so deleting a call site passes every test | checked | leave: no mutation reaches the sites (named in the report) |
| H14 | 1 | `cli_batch_stop.rs` `the_first_term_stops_a_plain_text_run_too` | Inlines the `kill` shell-out that `Run::signal` already holds | checked | leave: it reads stderr line by line, which `Run` does not offer; merging costs more than the copy |
| H15 | 4 | `vibegraph-cli/src/generate.rs` `output_encoding` and `vibegraph-lib/src/generate.rs` `OutputEncoding::of_path` | The lower-cased extension is read twice, once to refuse and once to map | checked | leave: different crates, one `match` each, the refusal needs the name |
| H16 | 1 | `diagrams/mod.rs` `generate_sets_inner` (240 lines), `generate.rs` `generate_sample` / `generate_proton_sample` (232 / 233), `cuts.rs` `compile_exempting` (109, 108 before) | Each already past 100 lines before the PRs and grew by a few | checked | leave (no new function crossed 100; splitting hot or large pre-existing generators is not local hygiene) |
| H17 | 2 | `report.rs` `VALUE_REFUSALS` | A hand-kept list of the refusals no field class records; `every_listed_parameter_has_an_entry` checks it one way (every listed name has an entry), nothing can check that a new `RunCardError` refusal has a row | checked | Found (a blind spot, not derivable) |

## Method notes (filled in below as the work proceeds)
