---
type: Session Report
title: "HY1 report: hygiene review of the early cut refusal"
description: "Findings table (written before any edit), dispositions, mutation evidence and gates."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-20}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-20}
---
## Findings (written before any edit)

| id | pt | site | evidence | conf. | planned disposition |
|---|---|---|---|---|---|
| H1 | 1 | `cuts.rs` `SINGLE_PT_MAXIMA` doc vs `detect_unimplemented` doc | the constant derives `sqrt(s_hat)/2`; the ceiling `(ebeam1+ebeam2)/2` (which takes sqrt(s) = ebeam1+ebeam2, as `integrate.rs:931` does) is stated only in the fn doc, so the derivation is split over two items | checked | fix: one helper carrying the whole derivation |
| H2 | 2 | `cuts.rs` tests `unbindable_pt_maxima_read_as_default`, `bindable_pt_maxima_are_refused` | both iterate `SINGLE_PT_MAXIMA` itself; deleting a name from the list (mutation: drop `"ptl4max"`) cannot fail them | suspected: run the mutation | fix: independent literal list, checked both ways against `UNIMPLEMENTED_CUTS` |
| H3 | 1/4 | `cli_unimplemented_cuts.rs:14-26` | re-implements `vibegraph()`, `stderr_of()` and card writing that `cli_hard_errors.rs:11-23` already holds, in a file for the same kind of test (refusals at the CLI boundary) | checked | fix: fold the tests into `cli_hard_errors.rs` |
| H4 | 2 | `integrate_refuses_an_unimplemented_cut_naming_it_and_its_value` | F1: stays green without the early call | checked | leave if its own mutation (drop the value from the message) fails it |
| H5 | 3 | `detect_unimplemented` `pub` | one cross-crate caller (`integrate.rs:530`), one in-module (`cuts.rs:554`); doc says it reads the run card alone | checked | leave |
| H6 | 4 | compile path vs early path | `grep -rn detect_unimplemented` shows one definition and two call sites | checked | leave (confirmed) |
| H7 | phys | the nine names vs MadGraph | see Method | checked | leave: all nine are single-object pT maxima or the neutrino-sum pT |
| H8 | found | HT-sum maxima | `cuts.f` defines them now (pinned reading below) | checked | Found |
| H9 | found | `sqrt_s_had = ebeam1 + ebeam2` | `s = 4 ebeam1 ebeam2` for head-on massless beams | checked | Found |

## Dispositions

| id | disposition | detail |
|---|---|---|
| H1 | fixed | the constant's doc now carries the whole derivation, including `s_hat <= s` and sqrt(s) = `ebeam1 + ebeam2`; the fn doc refers to it in a line |
| H2 | fixed | confirmed: with `"ptl4max"` dropped from `SINGLE_PT_MAXIMA` all 53 `cuts::` tests passed (`cutB-HY1-mutH2old.log`). Tests now use a literal `NINE_PT_MAXIMA`; new `no_other_unimplemented_cut_reads_an_unbindable_value_as_default` checks the other direction over `UNIMPLEMENTED_CUTS` (cards that do not parse at `1e5`, e.g. integer-typed `cutuse`, are skipped; the test requires more than 20 refused) |
| H3 | fixed | the four CLI tests moved into `vibegraph-cli/tests/cli_hard_errors.rs` (same helpers); `cli_unimplemented_cuts.rs` deleted. Test names gained the file's `cli_` prefix. Count unchanged (13 = 9 + 4) |
| H4 | leave | name says what it pins (message names cut and value); it fails with `describe` mutated to drop the value (below). It duplicates the second half of `cli_unimplemented_cut_is_refused_before_enumeration`, but is the only one on a process that enumerates fine |
| H5 | leave | `pub` is the narrowest the cross-crate call needs (`vibegraph::cuts` is already `pub`); the doc says it reads the run card alone |
| H6 | leave | confirmed one implementation: `grep -rn detect_unimplemented` gives the definition, `Cuts::compile_with` (`cuts.rs:554`) and `integrate.rs:530` |
| H7 | leave | pinned `cuts.f` at b7687064: `missetmax` bounds `pt(sum of neutrino momenta)` (`cuts.f:347-372`, `.gt. missetmax` guarded by `.ge.0`); `ptj<n>max` bounds the n-th hardest jet pT (`:853-855`, guarded by `.ge.0`); `ptl<n>max` the n-th hardest lepton pT (`:962`, `.ge.0`). All nine are a single object's pT or a subset's vector-sum pT; the derivation covers both. Defaults are -1 (`banner.py:4323,4384,4394`) |

## Mutations

Lib (`cutB-HY1-mut.log`, `cargo test -p vibegraph-lib --lib cuts::`; baseline 54 passed):

