---
type: Session Report
title: "HY1 report: hygiene review of the constant-integrand change"
description: "Review of P1 (79eacd9): two unreached guards now tested, admits_every_point kept in step with Cuts by construction, the declaration bound pinned from both sides, a NaN-blind gate fixed."
status: draft
tags: [hygiene, vegas, decays, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-17}
sources:
  - {id: 472ef1f, resource: "https://github.com/nsmith-/vibegraph/commit/472ef1f", title: "cuts: admits_every_point destructures Cuts and SingleLegCut in full"}
  - {id: facc329, resource: "https://github.com/nsmith-/vibegraph/commit/facc329", title: "hadronic: test the single-channel and no-configuration-draw conditions"}
  - {id: ddb8d6c, resource: "https://github.com/nsmith-/vibegraph/commit/ddb8d6c", title: "budget: pin both sides of the constant-declaration bound and the 0/0 case"}
  - {id: 79776cb, resource: "https://github.com/nsmith-/vibegraph/commit/79776cb", title: "cli_decay: a NaN width fails the exact two-body row"}
---
Scope: `git diff origin/main...HEAD` (P1, `79eacd9`). Worktree
`/home/user/wt/vegas-constant-integrand`, branch `vegas-constant-integrand`.
Logs in the session scratchpad, prefix `vegasconst-HY1-`. Cargo ran with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

## Findings (written before any edit)

| id | pt | site | evidence | conf. | disposition |
|---|---|---|---|---|---|
| H1 | 2/4 | `cuts.rs` `admits_every_point` | the body names fields by hand (`self.shat_min_sq`, `self.single.iter()...`); a new `Cuts` or `SingleLegCut` field compiles unnoticed and is treated as inert. Only the 12-threshold test guards it, and only for fields it names. | checked | **fixed** 472ef1f |
| H2 | 3 | `cuts.rs` `admits_every_point` is `pub` | `grep -rn admits_every_point --include=*.rs .` : users are `hadronic.rs` and its tests, all in the crate | checked | **fixed** 472ef1f (`pub(crate)`) |
| H3 | 2 | `hadronic.rs` `has_constant_integrand`, `channel_count() == 1`, `scale_draw_ndim() == 0` | P1 mutations M5, M6 survived. Both are reachable by installing the state on the integrand in a unit test (a two-channel `Sampler::Multi`; the `EventScaleSource` of `u u~ > e+ e- g`, whose `draws_configuration()` is true). A toy model is not needed. | checked | **fixed** facc329 |
| H4 | 2 | `budget.rs` `a_declaration_the_iteration_contradicts_is_withdrawn` | only tilt 0.5 (relative error ~2e-3). Mutation `CONSTANT_REL_ERROR` 1e-7 -> 1e-3 survives. | checked (mutation b2 survived) | **fixed** ddb8d6c |
| H5 | 2 | `budget.rs` the NaN -> infinity branch of the relative-error fold | an integrand of zero level gives `0/0`; mutation NaN -> 0.0 had no test. (I did not run the old suite on this mutation; no old test has a zero-level integrand: `grep -n "level: 0" budget.rs` was empty before.) | checked | **fixed** ddb8d6c |
| H6 | 2 | `cli_decay.rs` exact two-body branch | `fold(0.0, f64::max)` drops NaN, and `parse_width` accepts `NaN` (`"NaN".parse::<f64>() == Ok(NaN)`). Demonstration (`vegasconst-HY1-nan.rs`): old fold on `[NaN, 1.0]` = `0`, new fold = `inf`. | checked | **fixed** 79776cb |
| H7 | 1 | `budget.rs` `integrate_channels` | ~260 lines, P1 added ~35. | checked | **partly fixed** ddb8d6c: the relative-error fold is a helper. The function stays long: Found 1. |
| H8 | 1/2 | `CONSTANT_REL_ERROR` rationale | the doc argues a `sqrt(eps)` = 1.5e-8 floor from worst-case accumulation, with no measured value | checked | **fixed** ddb8d6c: doc cites the measured 2e-9 to 6e-9 (P1 report, 1.2e5 to 1e7 points). The 1e-7 sits ~7x above the worst-case floor and ~17x above the measurement; it is at the algorithm's own error scale, not ULP-exact. |
| H9 | 2 | `cli_decay.rs` `is_two_body` selects by process string | the brief asks whether it duplicates the library's condition | checked | **leave**, see below |
| H10 | 3 | `is_constant`, `has_constant_integrand`, `StopReason::Constant` | trait method needed by `budget`; `has_constant_integrand` follows the inherent+trait pair of `scale_draw_ndim`/`channel_count` and has an outside user (`tests/decay_widths.rs:139`); the variant is matched by `integrate.rs:642` | checked | **leave** |
| H11 | 1 | history/plan references in the diff | `git diff origin/main...HEAD -- vibegraph-lib vibegraph-cli \| grep '^+' \| grep -nE '\bT[0-9]\b|\bC[0-9]\b|Phase [0-9]|note [0-9]+|used to|no longer|P1|sprint|backlog'` returned nothing | checked | no finding |
| H12 | 1 | a second copy of the helper or the toy | `Declared` (budget tests) against `Toy`/`SparseTail`/`TwoDimensions`: different shapes (declares, tilt), not a copy | checked | **leave** |

