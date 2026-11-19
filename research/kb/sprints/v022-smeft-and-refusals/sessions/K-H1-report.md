---
type: Session Report
title: "K-H1 report: hygiene pass on the model loading"
description: "Hygiene review of sprint K's model-loading PRs (#141 negative coupling orders, #144 restriction cards, #145 UFO value forms): findings written before any edit, one mutation per added test, the fixes with their mutation evidence, and σ, grid and enumerate output bit for bit against main."
status: draft
tags: [hygiene, ufo, restriction, coupling-orders, non-vacuity, visibility, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Worktree `/home/user/wt/sprintk-hyg-model`, branch `sprintk-hyg-model`, based
on `main` at `b3bc5399`. The review covers the squash commits `54ded749`
(#141), `09490121` (#144) and `f65d1bb7` (#145): their code, tests and docs,
and the direct callers they touched. The [brief](K-H1.md) is binding.

## Findings

Written before any code edit (commit `420bb8fa`); the disposition column was filled in after the fixes. Point: 1 maintainability, 2 non-vacuity, 3
visibility, 4 abstractions. Confidence: *checked* (the quoted evidence settles
it) or *suspected* (the stated run settles it). Ranked by consequence;
disposition *fix* marks what this session fixes.

| id | rank | point | site | what is wrong (class) | evidence | proposed fix | confidence | disposition |
|---|---|---|---|---|---|---|---|---|
| H1 | 1 | 1, 2 | `ufo/mod.rs:335`, `:353` (and `:399`, `:417`) | A read error on an optional source file is swallowed (panic-or-unwrap class: a fallible path read as absence). `Err(_) => FunctionLibrary::default()` treats an unreadable or non-UTF-8 `function_library.py` as no library, so a SMEFTsim-style model silently evaluates `sec`/`csc`/`asec`/`acsc` on the complex argument instead of its own real-part definitions: a silent wrong answer. `configuration.py` likewise reads as empty. The two older sites (`coupling_orders.py`, `propagators.py`) have the same shape and sit in the same function. | `match std::fs::read_to_string(path.join(OPTIONAL_SOURCE_FILES[3])) { Ok(src) => FunctionLibrary::parse(&src)?, Err(_) => FunctionLibrary::default() }`; the required files map every error to `UfoError::Io` (`:316`). | One `read_optional` helper: `NotFound` is absence, any other error is `UfoError::Io` naming the file. A test with a non-UTF-8 `function_library.py` fails before and passes after. Name the four files instead of indexing `OPTIONAL_SOURCE_FILES[0..3]`. | checked (code); the test run settles the behaviour | fixed `eb9fb2ef` |
| H2 | 2 | 2 | `tests/ufo_value_forms.rs:362` `a_particle_charge_that_is_a_parameters_value_loads` | Vacuous test: it asserts only that the model loads, never the charge. | Mutation: `default_of_variable` returns `Ok(0.0)` for every external; the test reads nothing that changes. | Assert `S`'s charge is `MS`'s default, 10. | suspected; the mutation run settles it | fixed `eb9fb2ef` (see below: an in-crate test) |
| H3 | 3 | 2 | `diagrams/mod.rs:117`, `:1018` | `DiagramError::GluonTrialWithoutQcd` has no test: `grep -rn GluonTrialWithoutQcd` returns only its definition and its construction. | grep output above. | A test on a scratch model whose gluon (PDG 21) sits in the lowest class of a hierarchy without QCD, asserting the variant; it fails under `unwrap_or(&0)` in place of the refusal. | checked | fixed `784dd93c` |
| H4 | 4 | 3, 1 | `ufo/couplings.rs:109-117`, `ufo/particles.rs:166-191` | A production contract documented on a `cfg(test)` item, and production docs linking it. #145 made `parse_couplings` and `parse_particles` `#[cfg(test)]` wrappers but left the contract (`.anti()` handling, anti-particle entries, skipped attribute assignments) on `parse_particles`, while the production `parse_particles_with` and `parse_couplings_with` say only "[`parse_particles`], reading …", a link that does not resolve outside `cfg(test)`. | the quoted docs; `cargo doc -p vibegraph-lib --no-deps --document-private-items` (run after the first build) | Move the contract onto the production functions; the test wrappers point at them. | checked (code); the doc build settles the warning | fixed `4ccb0dc5` |
| H5 | 5 | 2 | `ufo/expr.rs:670` `the_unselected_branch_is_not_evaluated` | The test claims lazy evaluation of `Cond`, which its assert cannot see: `eval` is total, so an eager evaluation of `1/x` at `x = 0` yields a NaN that is discarded, and the result is still 0. What it does check is `conditional_expressions_select_a_branch`'s first case. | Mutation: evaluate both branches, then select. | Delete it (a duplicate that cannot fail on its claim); say in the report that the laziness is unobservable through `eval`. | suspected; the mutation run settles it | fixed `4ccb0dc5` |
| H6 | 6 | 2 | `ufo/expr.rs:691` `max_min_and_the_dm_width_form` | Its doc claims `max`/`min` return the first of equal arguments; no assert has a tie whose members differ, so the tie rule is unpinned. | Mutation: `v.re > best.re` → `v.re >= best.re` in `eval`'s `Max`/`Min`. | Add a tie whose arguments differ in their imaginary part. | suspected; the mutation run settles it | fixed `4ccb0dc5` |
| H7 | 7 | 4, 1 | `diagrams/chain.rs:305`, `diagrams/reduction.rs:210` | A second and third copy of `ufo::weighted::order_weight` (each order's power times its hierarchy, a missing order weighing 0). Filed as `weighted-order-computed-three-ways`; #141 added the `reduction.rs` instance and retyped the `chain.rs` one. | `n * model.order_hierarchy.get(name).copied().unwrap_or(0) as i64` (chain), `weighted += n * i64::from(model.order_hierarchy.get(name).copied().unwrap_or(0))` (reduction). | Both call `order_weight`. Integer arithmetic: identity by construction, checked by the enumerate oracle. The view-based sites stay (Found). | checked | fixed `784dd93c` |
| H8 | 8 | 4 | `config.rs:443` `free_parameters`, `ufo/mod.rs:2250` | The same "free externals" filter (not zeroed, not fixed at one, not a merged follower) written twice by #144's two tests. | Both bodies `flat_map(\|group\| group.iter().skip(1).map(\|(name, _)\| name))` then filter `zeros`/`ones`. | One `#[cfg(test)]` `ParameterSet::free_externals` in `parameters.rs`, used by both. | checked | fixed `eb9fb2ef` |
| H9 | 9 | 3 | `ufo/loaded.rs:24`, `:114`, `:135` | `pub` with no user outside the crate: `FULL_RESTRICTION`, `LoadedModel::from_dir_full`, `LoadedModel::interned_sm_full`. | `grep -rn "FULL_RESTRICTION\|from_dir_full\|interned_sm_full"` hits only `loaded.rs` and `config.rs`. | `pub(crate)`. | checked | fixed `4ccb0dc5` |
| H10 | 10 | 1 | `helas/eval/fold.rs:729-738` | A comment separated from its arm: #145 inserted the `Cond`/`Cmp`/`Logic` arm between the `complex(re, im)` comment and `Expr::Complex`, so the comment now sits above the wrong arm. | quoted lines 729-738. | Move the arm above the `complex` comment. | checked | fixed `4ccb0dc5` |
| H11 | 11 | 1 | `diagrams/selector.rs:173` | Comment contradicts the change: "Negative coupling powers are physically nonsensical" in the test #141 touched, while #141 reads SMEFTsim's `QED: -1` powers. | quoted line. | Say what the test pins: a negative bound builds without a panic. | checked | fixed `784dd93c` |
| H12 | 12 | 1 | `diagrams/mod.rs:733` `generate_undecayed` | 186 lines; #141 grew it by the gluon-free trial loop. | `sed -n 733,918p`. | Move the automatic WEIGHTED search into its own function. Identity oracle: `enumerate` output byte-identical. | checked | fixed `784dd93c` |
| H13 | 13 | 2 | `ufo/functions.rs:208` | Any-refusal assert: the two-argument definition is checked with `is_err()`. | quoted line. | Match `FunctionLibraryError::UnknownDefinition` naming `sec`. Mutation: drop the `arguments == ["z"]` filter. | checked | fixed `4ccb0dc5` |
| H14 | 14 | 2 | `diagrams/selector.rs:136-177` | The selector tests assert nothing (`let _ = build_selector(…)`): they fail only on a panic. They predate #141, which only changed their signature. | quoted lines. | Out of this change's reach: Found. | checked | Found |
| H15 | 15 | 1 | `ufo/expr.rs:299`, `:313` (`Logic` arms) | An `expect` and `args[0]` on the evaluator: a panic on a malformed `Expr`. | quoted line. | Leave: the arity is the parser's invariant (`call` refuses `max()`/`max(a)`, the grammar builds `Not` with one operand and `And`/`Or` with two), and `Expr` is crate-private. | checked | leave |
| H16 | 16 | 1 | `diagrams/reduction.rs:194` | `within_running_budget` keeps a diagram it cannot root. | quoted line. | Leave: every tree has a centre vertex or edge and a decay always has its initial leg, so `None` arises only for a graph with no vertex, which spends nothing. | checked | leave |

Every test the three PRs add or change gets one mutation below
(Non-vacuity), whether or not it is a finding.

Backlog items naming a reviewed path, by path tail: `weighted-order-computed-three-ways`
(H7: two of its new instances fixed here, the others Found),
`diagram-count-test-helper-duplicated` (`negative_orders.rs` adds a fifth
`count`/`try_generate` pair: Found), `lib-tests-hand-roll-temp-dirs` (the new
tests use `tempfile`; nothing to add), `rustdoc-warnings-ungated` (H4's links).
The others name these files for unrelated sites.

## What changed

| commit | findings | what |
|---|---|---|
| `420bb8fa` | — | this report's findings, before any code edit |
| `eb9fb2ef` | H1, H2, H8 | `ParsedModel::parse` reads each optional file through one `read_optional`: `NotFound` is absence, any other error is `UfoError::Io` naming the file; the four files are bound by name. New test `an_unreadable_optional_source_file_is_refused_naming_it` (all four files). New lib test `a_charge_naming_a_parameter_reads_its_default_through_parse`. `#[cfg(test)] ParameterSet::free_externals` serves both restriction tests. |
| `784dd93c` | H3, H7, H11, H12 | `search_weighted` holds the automatic WEIGHTED search; `generate_undecayed` is 141 lines. `chain.rs` and `reduction.rs` call `order_weight`. New test `a_gluon_trial_on_a_model_without_a_qcd_hierarchy_is_refused_by_name`. Selector test comment. |
| `4ccb0dc5` | H4, H5, H6, H9, H10, H13 | Contracts moved onto `parse_particles_with`/`parse_couplings_with`; `FULL_RESTRICTION`, `from_dir_full`, `interned_sm_full` are `pub(crate)`; the `fold.rs` arm moved above the `complex` comment; `max`/`min` tie asserts; `UnknownDefinition` matched; `the_unselected_branch_is_not_evaluated` deleted. |

**H2, as fixed.** The proposed assert on `model.particles["S"].charge` in
`ufo_value_forms.rs` does not compile: `Particle::charge` is `pub(crate)`, and
nothing in production reads it (`grep -rn '\.charge\b' vibegraph-lib/src`
outside `particles.rs` hits only `leg.charge`, a different field). The value
is pinned instead by a lib test in `ufo/mod.rs` that rewrites the toy's
`lt` charge to `Param.MLT.value` and reads 10 and −10 through
`ParsedModel::parse`. The integration test stays as the load check it is
(Found 2).

**H5.** `Expr::Cond`'s doc still says only the selected branch is evaluated,
which is true; no test can see it, because `eval` is total and a discarded
NaN changes nothing.

## Non-vacuity

One mutation per test the three PRs add or change, run on a copy of the tree
at `420bb8fa` (`kh1-mutate.py`: apply, run, restore, log), then the post-fix
set on a copy at `4ccb0dc5`. CAUGHT = the test failed. Each command was
`cargo test -q -p vibegraph-lib {--lib|--test <target>} -- <test>`. The
unmutated baseline passed every target on both copies (the copies lacked
`validation/madgraph/dy13_param_card.dat`, so two `ufo::bind` tests outside
this review failed there for that reason only).

| test | mutation | caught |
|---|---|---|
| `negative_orders::a_negative_coupling_order_loads_signed` | `order_value(..).map(\|n\| n.max(0))` | yes |
| `negative_orders::a_bound_is_spent_vertex_by_vertex_in_madgraph_order` | `within_running_budget` keeps every diagram | yes |
| same | `apply_signed_constraint` drops `negative.power` | yes |
| same | WEIGHTED filter drops `negative.weighted` | yes |
| `weighted_search::a_final_state_gluon_pair_without_a_qcd_vertex_is_refused` | `gluons_leave_trials` → false | yes |
| `weighted_search::a_gluon_trial_on_a_model_without_a_qcd_hierarchy_is_refused_by_name` (new) | `.get("QCD").or(Some(&1))` | yes |
| `config::sm_full_applies_no_restriction` | `interned_sm_full` starts from the default-restricted model | yes |
| `config::full_on_a_ufo_directory_skips_its_default_restriction` | never try the whole name as a directory | yes |
| `parse::model_import_splits_at_the_last_dash` | `rfind` → `find` | yes |
| `ufo::the_special_values_card_restricts_as_madgraph_does`, `card_classes::every_restrict_card_construct_behaves_as_classified` | a free spelling of one is fixed at one | yes, both |
| `parameters::madgraphs_free_spellings_become_free_defaults` | default kept as written | yes |
| `input_card_refusals::madgraphs_free_one_and_zero_load_as_free_defaults`, `every_restrict_card_…` | a free zero is locked to zero | yes, both |
| `expr::conditional_expressions_select_a_branch`, `ufo_value_forms::a_conditional_value_selects_by_the_default_of_its_condition` | `Cond` branches swapped | yes, both |
| `expr::the_unselected_branch_is_not_evaluated` | evaluate both branches, then select | **no** (H5: deleted) |
| `expr::boolean_operators_follow_python` | `and` returns a bool | yes |
| `expr::max_min_and_the_dm_width_form` (old) | tie rule `>` → `>=` | **no** (H6) |
| `expr::max_min_and_the_dm_width_form` (new) | same | yes |
| `expr::max_min_…`, `ufo_value_forms::a_width_written_with_max_re_and_cmath_sqrt_evaluates` | `max` ↔ `min` | yes, both |
| `expr::the_hyperbolic_and_log10_cmath_functions_evaluate` | `sinh` → `cosh` | yes |
| `expr::malformed_conditionals_and_calls_are_refused` | `max` takes one argument | yes |
| `expr::names_that_start_with_a_keyword_are_parameters` | `not` without `!ident_continue()` | yes |
| `functions::each_library_selects_its_own_evaluation_on_a_complex_argument` | `resolve` returns its input | yes |
| `functions::without_a_library_the_functions_take_the_complex_argument` | an empty library selects the real-part variants | yes |
| `functions::a_definition_in_another_form_is_refused_naming_the_function` | drop the `arguments == ["z"]` filter | yes, old and new |
| `functions::functions_this_loader_does_not_evaluate_are_not_read` | read every `Function` | yes |
| `functions::smeftsims_library_takes_the_real_part_…`, `each_library_…` | SMEFTsim's `sec` spelling marked complex | yes, both |
| `functions::no_smeftsim_value_calls_the_functions_the_libraries_disagree_on` | `map_calls` never calls `f` | yes (the `!seen.is_empty()` guard) |
| `particles::a_charge_that_is_a_parameters_value_is_its_default` | an aliased import maps to the alias | yes |
| `particles::a_charge_the_resolver_refuses_is_refused_naming_the_particle` | the message drops the particle | yes |
| `vertices::a_list_of_couplings_keeps_its_members_in_order` | list members sorted | yes |
| `vertices::an_empty_or_mixed_coupling_list_is_refused` | `[]` accepted | yes |
| `ufo_value_forms::orders_come_from_the_couplings_when_coupling_orders_py_is_absent` | the old fixed SM default | yes |
| `ufo_value_forms::a_model_with_exactly_qcd_and_qed_weights_qed_at_two_without_the_file` | no QED=2 special case | yes |
| `ufo_value_forms::a_value_split_over_lines_with_plus_is_one_string` | `+` keeps the left literal | yes |
| `ufo_value_forms::a_value_that_is_a_bare_name_reads_that_name` | the number lookup never hits | yes |
| `ufo_value_forms::a_bare_name_nothing_defines_is_refused_naming_it` | an undefined internal name reads 0 | yes |
| `ufo_value_forms::a_coupling_list_is_one_interaction_per_order` | a list keeps its first member | yes |
| `ufo_value_forms::two_couplings_of_one_order_in_a_list_are_refused` | no refusal | yes |
| `ufo_value_forms::a_coupling_order_may_switch_on_a_configuration_flag` | `not` is the identity | yes |
| `ufo_value_forms::a_coupling_order_outside_the_read_forms_is_refused` | a missing flag reads 0 | yes |
| `ufo_value_forms::a_particle_charge_that_is_a_parameters_value_loads` | `default_of_variable` returns 0 | **no** (H2), before and after |
| `ufo::a_charge_naming_a_parameter_reads_its_default_through_parse` (new) | same | yes |
| `ufo_value_forms::a_particle_charge_naming_nothing_is_refused` | an internal parameter's charge reads 0 (`_ => Ok(0.0)`) | yes |
| `ufo_value_forms::an_unreadable_optional_source_file_is_refused_naming_it` (new) | an unreadable file is absent (the pre-fix behaviour) | yes |
| `ufo::the_special_values_…`, `config::sm_full_…`, `config::full_on_…` (H8 helper) | `free_externals` keeps merged followers | yes |
| `test_parse.py` `test_import_flags_may_precede_the_model`, `test_the_model_rewrite_…` | the model is the first token | yes, both |

Two notes on the table. A first attempt at the last `ufo_value_forms` row
returned 0 for a `Some(_)` external, which no internal parameter reaches,
since `default_of_variable` searches only the externals; that run was not a
verdict, so the row is the second mutation. The `check.rs` cases #145 added
cover a parse path (`MODELNAME_OPTIONS`) that #145 did not change, so no
mutation of #145's code reaches them.

**What the tests cannot detect.**
- `Cond`'s laziness: unobservable through `eval` (H5).
- The `max`/`min` tie rule only matters for complex arguments, which
  Python's `max` refuses; the pin is on this evaluator's rule, not MadGraph's.
- The H13 tightening has no mutation that separates old from new: dropping
  the argument filter fails both. The new form rules out a refusal of another
  kind passing.
- `an_unreadable_optional_source_file_is_refused_naming_it` uses invalid
  UTF-8; a permission error takes the same branch but is not run.
- The H2 lib test pins the value read; nothing downstream reads
  `Particle::charge` (Found 2), so no σ can see it.

## Bit for bit

`kh1-bits.sh`: the base is `main` at `b3bc5399`, extracted with `git archive`
and built in its own target dir (deleted afterwards); the fix is `4ccb0dc5`.
Both are `cargo build -p vibegraph` in the `dev` profile, with lhaid 247000
from `validation/pdf` and `VIBEGRAPH_NO_NETWORK=1`. The σ runs are
`integrate <card> --run-card validation/madgraph/<run card> --fixed-budget --neval 20000 --niter 3 --seed 7 -j 2`.
`dy13` is `generate p p > e+ e-` on `dy13_default_run_card.dat`, and
`llj_mlm` is `generate p p > e+ e- j` on `pp_to_llj_mlm_run_card.dat`. The
`enumerate` runs use `--svg all -j 2 --ufo-dir validation/ufo`, and each
digest is the sha256 of every output file's sha256.

```
base: dy13 rc=0 063e6ff624eba7cd62e55269a08b806813899f03032eaccb60c3de9799597f0f σ = 935 ± 11 pb
fix:  dy13 rc=0 063e6ff624eba7cd62e55269a08b806813899f03032eaccb60c3de9799597f0f σ = 935 ± 11 pb
base: llj_mlm rc=0 168ec7ef694648eb21898c8095d8c5b09312d219478662fcc21476fd0924bebf σ = 267.0 ± 2.5 pb
fix:  llj_mlm rc=0 168ec7ef694648eb21898c8095d8c5b09312d219478662fcc21476fd0924bebf σ = 267.0 ± 2.5 pb
base: enum smeft_np rc=0 43e26d58a32c9660f22046db2a6bd836c53d12dfe01e9e6df531e7032a569a61 files=13
fix:  enum smeft_np rc=0 43e26d58a32c9660f22046db2a6bd836c53d12dfe01e9e6df531e7032a569a61 files=13
base: enum smeft_auto rc=0 0ccd369fab8a91f423380f92a724054d0eb25061fcf66f43d0854b1e612c5496 files=6
fix:  enum smeft_auto rc=0 0ccd369fab8a91f423380f92a724054d0eb25061fcf66f43d0854b1e612c5496 files=6
base: enum sm_gluon rc=0 426777c8ac42ff51faad2197241572a85ba36733aaea07e3e044ed678e0fdff0 files=738
fix:  enum sm_gluon rc=0 426777c8ac42ff51faad2197241572a85ba36733aaea07e3e044ed678e0fdff0 files=738
$ diff kh1-bits-base.txt kh1-bits-fix.txt && echo IDENTICAL
IDENTICAL
```

The integrate logs differ only in `ns/eval` timings and the enumeration's
wall time; every per-iteration estimate is identical. The three enumerate
processes each reach a code path the refactors touched:
- `smeft_np` is SMEFTsim `-SMlimit_massless`, `e+ e- > z h h NP=1`: 11
  diagrams, through the signed selector and `reduction.rs`.
- `smeft_auto` is the same process with no orders: 4 diagrams, through
  `search_weighted`.
- `sm_gluon` is `p p > e+ e- j j`: 65 subprocesses, through the gluon-free
  trials.

**Rustdoc (H4).** `cargo doc -p vibegraph-lib --no-deps --document-private-items`
gives 11 warnings at the base, including
`unresolved link to parse_couplings` (`ufo/couplings.rs:115`) and
`unresolved link to parse_particles` (`ufo/particles.rs:184`). The fix gives
9 warnings, the 9 that `rustdoc-warnings-ungated` already lists, and none in
`ufo/`.

## Gates

On `4ccb0dc5` (code), with the three lean variables and `CARGO_BUILD_JOBS=2`:

```
$ cargo fmt --all --check                                   → (no output) FMT-OK
$ cargo clippy --workspace --all-targets -- -D warnings     → Finished `dev` profile [optimized] target(s) in 1m 34s
$ cargo clippy --workspace --all-targets --all-features -- -D warnings → Finished `dev` profile [optimized] target(s) in 56.24s
$ cargo test --workspace                                    → summed test results: 2238 passed, 0 failed, 26 ignored
$ pixi run kb-lint                                          → 1301 concepts, 433 backlog items; 0 errors, 5 warnings (pre-existing)
```

The suite changes by +2: three tests added
(`an_unreadable_optional_source_file_is_refused_naming_it`,
`a_charge_naming_a_parameter_reads_its_default_through_parse`,
`a_gluon_trial_on_a_model_without_a_qcd_hierarchy_is_refused_by_name`) and
one deleted (`the_unselected_branch_is_not_evaluated`). The base count was not
re-run. No banked target was run: none of the changes reaches an amplitude, a
sampler or a reference, and σ is bit for bit above.

## Method

| technique | share | yield |
|---|---|---|
| Reading the three diffs against the checklist | ~25% | H4, H5, H6, H10, H11, H12, H13, H15, H16 |
| Grepping a new item's users workspace-wide | ~10% | H3 (an untested variant), H7, H8, H9, and the no-reader `charge` (Found 2) |
| `grep 'Err(_) =>'`-style reading of fallible paths | ~5% | H1 |
| The mutation batch (49 runs, ~2 h of rebuilds on a copy of the tree, while the fixes went ahead in the worktree) | ~45% | H2, H5 and H6 confirmed; M40's first form showed a mis-aimed mutation |
| The bit-for-bit and enumerate oracle | ~15% | identity of H7 and H12 |

Leads that did not hold:
- `the_unselected_branch_is_not_evaluated` is not fully vacuous: swapping the
  branches fails it. It is a duplicate whose stated claim cannot be seen.
- The `check.rs` cases look new but cover a path that predates #145.
- A tree copy made with `git archive | tar` keeps the commit's mtimes, so
  cargo reran a stale lib test binary in the first post-fix baseline. Touch
  the sources after such a copy.

## Found

1. **The view-based WEIGHTED sites remain** (`weighted-order-computed-three-ways`):
   - the `generate_sets_inner` closure, `diagrams/mod.rs` `weighted_fn`
     (hierarchy times `diag_view.order`);
   - `ufo/topo.rs` `NegativeOrders::weighted`;
   - the test-only `helas/eval/stitching.rs` `weighted`, which still indexes
     `order_hierarchy[order]` and panics on a missing order.

   `chain.rs` and `reduction.rs` now use `order_weight`. Closing the item
   needs a `DiagramView` form of the helper. The item's text names three
   sites and should list these.
2. **`Particle::charge` has no production reader.** #145's parameter-valued
   charge is parsed and pinned, but nothing downstream reads it, so a wrong
   value is invisible to every σ and amplitude. Evidence: `grep -rn '\.charge\b' vibegraph-lib/src`
   outside `ufo/particles.rs` hits only `leg.charge` (the helas external
   leg's own field). Closing it takes either a consumer (charge conservation
   at vertices, as MadGraph's `check_charge` does) or a statement that the
   field is informational.
3. **The selector unit tests assert nothing** (`diagrams/selector.rs:136-177`,
   six `let _ = build_selector(…)` tests). They predate #141. They can fail
   only on a panic, and `apply_signed_constraint`'s `==` and `>` arms have no
   test anywhere: the negative-order table uses `<=`/`=` bounds only. Closing
   it: a toy row with `QED==0` and `NP>0` against MadGraph's counts.
4. **`tests/negative_orders.rs` adds a fifth copy of the diagram-count helper**
   (`count` with a `NoDiagrams` → `None` arm): another site for
   `diagram-count-test-helper-duplicated`.
5. **Two tests hand-copy a UFO directory into a tempdir** with the same
   `read_dir` / `is_file` / `copy` loop: `config.rs`
   `full_on_a_ufo_directory_skips_its_default_restriction`, and this pass's
   `ufo::a_charge_naming_a_parameter_reads_its_default_through_parse` and
   `weighted_search::a_gluon_trial_…`. A shared test helper ("copy a vendored
   UFO and edit one file, asserting the edit hits once") would serve all three
   and the `ufo_value_forms` builder's edit step.

## Brief corrections

- The brief says "delete the closed items' files in your final commit". This
  session closes no item (`closes: []`), so nothing is deleted.
- "Stay inside the files these PRs touched": H7 and H12 edit `diagrams/chain.rs`
  and `diagrams/mod.rs`. Those files belong to the diagrams cluster, but #141
  touched both at the edited lines.
- The release-build rule did not apply: every build here was the workspace's
  `dev` profile, which is optimised (`Finished dev profile [optimized]`).