| mutation | result |
|---|---|
| drop `"ptl4max"` from `SINGLE_PT_MAXIMA` | old tests: 53 pass; new: `unbindable_pt_maxima_read_as_default` FAILED |
| add `"htjmax"` to the list | `no_other_unimplemented_cut_...` and `unbindable_value_does_not_excuse_other_cuts` FAILED |
| add `"ptj1min"` to the list | the same two FAILED |
| ceiling `0.5*` -> `1.0*` | `pt_maximum_threshold_is_half_the_collider_energy` FAILED |
| `>=` -> `>` | `pt_maximum_threshold_is_half_the_collider_energy` FAILED |

CLI (`cutB-HY1-mut2.log`, `cargo test -p vibegraph --test cli_hard_errors`; baseline 13 passed):

| mutation | result |
|---|---|
| early `detect_unimplemented` call removed from `load_run_card` | `cli_unimplemented_cut_is_refused_before_enumeration` and `cli_generate_refuses_...` FAILED; `cli_integrate_refuses_...` green (as F1 found) |
| `describe` prints `?` for a float | 4 FAILED, including `cli_integrate_refuses_...` |

No mutation reaches: the order of the early check against model loading, and the cost of a refused run.

## Gate

(`cutB-HY1-gate.log`)

- `cargo fmt --all --check`: rc=0.
- `cargo clippy --workspace --all-targets -- -D warnings`: `Finished dev profile`, rc=0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: `Finished dev profile`, rc=0.
- `cargo test --workspace`: every `test result` line `ok`, 0 failed; lib 1015 passed (1014 + `no_other_unimplemented_cut_reads_an_unbindable_value_as_default`), `cli_hard_errors` 13 passed (the four moved tests included). Net suite change: +1 test.
- `cargo doc -p vibegraph-lib --no-deps --document-private-items`: 8 warnings, none in `cuts.rs` (`configs.rs`, `channel.rs`, `alphas.rs`, `scales.rs`, `parameters.rs`, `lhef/mod.rs`); `-p vibegraph`: 1 warning, in neither touched file's doc.
- `python3 scripts/kb.py lint` (after `python3 scripts/kb.py index` regenerated the sprint session index): `825 concepts, 288 backlog items; 0 errors, 4 warnings` (the warnings are pre-existing `state: blocked` without `blocked_by`).
- Banked MG targets not run: no amplitude, colour, phase-space or sampling code changed.

## Method

Reading the diff against F1's report (30%), fetching `cuts.f`/`banner.py` at the pinned commit for the nine names (25%), mutating the new tests (30%), grep for duplicated CLI helpers (15%). The lead that held: a test that iterates the list it checks cannot see the list shrink. Leads that did not: visibility (`pub` is needed), duplicate implementation (none), history comments in the diff (grep empty).

## Found

- **HT-sum maxima can join the no-op rule.** `cuts.f:884-892,909`: `htjmax` is the sum of jet pTs, `ht2max`..`ht4max` its partial sums over the hardest 2..4 jets, `ihtmax` the same plus `ptheavyjet` (`:904`). Each is `<= sum |p_i| <= sum E_i <= sqrt(s_hat)`, so a value `>= sqrt(s)` cannot bind. Closing needs a second constant and ceiling and tests like `NINE_PT_MAXIMA`; not done here because the ceiling and its edge are a new physics reading.
- **`sqrt_s_had = rc.ebeam1 + rc.ebeam2`** (`vibegraph-cli/src/integrate.rs:931`, `generate.rs:1713`; `runcard.rs:188` "√ŝ = ebeam1 + ebeam2"): for head-on massless beams `s = 4 ebeam1 ebeam2`, so the sum is right only for equal beams. The pT ceiling here follows the same convention. An asymmetric-beam run card would need the fixed-energy path checked against MadGraph.
- **CLI test helpers are copied per file.** `fn vibegraph(cwd, home)` and `fn stderr_of` appear in `cli_hard_errors.rs`, `cli_first_run.rs` (with a dead-proxy variant) and, before this review, `cli_unimplemented_cuts.rs`; `["0 = lpp1 ... 45.6 = ebeam2"]` card text appears in `cli_first_run.rs:212`, `cli_hard_errors.rs` (x4), `cli_fixed_energy.rs`, `cli_generate.rs`. A `tests/common/mod.rs` would close it.
- Stale site: `research/kb/pipeline/release-scope.md:45` and `research/kb/history/notes/29-...:3989` describe `detect_unimplemented` without the early call or the no-op rule (the first is current docs, worth one clause).

## Brief corrections

- The F1 brief named the threshold from `ebeam1 + ebeam2` as "sqrt(s)"; for unequal beams that is not `sqrt(s)` (see Found). The ceiling stays `(ebeam1 + ebeam2)/2`, the generator's own convention, and the doc now says so.
