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

Written before any code edit. Point: 1 maintainability, 2 non-vacuity, 3
visibility, 4 abstractions. Confidence: *checked* (the quoted evidence settles
it) or *suspected* (the stated run settles it). Ranked by consequence;
disposition *fix* marks what this session fixes.

| id | rank | point | site | what is wrong (class) | evidence | proposed fix | confidence | disposition |
|---|---|---|---|---|---|---|---|---|
| H1 | 1 | 1, 2 | `ufo/mod.rs:335`, `:353` (and `:399`, `:417`) | A read error on an optional source file is swallowed (panic-or-unwrap class: a fallible path read as absence). `Err(_) => FunctionLibrary::default()` treats an unreadable or non-UTF-8 `function_library.py` as no library, so a SMEFTsim-style model silently evaluates `sec`/`csc`/`asec`/`acsc` on the complex argument instead of its own real-part definitions: a silent wrong answer. `configuration.py` likewise reads as empty. The two older sites (`coupling_orders.py`, `propagators.py`) have the same shape and sit in the same function. | `match std::fs::read_to_string(path.join(OPTIONAL_SOURCE_FILES[3])) { Ok(src) => FunctionLibrary::parse(&src)?, Err(_) => FunctionLibrary::default() }`; the required files map every error to `UfoError::Io` (`:316`). | One `read_optional` helper: `NotFound` is absence, any other error is `UfoError::Io` naming the file. A test with a non-UTF-8 `function_library.py` fails before and passes after. Name the four files instead of indexing `OPTIONAL_SOURCE_FILES[0..3]`. | checked (code); the test run settles the behaviour | fix |
| H2 | 2 | 2 | `tests/ufo_value_forms.rs:362` `a_particle_charge_that_is_a_parameters_value_loads` | Vacuous test: it asserts only that the model loads, never the charge. | Mutation: `default_of_variable` returns `Ok(0.0)` for every external; the test reads nothing that changes. | Assert `S`'s charge is `MS`'s default, 10. | suspected; the mutation run settles it | fix |
| H3 | 3 | 2 | `diagrams/mod.rs:117`, `:1018` | `DiagramError::GluonTrialWithoutQcd` has no test: `grep -rn GluonTrialWithoutQcd` returns only its definition and its construction. | grep output above. | A test on a scratch model whose gluon (PDG 21) sits in the lowest class of a hierarchy without QCD, asserting the variant; it fails under `unwrap_or(&0)` in place of the refusal. | checked | fix |
| H4 | 4 | 3, 1 | `ufo/couplings.rs:109-117`, `ufo/particles.rs:166-191` | A production contract documented on a `cfg(test)` item, and production docs linking it. #145 made `parse_couplings` and `parse_particles` `#[cfg(test)]` wrappers but left the contract (`.anti()` handling, anti-particle entries, skipped attribute assignments) on `parse_particles`, while the production `parse_particles_with` and `parse_couplings_with` say only "[`parse_particles`], reading …", a link that does not resolve outside `cfg(test)`. | the quoted docs; `cargo doc -p vibegraph-lib --no-deps --document-private-items` (run after the first build) | Move the contract onto the production functions; the test wrappers point at them. | checked (code); the doc build settles the warning | fix |
| H5 | 5 | 2 | `ufo/expr.rs:670` `the_unselected_branch_is_not_evaluated` | The test claims lazy evaluation of `Cond`, which its assert cannot see: `eval` is total, so an eager evaluation of `1/x` at `x = 0` yields a NaN that is discarded, and the result is still 0. What it does check is `conditional_expressions_select_a_branch`'s first case. | Mutation: evaluate both branches, then select. | Delete it (a duplicate that cannot fail on its claim); say in the report that the laziness is unobservable through `eval`. | suspected; the mutation run settles it | fix |
| H6 | 6 | 2 | `ufo/expr.rs:691` `max_min_and_the_dm_width_form` | Its doc claims `max`/`min` return the first of equal arguments; no assert has a tie whose members differ, so the tie rule is unpinned. | Mutation: `v.re > best.re` → `v.re >= best.re` in `eval`'s `Max`/`Min`. | Add a tie whose arguments differ in their imaginary part. | suspected; the mutation run settles it | fix |
| H7 | 7 | 4, 1 | `diagrams/chain.rs:305`, `diagrams/reduction.rs:210` | A second and third copy of `ufo::weighted::order_weight` (each order's power times its hierarchy, a missing order weighing 0). Filed as `weighted-order-computed-three-ways`; #141 added the `reduction.rs` instance and retyped the `chain.rs` one. | `n * model.order_hierarchy.get(name).copied().unwrap_or(0) as i64` (chain), `weighted += n * i64::from(model.order_hierarchy.get(name).copied().unwrap_or(0))` (reduction). | Both call `order_weight`. Integer arithmetic: identity by construction, checked by the enumerate oracle. The view-based sites stay (Found). | checked | fix |
| H8 | 8 | 4 | `config.rs:443` `free_parameters`, `ufo/mod.rs:2250` | The same "free externals" filter (not zeroed, not fixed at one, not a merged follower) written twice by #144's two tests. | Both bodies `flat_map(\|group\| group.iter().skip(1).map(\|(name, _)\| name))` then filter `zeros`/`ones`. | One `#[cfg(test)]` `ParameterSet::free_externals` in `parameters.rs`, used by both. | checked | fix |
| H9 | 9 | 3 | `ufo/loaded.rs:24`, `:114`, `:135` | `pub` with no user outside the crate: `FULL_RESTRICTION`, `LoadedModel::from_dir_full`, `LoadedModel::interned_sm_full`. | `grep -rn "FULL_RESTRICTION\|from_dir_full\|interned_sm_full"` hits only `loaded.rs` and `config.rs`. | `pub(crate)`. | checked | fix |
| H10 | 10 | 1 | `helas/eval/fold.rs:729-738` | A comment separated from its arm: #145 inserted the `Cond`/`Cmp`/`Logic` arm between the `complex(re, im)` comment and `Expr::Complex`, so the comment now sits above the wrong arm. | quoted lines 729-738. | Move the arm above the `complex` comment. | checked | fix |
| H11 | 11 | 1 | `diagrams/selector.rs:173` | Comment contradicts the change: "Negative coupling powers are physically nonsensical" in the test #141 touched, while #141 reads SMEFTsim's `QED: -1` powers. | quoted line. | Say what the test pins: a negative bound builds without a panic. | checked | fix |
| H12 | 12 | 1 | `diagrams/mod.rs:733` `generate_undecayed` | 186 lines; #141 grew it by the gluon-free trial loop. | `sed -n 733,918p`. | Move the automatic WEIGHTED search into its own function. Identity oracle: `enumerate` output byte-identical. | checked | fix |
| H13 | 13 | 2 | `ufo/functions.rs:208` | Any-refusal assert: the two-argument definition is checked with `is_err()`. | quoted line. | Match `FunctionLibraryError::UnknownDefinition` naming `sec`. Mutation: drop the `arguments == ["z"]` filter. | checked | fix |
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
