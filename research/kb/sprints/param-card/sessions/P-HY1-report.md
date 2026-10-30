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

## Fixed

All in one code commit (listed under Commits), each fix tied to its
finding:

- **H1, H2, H3.** The three history-framed comments now describe the code as
  it is: the parse-error test's doc, the reported ratio's doc ("so the two
  meet on one point from two directions"), and `slha_block`'s ("as MadGraph's
  restriction removes it from the model").
- **H4.** `validate_param_card_sigma`'s `reference()` returns the seed record
  or fails through `validation::require` naming the missing file. The planned
  branch, the `AWAITING THE BANK` print, the `Option` at both callers and the
  `manifest` module import are gone, and the module doc says both rows are
  banked.
- **H5, H6.** `ParamCard::insert`, `entries` and `auto_widths`, and
  `lhef::write::slha_block`, are `pub(crate)`.
- **H7.** `slha_block` is now 43 lines. Its sections are named functions:
  `slha_value`, `slha_free_code`, `slha_externals`, `slha_particles`,
  `slha_derived`, `slha_dependent_lines`, `slha_heading`,
  `slha_external_line` and `slha_qnumbers`, the longest 36 lines. The bodies
  moved unchanged.
- **H8.** `slha::code_text` is the one LHA-code formatter. `bind.rs` imports
  it, and `ParamCard::record`'s error message calls it.
- **H9.** `ParameterSet::bound_value(p, card)` is the binding rule: the card's
  value, else the restricted default, and zero for a zero-locked parameter.
  `ParameterSet::evaluate` and `bind::bound_externals` both call it, so the
  value `moved` records is the value the evaluation used, by construction.
- **H10.** The reweight engine's test helper `fresh` builds its card with
  `ParamCard::insert`. The hand-written block merging is gone (19 lines to 3).
- **H11.** New test `ufo::bind::tests::a_dependent_entry_is_reported_beyond_the_cards_rounding`
  puts MW at ±4e-7 (not reported: inside `%e`'s 5e-7, as the constant's doc
  states) and at ±2e-6 (reported). The bounds come from the constant's
  recorded rationale, not from a fresh number, and the test is deterministic,
  so its power is exact.
- **H12.** `docs/src/cli/overview.md` lists the fixed-to-one refusal.

**Identity evidence (H7, H9).** I ran the temporary probe described above on
the base tree (`6a7365d3` plus the findings commit) and on the fixed tree,
then removed it before the commit. It covered 22 cases: ten restrictions at
the defaults and at the moved card, plus the toy model at its defaults and
moved. The two outputs are 3122 lines and 113127 bytes each, and
`cmp identity-before.txt identity-after.txt` reports no difference. The
probe covers both `bind_card` outcomes: 20 binds succeed, and one is
refused (`RevivesZeroed` on `WT`, under the restriction that zeroes it).

**Identity evidence (H10).** A temporary assert in `fresh` compared, bit for
bit, the `entries()` of the inserted card with those of the text card it
replaces. It held on all 516 calls across the 20 engine tests (88 cards of
211 entries, 426 of 26, and two whose count interleaved with the test
output), and every engine test passed. The assert was
removed before the commit.

## Mutations

Driver: `P-HY1/mutate.sh` in the scratchpad. It applies each mutation, runs
`cargo test -p vibegraph-lib --lib -- ufo::bind`, restores the file, and
`cmp`s it against a pristine copy.

| mutation | old tests (the 17 bind tests without H11's) | new test |
|---|---|---|
| `DEPENDENT_TOLERANCE` `1e-6` → `1e-3` | 17 passed | FAILED: `MW off by 2e-6`, left 0, right 1 |
| `DEPENDENT_TOLERANCE` `1e-6` → `1e-7` | 17 passed | FAILED: `MW off by 4e-7`, left 1, right 0 |
| unmutated | 18 passed | |

The first run of this batch reported the unmutated tree as failing. The
driver restored each file with `mv` from a copy taken before the mutation,
so the restored file's mtime was older than the mutated build, cargo did not
rebuild, and the "unmutated" run used the `1e-7` binary. With a `touch`
after each restore, the rerun gives the table above. This is a driver lesson
for `hygiene-review.md` (Method).

**H4** (`P-HY1/banked.sh`): the Drell-Yan row's `record` points at
`dy13_mmll_60_120_mz95_absent`. The result is under Gate. On the old code
this mutation also failed, through the `assert!` naming the manifest
(reasoned from the code, not run: the row has no `planned` status). H4 is a
dead-code deletion, not a vacuity fix: the branch it deletes could pass only
on a row marked planned.

**Unreachable by mutation.**

- H7, H9 and H10 are refactors; the identity oracles above are their
  evidence.
- H1–H3, H5, H6, H8 and H12 change no behaviour.

## Gate

Commands as run (`-j 2`, with the environment above), each with the tail of
its output. The tree is `024ea138`; the report commit touches only the
sprint folder.

```
$ cargo fmt --all --check
exit=0

$ cargo clippy -j 2 --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 51.71s
exit=0

$ cargo clippy -j 2 --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
    Finished `dev` profile [optimized] target(s) in 11.41s
exit=0

$ cargo doc -j 2 -p vibegraph-lib --no-deps --document-private-items
warning: `vibegraph-lib` (lib doc) generated 8 warnings
exit=0
$ cargo doc -j 2 -p vibegraph --no-deps --document-private-items
   Generated /home/user/wt/P/target/doc/vibegraph/index.html
exit=0

$ cargo test -j 2 --workspace --no-fail-fast
49 binaries, 1736 passed, 0 failed, 19 ignored; exit=0

$ python3 scripts/kb.py lint
1006 concepts, 332 backlog items; 0 errors, 4 warnings
```

The 8 rustdoc warnings are the 8 sites `rustdoc-warnings-ungated` lists
(`configs.rs:4`, `channel.rs:774`, `alphas.rs:60`, `scales.rs` ×2,
`parameters.rs:365` (the pre-existing `Vec<i32>` in `extract_lhacode`'s
doc), `lhef/mod.rs` ×2), so none is new. The suite count is P-V2's 1735 plus
the one test H11 adds. Nothing was deleted. kb lint first failed on a stale
`sessions/index.md`, because this report was new. `python3 scripts/kb.py
index` regenerated it, and the four warnings are `state: blocked` backlog
items this session did not touch.

Banked targets (`--config 'profile.release-debug.debug=0' --profile
release-debug --features extended-validation -- --nocapture
--test-threads 1`), run one at a time by `P-HY1/banked.sh`. All four are
reached by H9, the binding rule every evaluation goes through, and
`validate_param_card_sigma` also by H4:

```
validate_param_card_sigma exit=0 279s | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 71.69s
reweight_mg_oracle exit=0 152s | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
cli_reweight_proton exit=0 241s | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 220.49s
validate_lhef exit=0 175s | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 93.98s
[pp_to_ll_mz95] GATE vibegraph σ = 7.4688953e2 ± 4.61e-1 pb (5 seeds, χ²/dof = 2.00) | MG σ = 7.4686000e2 ± 1.58e-1 pb | pull = +0.06
[ee_to_tth_ymt250] GATE vibegraph σ = 4.0961865e-3 ± 1.01e-6 pb (5 seeds, χ²/dof = 1.02) | MG σ = 4.0958200e-3 ± 2.35e-6 pb | pull = +0.14
reweight_mg_oracle: every row 0 beyond 2e-7; worst |w/w_MG - 1| 4.46e-8 (ee_tth_ymt ymt0)
```

Both σ rows equal P-V2's, digit for digit, as a bit-identical binding must
give. The H4 mutation, with the Drell-Yan record pointed at
`dy13_mmll_60_120_mz95_absent`:

```
h4-mutation exit=101 | panicked at vibegraph-lib/src/validation.rs:32:5: `validate_param_card_sigma`
needs the row's seed record (output/seeded_sigma/dy13_mmll_60_120_mz95_absent/seed_record.json). ...
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out
h4 restored byte-identical
```

**Not run**: the whole banked layer (`validate.sh`). The change touches no
amplitude, colour, phase-space or sampling code. The four targets above are
the brief's list for code they cover.

## Commits

- `a1a53a80`: the findings, before any edit.
- `162f9132`: H1, H3, H5–H11 (library code and tests).
- `024ea138`: H2, H4, H12 (the σ gate and the CLI overview).
- this report.

## Routed

None. No finding moves a computed number, a gate's threshold or a convention.
H9 changes how the binding rule is reached, not the rule, and the identity
oracle shows bit-equal values.

## Method

- **Reading the diff with the four points in hand (about 35%).** This found
  H7, H9 and H10: the restated binding rule, and a test helper working around
  the stricter reader when the change's own `insert` would do.
- **Grepping added lines for history and plan words (5%).** Found H1–H3. A
  grep for session ids (`P-F1`, `sprint`, `brief`) over the code and docs
  found nothing.
- **Grepping each new `pub` item's callers across the workspace (10%).**
  Found H5 and H6.
- **Arithmetic on each tolerance against the values its tests use (10%).**
  Found H11. The tests' entries sit at 3e-8, 0.5 % and 5 %, which leaves
  three decades of tolerance unpinned.
- **Checking each refusal test for which refusal it asserts (10%).** None was
  vacuous (H16, H17 and the summary under Findings). This was the brief's
  main suspicion, and it did not hold.
- **The manifest against the gate's planned branch (5%).** Found H4.
- **The backlog check by path tail (10%).** Found three items with sites the
  change closes or makes stale.
- **Identity oracles and mutation runs (15%).**

Leads that did not hold up:

- The external-order test, which looked able to pass vacuously. It does not,
  because the SM defines `ymt` before `MZ`.
- `bind_recorded`'s refusal cases, which each reach their own clause.

**A driver lesson for `hygiene-review.md`.** A mutation driver that restores
a file by `mv` from a copy taken before the mutation gives it an older mtime
than the mutated build. Cargo then skips the rebuild, and the next run tests
the mutant. Touch the file after each restore, or restore with `cp` over it.
My first batch reported the unmutated tree as failing for this reason.

The first gate driver was detached (`setsid`) and so sent no completion
notice. I stopped it, killing my own process group only, after the default
clippy had passed, and reran the remaining gates as a notifying background
job.

## Found

- **The reweight card resolves an external by `(block, code)` with its own
  copy of the lookup** (H13). `bind::external_at`
  (`vibegraph-lib/src/ufo/bind.rs`) and `reweight::external_name`
  (`vibegraph-lib/src/reweight/mod.rs:229-251`) each scan
  `params.externals` for a case-insensitive block and an equal code, and
  `reweight/mod.rs:247-250` formats the code by hand. Closing it: a
  `ParameterSet::external_at(block, code)` and `slha::code_text` used by
  both.
- **`pdf_dir()` is written in nine CLI test files** (H14):
  `validate_param_card_sigma.rs:93`, `cli_generate_proton.rs:128`,
  `cli_onshell_veto.rs:141`, `cli_integrate.rs:37`,
  `cli_decay_chain_events.rs:133`, `cli_pdf_cache_fetched.rs:70`,
  `cli_decay_chain.rs:84` and `validate_samples_proton.rs:101`, plus
  `cli_reweight_proton.rs:62` through `repo()`. Closing it: one
  `common::pdf_dir()` in `vibegraph-cli/tests/common/mod.rs`.
- **`feature/restrict-card-unit-values-not-fixed` is largely done by this
  PR.** `ParameterSet::ones` records a restriction's ones, and `bind_card`
  and `bind_recorded` refuse moving one. What its `closes_when` still asks
  is the comparison against MadGraph's generated param card. Its text is also
  stale: "no restrict card in the repository set a non-`QNUMBERS` parameter
  to 1" is false for `validation/ufo/vibegraph_toy_UFO/restrict_all.dat`
  (`WO8`), and `parameters.rs:115`, `:125` have moved. The manager should
  close or narrow it.
- **`hygiene/ufo-required-fields-default-silently`: its `DECAY Auto` half is
  closed** (`ParamCard::auto_widths`, `CardError::AutoWidth`). The item's
  `ufo/slha.rs` site and its title's second clause are stale.
- **`feature/input-cards-lack-field-classification`: the param-card bullet
  is out of date.** Unknown blocks and codes are now refused, `QNUMBERS` and
  dependent `MASS`/`DECAY` entries are named exemptions, and a reopened block
  or a conflicting repeat is a parse error. The classification table the item
  asks for is still absent.
- **The `<slha>` block still writes a restriction's ones and merged
  members, which MadGraph's card leaves out.** P-F1 noted this. Now that
  `ParameterSet::ones` and `merged` exist, `slha_externals` could skip them.
  It is layout parity with MadGraph's writer (no repository model has either
  case), and it needs a test on the toy model's `WO8`.
- **The planned-row escape pattern** (`hygiene/absent-run-escapes-spelled-per-gate`):
  this PR added one more copy, in `validate_param_card_sigma.rs`, and H4
  deleted it. Nothing in that item goes stale.

## Brief corrections

- The brief lists `reweight_mg_oracle`, `cli_reweight_proton`,
  `validate_lhef` and `validate_param_card_sigma` as the banked targets to
  rerun when touched code is covered. H9 reaches every model evaluation, so
  I ran all four (Gate).
- "The σ gate's awaiting-the-bank path should now be unreachable for these
  two rows": confirmed, and the path is deleted (H4) rather than left
  unreachable.