Backlog check (paths `/budget.rs`, `/cuts.rs`, `/hadronic.rs`, `/unweight.rs`,
`cli_decay`, `decay_widths`, `integrate.rs`): filed items naming them are
`vegas-refinement-noises-constant-integrand` (the claimed one),
`vegas-chi2-per-dof-overflows-on-wide-splits`, `stop-scale-inflated-by-few-accepted-iterations`,
`spacelike-floor-too-loose`, `alpha-survey-iteration-limited-on-wide-splits`,
`adapt-survey-bounds-untested-at-hundreds-of-channels`, `drawn-scale-config-frequency-law-ungated`,
`integrand-methods-duplicated`. None has a site this change makes wrong or
fixed; the `MIN_POSITIVE` floor of the chi2 item is what P1's Found 3 prints as
`1.5e-154`, unchanged.

### Answers to the brief's questions

- **The two guards.** Reachable, by a unit test, without a toy model:
  `hadronic::tests::a_second_channel_or_a_configuration_draw_withdraws_the_declaration`
  builds `z > e+ e-` under the decay card (positive control: one channel,
  `scale_draw_ndim() == 0`, declared), then installs two flat channels, then the
  scale source of `u u~ > e+ e- g`. A finding on the way: `compile_scale_source`
  returns no channel forest when every subprocess has one incoming leg, so for a
  real decay the no-draw condition is structurally always true and its guard is
  defensive. The doc of `has_constant_integrand` now says that.
- **`admits_every_point` kept in step by construction.** Now yes: both structs
  are destructured with no `..`, with the two fields that reject nothing
  (`shat_min_hint`, `bwcutoff`) and the ones that are not thresholds named
  `_` with a reason. Adding a field to `Cuts` fails to compile
  (`error[E0027]: pattern does not mention field 'new_threshold'`,
  `cuts.rs:840`, from a temporary `new_threshold: f64` in `Cuts`, reverted).
