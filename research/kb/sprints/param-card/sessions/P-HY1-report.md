---
type: Session Report
title: "P-HY1 report: hygiene review of integrate --param-card"
description: "Hygiene review of PR #82 (integrate --param-card, the recorded parameter point, the <slha> header, the moved-parameter sigma gate): findings written before any edit, then fixes with mutation and identity evidence."
status: draft
tags: [hygiene, param-card, slha, review, report, sprint-e]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-30}
---
Worktree `/home/user/wt/P`, branch `sprinte-param-card` (PR #82), from
`6a7365d3`. Scope: `git diff origin/main...HEAD` (merge base `f5fb3e65`),
without the refdata pin commit and `research/kb/sprints/`. Cargo ran with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` and
`-j 2`.

## Findings

Written before the first edit. Line numbers are at `6a7365d3`.

| id | point | site | evidence | mutation the test should catch | confidence | disposition |
|---|---|---|---|---|---|---|
| H1 | 1 | `vibegraph-lib/src/ufo/slha.rs:291` | test doc: "The lines a card used to drop without a word are errors" (history framing) | — | checked | fix |
| H2 | 1 | `vibegraph-cli/tests/validate_param_card_sigma.rs:360-361` | "is kept beside the gate as the comparison that existed before the row was banked" (history framing) | — | checked | fix |
| H3 | 1 | `vibegraph-lib/src/lhef/write.rs:342` | "as MadGraph's restricted model no longer has it" (says what MadGraph's restriction removes as a change over time) | — | checked | fix |
| H4 | 2, 1 | `vibegraph-cli/tests/validate_param_card_sigma.rs:22-25, 66-67, 198-224` | `reference()` returns `None` ("AWAITING THE BANK") only when `manifest::planned_rows()` holds the row; `validation/manifest.toml:563-588` gives neither `pp_to_ll_mz95` nor `ee_to_tth_ymt250` a `status` line, and they are the file's only two rows, so the branch cannot run on any committed row. An absent record already fails, through an `assert!` naming the manifest rather than through `validation::require`. Dead test code of the kind `absent-run-escapes-spelled-per-gate` describes, added after that item's census | delete the record (point `record` at a missing directory): must fail naming the missing input | checked | fix: route an absent record through `validation::require`, delete the planned branch and the `manifest` module import |
| H5 | 3 | `vibegraph-lib/src/ufo/slha.rs:223, 233, 249` | `ParamCard::insert`, `entries`, `auto_widths` are `pub`; `grep -rn 'card\.insert\|\.entries()\|auto_widths()'` over `vibegraph-lib`, `vibegraph-cli` finds callers only in `ufo/bind.rs` and `slha.rs`'s own tests | — | checked | fix: `pub(crate)` |
| H6 | 3 | `vibegraph-lib/src/lhef/write.rs:350` | `pub fn slha_block`; its only callers are `vibegraph-lib/src/generate.rs:549, 1001, 1005` | — | checked | fix: `pub(crate)` |
| H7 | 1 | `vibegraph-lib/src/lhef/write.rs:350-539` | `slha_block` is 190 lines, five closures and three sections in one body, no stated reason | — | checked | fix: split into section helpers; identity oracle chosen below |
| H8 | 4 | `vibegraph-lib/src/ufo/bind.rs:140` and `vibegraph-lib/src/ufo/slha.rs:185-191` | the same `code.iter().map(i32::to_string).collect::<Vec<_>>().join(" ")` written twice in the change, once as `code_text`, once inline in `ParamCard::record` (also in `reweight/mod.rs:247-250`, outside the change) | — | checked | fix inside the change (one `slha` helper); the `reweight` copy to Found |
| H9 | 1, 4 | `vibegraph-lib/src/ufo/bind.rs:189-214` against `vibegraph-lib/src/ufo/parameters.rs:100-120` | `bound_externals` restates `ParameterSet::evaluate`'s binding rule (card value, else default; a zero-locked parameter stays at zero) to compute what `evaluate` bound. Its doc says it is "the value `from_model_card` evaluates it at", a claim held only by the two copies agreeing | a change to the rule in one copy only: `moved` would record a value the evaluation did not use | checked | fix: one `ParameterSet` method both call; identity oracle chosen below |
| H10 | 4 | `vibegraph-lib/src/reweight/engine.rs:1024-1057` | the test helper `fresh` writes a card as text, merging lines per block by hand because the stricter reader refuses a reopened block, then parses it; `ParamCard::insert`, added by this change, sets the values directly | — | checked | fix: build the card with `insert` |
| H11 | 2 | `vibegraph-lib/src/ufo/bind.rs:41` (`DEPENDENT_TOLERANCE = 1e-6`) | the tests that read `ignored` put a dependent entry at 3e-8 (MadGraph's own card's MW, against 80.41900245), at 0.5 % (MW = 80.0) or at 5 % (MW = 80.419 at MZ = 95) off the computed value. Arithmetic: any tolerance in [3e-8, 5e-3) passes all of them, so a 1000× looser tolerance is invisible | `DEPENDENT_TOLERANCE = 1e-3` (and `1e-7` on the other side) | checked by arithmetic; to be run | fix: pin both edges, 4e-7 not reported (inside `%e`'s 5e-7) and 2e-6 reported |
| H12 | 1 | `docs/src/cli/overview.md:87-91` | the refusal list names `Auto`, a revived zero, a split merge and an unknown entry; the `--param-card` help (`vibegraph-cli/src/integrate.rs:343-350`) and `bind_card` also refuse a parameter the restriction fixes to one (`CardError::MovesFixedOne`) | — | checked | fix |
| H13 | 4 | `vibegraph-lib/src/ufo/bind.rs:149-160`, `vibegraph-lib/src/reweight/mod.rs:229-251` | two lookups of an external by `(block, code)`, case-insensitive block: `external_at` (new) and `external_name` (reweight card) | — | checked | Found (the reweight module is outside the change) |
| H14 | 4 | `vibegraph-cli/tests/validate_param_card_sigma.rs:93-95` | `pdf_dir()` is `Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf")`; the same body is in `cli_generate_proton.rs:128`, `cli_onshell_veto.rs:141`, `cli_integrate.rs:37`, `cli_decay_chain_events.rs:133`, `cli_pdf_cache_fetched.rs:70`, `cli_decay_chain.rs:84`, `validate_samples_proton.rs:101` (and `cli_reweight_proton.rs:62` through `repo()`) | — | checked | Found |
| H15 | 1 | `vibegraph-lib/src/ufo/bind.rs:47-50`, `cli_param_card.rs:317` | the `AutoWidth` message says "computing Auto widths is not supported yet" | — | checked | leave: a user-facing statement of a limitation, describing the missing work itself, and pinned by two tests |
| H16 | 2 | `vibegraph-lib/src/ufo/bind.rs:478-493` | `moves_are_recorded_in_the_models_external_order` could be vacuous if the card's sorted entry order (`mass` < `yukawa`: MZ, ymt) equalled the model's | — | checked | leave: the SM's `parameters.py` defines `ymt` (line 93) before `MZ` (line 125), so the model order differs from the entry order and a "record in card-entry order" mutation fails the test |
| H17 | 2 | `vibegraph-lib/src/ufo/bind.rs:743-761` | `a_recorded_parameter_the_model_lacks_is_refused`: is each clause of `bind_recorded`'s `free` predicate reached? | — | checked | leave: `MC` reaches the zero clause, `XX` at `MASS 23` the name clause, `MZ` at `MASS 24` the no-external clause; the ones clause is reached by the toy-model test |

Every refusal test in the change was read for an any-refusal assert. All
match the variant (`ufo::bind` tests, `CardError::*`) or the path's own
message (`slha.rs` parse errors by line and text, `cli_param_card.rs` by the
message's distinctive words plus no artifact written). None was found
vacuous. The `<slha>` layout test compares every data line and heading with
MadGraph's card; what it cannot see (the preamble wording, a BSM model's
MadGraph card) is stated in P-F1's report.

**Backlog check.** Items that name a reviewed path (`grep -rlE` over
`research/kb/backlog` for the path tails `/bind.rs`, `/slha.rs`,
`lhef/write.rs`, `/parameters.rs`, `reweight/engine.rs`, the two CLI command
files and the two new test files): 16. Those with a site in the change:

- `feature/restrict-card-unit-values-not-fixed`: the code now records a
  restriction's ones (`ParameterSet::ones`) and `bind_card` refuses a move of
  one. Its claim that no repository restrict card sets a parameter to 1 is
  wrong for `vibegraph_toy_UFO/restrict_all.dat` (`WO8`), and its line
  citations (`parameters.rs:115`, `:125`) have moved. Found.
- `hygiene/ufo-required-fields-default-silently`: its `DECAY Auto` half
  ("silently skipped") is closed by `ParamCard::auto_widths` and
  `CardError::AutoWidth`. Found.
- `feature/input-cards-lack-field-classification`: the param-card part now
  refuses unknown blocks and codes and names its two exemptions; there is still
  no classification table. Not a finding; recorded in Found as a site update.
- `hygiene/absent-run-escapes-spelled-per-gate`: H4 removes the copy this
  change added; the item's census predates it, so no site there goes stale.
- `hygiene/rustdoc-warnings-ungated` names `ufo/parameters.rs`; checked under
  the doc gate below.

The other eleven name the paths for code the change does not touch.

**Identity oracles, chosen before the first edit.**

- H7, H9: a temporary test (deleted before commit) writes, for each of the ten
  interned SM restrictions at its defaults and at a card moving `MZ`, `ymt` and
  `WT`, and for the toy model under `restrict_all.dat`, the full `slha_block`
  text and the `to_bits` of every parameter `EvaluatedModel::from_model_card`
  binds plus `bind_card`'s `moved` list (`Debug`), to one file. The file from
  the base tree and the one from the fixed tree must be byte-identical (`cmp`).
- H10: the engine's unit tests, plus a temporary assert inside `fresh` that the
  inserted card's `entries()` equal those of the text card it replaces, run
  once and removed.
