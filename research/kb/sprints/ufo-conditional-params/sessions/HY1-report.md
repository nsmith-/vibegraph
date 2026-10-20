---
type: Session Report
title: "HY1 report: hygiene review of the conditional-definition refusals and the fallible recompute"
description: "Review of F1 and F2: a dead error branch in the no-aS rescale path replaced by a stated invariant, an unchecked public launch name now refused, untested refusals pinned, one displaced doc comment and duplicated formatting merged."
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-20}
---
## Findings (written before any edit)

| id | pt | site | evidence | confidence | disposition |
|---|---|---|---|---|---|
| H1 | 1 | `ufo/mod.rs` `recompute_refuses_an_unknown_parameter_and_changes_nothing` | F2 inserted the test between `Moving a parameter the restriction locked to zero...` and its `#[test]`; the doc now describes the wrong test and the locked-parameter test has none | checked (diff) | fixed |
| H2 | 1/2 | `helas/eval/rescale.rs` `set_alpha_s` and `sqrt_law_violation`, new `is_err()` branches | `model` is `Some` only if `plan.fallback.is_some()`, which needs `scale_dependent`, i.e. a pool entry tagged with a non-zero power of `G`; `g_powers` (`fold.rs:600`) tags `G` only if `driven` holds it, and `driven = dependents("aS") + aS`, empty of `G` for a model with no `aS` (F1's load check refuses a coupling naming an undefined `aS`). So a no-`aS` model takes the `!scale_dependent` return and never reaches the `is_err()` branch, and the `NoReferenceCoupling` returned from the probe is a false reason. A zero-restricted `aS` returns `Ok` (the name exists). No test of the no-`aS` model existed | checked (reasoning plus mutation M1/M1b) | fixed: replaced with `expect` of the invariant, plus a pinning test |
| H3 | 1/4 | `reweight/engine.rs` four `expect("...checked when the card is read")` | `ReweightPlan::new` is `pub` and takes `Vec<Launch>` with `pub` fields; a hand-built launch with an unknown name panics (in `check_finite`, before the old text's claim of a card check holds). Four copies of the same loop-and-expect | checked (test below fails before) | fixed: one check at the top of `new` returning `ReweightError::UnknownParameter`, one `move_parameter`/`moved_to` helper carrying the single `expect` |
| H4 | 4 | `ufo/mod.rs` and `parameters.rs` | the quote-and-join of conditional names appears twice | checked | fixed: `ast_util::conditional_definitions`; `conditional_constructor_names` narrowed to private |
| H5 | 2 | `tests/toy_models.rs` | no test for `coupling_orders.py` and `propagators.py` refusals (mutations M3, M4 pass the old tests); the coupling and Lorentz asserts check only `contains("'name'")`, so a refusal labelled with the wrong file passes (M5) | checked | fixed |
| H6 | 4 | `refuse_conditional_definitions` call sites vs readers | `mod.rs` repeats the (file, ctor) literals and each reader (`couplings.rs:82`, `lorentz.rs:240`, `particles.rs:155`, `vertices.rs:50`, `propagators.rs:50`, `mod.rs:103`) repeats its own constructor literal; two lists that can drift | checked | leave: unifying means threading a table through six readers with differing parse shapes; the new test checks all six constructor/file pairs, so a drift fails it |
| H7 | 3 | `UnknownParameter` (pub, `pub String` field), `ParameterSet::defines`, `conditional_constructor_names` | `UnknownParameter` is returned by pub `recompute`/`set_alpha_s`, so must be pub; `defines` is used in `ufo/mod.rs` only (`pub(crate)` matches the neighbours); `conditional_constructor_names` has users only in `ast_util.rs` after H4 | checked | `conditional_constructor_names` narrowed (H4); others leave |
| H8 | 1 | history-reference grep over the diff's added lines | no matches | checked | none |
| H9 | -- | backlog items naming the reviewed paths | none concern these sites (`probe-points-duplicate...` names engine.rs:~962 test points, unrelated) | checked | none |

`try_edited_toy_model`: shared with `edited_toy_model`, minimal. Its `appends` now create the file when absent (needed for `propagators.py`).

## Fixed

H1-H5 in `b810f3d`.

## Mutations

Driver `/tmp/.../ufoC-HY1-mut.py` applies, runs and reverts each.

| id | mutation | new test | old tests |
|---|---|---|---|
| M1 | `model = Some(..)` always and no `scale_dependent` early return in `ScaleAwareAmplitude` | `a_model_without_a_strong_coupling_ignores_the_per_event_alpha_s` FAILS (`expect` at `rescale.rs:200`) | no test existed |
| M1b | early return inverted alone (`|| self.plan.scale_dependent`) | passes: **blind spot**. A no-`aS` model has every pool power zero, so a rescale multiplies nothing and the early return is unobservable; no mutation reaches it | -- |
| M2 | engine name check disabled | `a_launch_naming_an_unknown_parameter_is_refused` FAILS (panic at `engine.rs:442`, the former behaviour) | no test existed |
| M3 | `CouplingOrder` refusal removed | `conditional_coupling_lorentz_order_and_propagator_definitions_are_refused` FAILS | passes (never touched `coupling_orders.py`) |
| M4 | `Propagator` refusal removed | same test FAILS | passes (same reason) |
| M5 | Lorentz refusal labelled `couplings.py` | same test FAILS (file assert) | passes (name-only assert) |
| M6 | Lorentz refusal removed | same test FAILS | old test fails (as F2 recorded) |

The test for H2 does not distinguish the two `!scale_dependent`/`is_err` outcomes of the old code (M1 on the old code returned silently); its value is that the path is now exercised on a real no-`aS` model and a future route into `EvaluatedModel::set_alpha_s` panics loudly instead of silently.

## Gate (/home/user/wt/C, `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`)

- `cargo fmt --all --check` -> `fmt=0`
- `cargo clippy --workspace --all-targets -- -D warnings` -> `Finished ... clippy=0`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> `Finished ... clippy-all=0`
- `cargo test --workspace` -> `test=0`; 37 `test result: ok`, none failing, 1550 passed. Added tests: `toy_models::a_model_without_a_strong_coupling_ignores_the_per_event_alpha_s`, `toy_models::conditional_coupling_lorentz_order_and_propagator_definitions_are_refused` (replaces `conditional_coupling_and_lorentz_definitions_are_refused_by_name`), `reweight::engine::tests::a_launch_naming_an_unknown_parameter_is_refused`.
- `cargo doc -p vibegraph-lib --no-deps --document-private-items` -> `doc=0`, 8 warnings, none in changed code (the `parameters.rs:274` one is the pre-existing `Vec<i32>` link).
- `cargo test -p vibegraph-lib --profile release-debug --features extended-validation,bench-internals --test validate_scale_couplings` (the `rescale.rs` change) -> `scaling_path_reproduces_the_reference_path ... ok`.
- `python3 scripts/kb.py lint` -> `827 concepts, 289 backlog items; 0 errors, 4 warnings` (after `kb.py index` refreshed the sessions index).

## Found

- (suspected) `ReweightPlan::new`'s new check accepts any name in `base.param_values`, internal parameters included, whereas `resolve` allows externals only. A hand-built `Launch` naming an internal parameter reaches `recompute` and moves it without its externals. Close by checking `model.params.externals` in the same check; not settled here.
- The (file, constructor) literals of the six UFO readers and of `ParsedModel::parse`'s refusals are two independent lists (H6).