- **`CONSTANT_REL_ERROR`.** Rationale was stated; the measured values are now
  in it, and the bound is pinned from above (H4) and below (P1's M10).
- **`cli_decay` exact-width branch.** Left selecting by process string. Asking the
  run for its own log line would route a regression that stops declaring
  constants into the statistical branch, where a noisy but correct exact-width
  row passes; the string selection keeps the row held to 1e-6, which a refined
  grid (1e-4) fails. The dedicated test `a_two_body_width_is_integrated_once_and_a_three_body_one_is_not`
  already asserts the log line. The only structural overlap is the daughter
  count, which is the row's own expectation, not the library's decision.
- **Visibility.** H2 narrowed one item; H10 left the rest.

## Fixed

| id | commit | change |
|---|---|---|
| H1, H2 | 472ef1f | exhaustive destructure, `pub(crate)` |
| H3 | facc329 | new test, doc paragraph on the two guards |
| H4, H5, H7 (part), H8 | ddb8d6c | tilts `[0.5, 1e-3]`; new `a_declared_integrand_that_vanishes_is_withdrawn`; `worst_relative_error` helper; doc |
| H6 | 79776cb | NaN counted as the worst difference |

Tests added: `hadronic::tests::a_second_channel_or_a_configuration_draw_withdraws_the_declaration`,
`budget::tests::a_declared_integrand_that_vanishes_is_withdrawn`. Changed:
`a_declaration_the_iteration_contradicts_is_withdrawn` (loops two tilts). No
test deleted. The helper move is a pure function extraction: identity oracle is
the unchanged budget tests (`a_declared...`, `an_undeclared...`) and the seeded
goldens in `cargo test --workspace`, all passing.

## Mutations

Driver `vegasconst-HY1-mutate.py` (apply, run `cargo test -p vibegraph-lib --lib -- cuts:: hadronic::tests::a_second hadronic::tests::two_beams budget::tests::a_declar budget::tests::an_undecl`, restore); logs `vegasconst-HY1-mutate.log`, `-mutate2.log`.

| mutation | result |
|---|---|
| c1 drop `shat_min_sq` clause | caught: `a_shat_window_or_a_decay_window_withdraws_the_claim` |
| c2 drop `shat_max_sq` | same |
| c3 drop `pairs.is_empty()` | caught: `each_threshold_withdraws_the_claim_that_nothing_is_rejected` |
| c4 drop `mmnl` | same |
| c5 drop `windows` | caught: `a_shat_window_or_a_decay_window...` |
| c6-c11 drop `pt_min`, `pt_max`, `e_min`, `e_max`, `eta_min`, `eta_max` | each caught: `each_threshold_withdraws...` (P1 had mutated only `pt_min`, `eta_max`, pairs, mmnl, windows, ŝ; the four others are new) |
| h1 drop `channel_count() == 1` | **caught**: `a_second_channel_or_a_configuration_draw...` (P1: survived) |
| h2 drop `scale_draw_ndim() == 0` | **caught**: same test (P1: survived) |
| b1 bound 1e-7 -> 1e-1 | caught: `a_declaration_the_iteration_contradicts_is_withdrawn` |
| b2 bound 1e-7 -> 1e-3 | caught after the change; **survived** before it (tilt 0.5 gives ~2e-3 > 1e-3) |
| n1 NaN -> 0.0 in `worst_relative_error` | caught: `a_declared_integrand_that_vanishes_is_withdrawn` |

Not reached by any mutation: the `unpolarized` condition at the unit level
(P1's M3 is caught only by `decay_widths`); `shat_min_hint`/`bwcutoff` named
`_` (they cannot reject a point by `Cuts::pass`, which reads neither:
`grep -n 'shat_min_hint\|bwcutoff' cuts.rs` shows no use in `pass`). The
cli_decay NaN fix has no mutation through the binary (a NaN width is not
producible by the real CLI); its evidence is the fold demonstration of H6.

## Gate

Each at `79776cb`, from `vegasconst-HY1-gate.sh`; status lines in
`vegasconst-HY1-gate.status`.

| command | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 |
| `cargo doc -p vibegraph-lib --no-deps --document-private-items` | exit 0, 8 warnings, none in `hadronic/budget/cuts/unweight.rs` (`grep -cE 'src/(hadronic\|budget\|cuts\|unweight)\.rs'` = 0); plain `--no-deps`: 6 warnings, same files untouched. I did not run the base for a count; the warnings sit in `coupling/`, `phasespace/channel.rs`, `ufo/parameters.rs`, `lhef/mod.rs`. |
| `cargo test -p vibegraph-lib --lib` | `997 passed; 0 failed; 10 ignored` (+2: my two new tests) |
| `cargo test -p vibegraph-lib --test decay_widths` | `5 passed; 0 failed; 1 ignored` |
| `cargo test -p vibegraph --test cli_decay -- --test-threads=1` | `3 passed; 0 failed`, 85.7 s |
| `cargo test --workspace` | exit 0; 36 test binaries `ok`, 0 failing; 1490 passed in total |

Banked/extended-validation targets: none run; the change touches only the
decay path (`InitialState::Decay`, pinned by `two_beams_into_two_bodies...`) and
test code, and a scattering run is unchanged code. The manager's
`pixi run --skip-deps validate` remains P1's Found 6.

## Method

Reading the diff against the brief (30%); mutation batches on the guard and
the cuts clauses (35%, each ~3 min of rebuild on a loaded host); tracing
callers (`compile_scale_source`, `for_decay`) to settle reachability (15%);
greps for history references, outside users, backlog (10%); the NaN
demonstration (5%). Leads that did not hold up: the `Declared` toy is not a
copy of `Toy`; the process-string selection in `cli_decay` is the stronger
oracle; every `pub` item of P1 has a user or is required by a trait or match.

## Found

1. **`integrate_channels` (`budget.rs:669`) is ~260 lines.** One iteration of the
   loop does planning, drawing, recording, reporting, stopping and refinement
   in line. The declaration check is now a helper call, but the function needs
   splitting (plan / record / report / decide). Close with an extraction that
   keeps the seeded goldens bit-equal. This is a hot-loop restructure, so
   performance-dev.
2. **The inactivity sentinels live twice.** `Cuts::pass` and `admits_every_point`
   each encode "a lower bound at or below zero, an upper bound below zero is
   off" per family. A `SingleLegCut`/`PairCut` method (`is_inactive`) used by both
   would put the convention next to its type and let `pass` skip an inactive
   family from one definition. Not done: it touches `pass`, the hot cut path.
3. **P1's Found 1-6 stand** (α survey on a single channel; `generate` overweight
   on exactly flat weights; `± 1.5e-154` print; polarized final legs excluded;
   no `validate` run). Nothing in this review moves them.
4. **`has_constant_integrand` doc previously claimed the six conditions "together
   sufficient"** while two could never be false for a real decay; the doc now
   says so. If a model with two vertices on one `1 -> 2` triple is ever added, a
   gate on its width against an analytic oracle would be the first real
   exercise of the single-channel condition.

## Brief corrections

- The brief offers "a toy-model decay with two diagrams" as the route to the two
  guards; neither needed one (direct installation on the integrand reaches both).
- It says the decay-chain/`mmnl`/threshold families are covered by the 12
  thresholds test; the single-leg `pt_max`, `e_min`, `e_max`, `eta_min` clauses
  were covered by it, but never mutated until now.
