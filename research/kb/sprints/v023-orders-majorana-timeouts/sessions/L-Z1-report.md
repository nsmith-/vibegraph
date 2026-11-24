---
type: Session Report
title: "L-Z1 report: hygiene pass on squared orders and per-event-helicity reweighting"
description: "Findings, mutations and fixes for sprint L's PR 1 (#168, squared-order split) and PR 10 (#182, per-event-helicity reweighting), with the bit-for-bit identity oracle against main."
status: draft
tags: [hygiene, squared-orders, reweight, helicity, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-hyg-amp` off main 8b3af5ab. Scope: the squash commits
`4b132cc5` (#168) and `dfdf85d4` (#182), their direct callers and tests.

## Findings (written before any edit)

Ranked by consequence. Points: 1 maintainability, 2 non-vacuity, 3 visibility,
4 abstractions. *Fix* marks the ones fixed here.

| id | pt | site | what is wrong | evidence | disposition |
|---|---|---|---|---|---|
| F1 | 4 | `reweight/engine.rs:441-447`, `:527-533`; `:455-466`, `:538-555` | Second copy of the split read-out: the node quadratic form `uᵀRu` is the same eight lines in `Reweighter::ratios` and `BoundSub::ratios_in`, and the exact-slot memo loop is copied with only the per-slot evaluation changed. | `git show dfdf85d4 -- vibegraph-lib/src/reweight/engine.rs`: the `for (j, &uj) in u.iter().enumerate() { let row = &r[j * k..(j + 1) * k]; …` block appears twice, identical. | **fix**: one `quadratic_form(r, u)` and one slot-memo helper taking the evaluation as a closure. |
| F2 | 3/1 | `helas/eval/lower.rs:36-88` | The production lowering's contract (the JAMP formula, the `CoeffRat` rule, the configs bundle) is documented on `lower_flows`, which #168 made `#[cfg(test)]`; `lower.rs:38` says "the production path is [`lower_flows`]", false since `compile` calls `lower_split_flows`. Seven doc sites name `lower_flows` as what `compile` runs: `lower.rs:89,98,190`, `compile.rs:1460`, `eval/mod.rs:10`, `egraph.rs:332`, `run/probe_bridge.rs:6`. | `grep -rn lower_flows vibegraph-lib/src` gives one call, `compile.rs:1477`, inside `#[cfg(test)]`. | **fix**: delete the test-only wrapper, give its name and its doc to the one production function (`lower_flows(…, class_of)`), the one test caller passes `None`. |
| F3 | 1 | `helas/eval/lower.rs:151` | "reduces to today's color-free lowering": temporal framing (AGENTS.md comment rules). | `git show 4b132cc5 … \| grep today` | **fix** |
| F4 | 1 | `generate/audit.rs:166-168` | `finish`'s doc lists the subprocess and non-finite refusals only; #182 added three more it makes (refused helicity, identical particles under different mothers, helicity denominator mismatch). | quoted doc vs the `why` chain at `audit.rs:205-226`. | **fix** |
| F5 | 4 | `generate/audit.rs:89`, `:125` | The relative-deviation expression `(a - b).abs() / b.abs().max(f64::MIN_POSITIVE)` written twice in one function. | quoted lines. | **fix**: one helper. |
| F6 | 4 | `hadronic.rs:1684-1696` | `drawn_helicity` finds the drawn combination's index again by a linear `position` over `helicities()` after `select_helicity`, its only caller, already had it from `select_index`; the second lookup's `?` cannot fail. | `grep -rn "select_helicity(" vibegraph-lib/src`: one call, `hadronic.rs:1691`. | **fix**: `select_helicity` returns the index. |
| F7 | 3 | `diagrams/squared.rs:118-178`, `:396` | `SquaredOrders::{diagram_orders, accepts, classes, selected_pairs, is_signed, is_signed_on}` and `annotate_sets` are `pub` with in-crate users only (`compile.rs`, `generate.rs`, `artifact.rs`, `diagrams/artifact.rs`); two `impl SquaredOrders` blocks for no reason. | `grep -rn` of each name over `vibegraph-lib vibegraph-cli`: no hit in `tests/` or the CLI. | **fix**: `pub(crate)` (private where only the module reads it), one impl block. |
| F8 | 1 | `diagrams/squared.rs:349-358` | `filter` builds a throwaway `SquaredRequest { constraints: constraints.clone() }` only to read `constraints[k]` back out of it. | quoted lines. | **fix** |
| F9 | 3 | `reweight/card.rs:110` | `ReweightCard::helicity` is a `pub(crate)` field beside the derived `helicity()` (default applied) and `stated_helicity()`; nothing outside `card.rs` reads the field, and a crate reader of it would skip MadGraph's default. | `grep -rn "\.helicity\b" vibegraph-lib/src` outside `card.rs`: only `options.helicity` / `plan.helicity`, other types. | **fix**: private. |
| F10 | 2 | `vibegraph-cli/tests/cli_reweight_helicity.rs:153-159` | The contradicted-flag refusal asserts `"--reweight-helicity"` and `"change helicity"`, both substrings of `SUMMED_REMEDY`, which every per-event refusal prints: it does not assert which refusal fired. Mutation that survives: `helicity_word(flag)` → `helicity_word(line)` in `vibegraph-cli/src/generate.rs` (the refusal names the wrong flag). | mutation table below. | **fix**: assert the refusal's own head, `--reweight-helicity <flag>: … says \`change helicity <True\|False>\``. |
| F11 | 3 | `helas/eval/compile.rs:342-347` | `select_helicity`'s signed branch draws `∝ |T_c|`; no production path reaches it: `generate` refuses a signed selection (`generate.rs:427`) before any draw, and `select_helicity`'s only caller is the generation integrands. | `grep` above; mutation M11 below. | **leave**: a guard that keeps the draw defined (MadGraph's `DABS`) if the refusal is lifted; reported, no test can reach it through `generate`. |
| F12 | 2 | `reweight/engine.rs:748` | `ReweightError::HelicitySigned` is reachable only through `ReweightPlan::new` called directly (unit tests, `reweight_mg_oracle`): `generate` refuses a signed set first. | as F11. | **leave**: the plan is a public API with its own contract; the unit test reaches it. |
| F13 | 1 | `reweight/engine.rs:756-758` | `.expect("probed at the card's point, so at every point")` in `check_event_helicity`. | `contributing_helicities` returns `None` only on the evaluator's shape (`n_ext`, `n_in`), never on the point. | **leave**: an invariant, not a fallible path. |

Test non-vacuity (point 2) is in the mutation table: one mutation per test the
two PRs added, plus the split's runtime sites (M9-M12).

Found during the fixes (after the table above was committed in c4da11c2):

| id | pt | site | what is wrong | evidence | disposition |
|---|---|---|---|---|---|
| F14 | 1 | `hadronic.rs:1678-1696` | #182 inserted `drawn_helicity` between `zeroed_diagonals`' doc comment and its `fn`: the rustdoc of `drawn_helicity` began with `zeroed_diagonals`' paragraph, and `zeroed_diagonals` had none. | `sed -n 1678,1700p` at c4da11c2; `git show dfdf85d4^:vibegraph-lib/src/hadronic.rs` has the paragraph directly on `zeroed_diagonals`. | **fixed** with F6 (the helper is gone). |
| F15 | 1 | `diagrams/squared.rs:26` | Redundant explicit link target ``[`DiagramSet`](super::DiagramSet)``, a `cargo doc` warning #168 added. | `cargo doc -p vibegraph-lib --no-deps --document-private-items`: "redundant explicit link target --> vibegraph-lib/src/diagrams/squared.rs:26:20". | **fixed** |
| F16 | 4/2 | `diagrams/squared.rs:189-205` | `orders_of` hand-wrote the weighted order `Σ n·hierarchy` that `ufo::weighted::order_weight` already computes (and unit-tests); and no test reached it: M7 survived, because `weighted_exact_and_lower_bounds_filter_the_weighted_order` exercises the amplitude-level `WEIGHTED==`/`>` filter (`diagrams/mod.rs:929`), not `WEIGHTED^2`. | M7 below; `grep -rn "fn order_weight"`. | **fixed**: `orders_of` calls `order_weight`; new test `a_weighted_squared_order_weighs_each_order_by_its_hierarchy` (N7). |
| F17 | 2 | `helas/eval/run.rs:460-463` (split `JAMP2`), `:718-720` (pruning moduli), `compile.rs:342-347` (signed draw) | No hermetic test reaches the `.abs()` of the split `JAMP2` (M10), the moduli in `mark_contributing_helicities` (M12) or the signed helicity draw (M11). | M10, M10L, M11, M12, M12L, N12 survived. | **Found** (validation-dev): a test of each needs a reference or a constructed signed term; a pruning test over the three banked tables (N12) did not reach M12 and was deleted rather than shipped. |

Final dispositions: F1-F10, F14-F16 **fixed**; F11-F13 **leave** (reasons in
the table); F17 **Found**. No finding was routed for physics or performance.
The backlog has no filed item on any site above (`grep -rlE` of the path
tails over `research/kb/backlog`: the hits, `probe-points-duplicate-generic-probe-points`,
`colour-sum-dense-over-madgraph-flow-basis`, `reweight-forbidden-schannel-and-as-refused`,
name other sites in these files).

## What changed

| commit | findings | change |
|---|---|---|
| f36c5e53 | F1 | `quadratic_form` and `BoundSub::exact_ratios` serve both ratio paths of `Reweighter::ratios`. |
| f818b97f | F2, F3, F6, F14 | `lower_split_flows` is `lower_flows(.., class_of)` with the production doc; the test-only wrapper is deleted. `select_helicity` returns `(combination, its |M|² entry)`; `drawn_helicity` is deleted, and with it the misplaced doc. |
| 0fac1ab3 | F4, F5 | `ReweightAudit::finish` doc names every refusal; `relative_deviation` helper. |
| cc6d794d | F7, F8, F9, F15 | `SquaredOrders` helpers and `annotate_sets` to `pub(crate)` or private, one impl block; `filter` reads the resolved constraint directly; `ReweightCard::helicity` private; doc link. |
| e38a38c9 | F10 | the contradicted-flag assertion reads the refusal's own head. |
| b165dff8 | F16 | `orders_of` calls `order_weight`; new test `a_weighted_squared_order_weighs_each_order_by_its_hierarchy`. |

Tests added: `a_weighted_squared_order_weighs_each_order_by_its_hierarchy`
(`vibegraph-lib/tests/diagrams.rs`). Deleted: none (a pruning test was written
and deleted before any commit, see F17). Hermetic suite 2352 → 2353 passed.

## Identity oracle

Chosen before the first edit. Main 8b3af5ab built in
`/home/user/wt/lz1-main-target` (deleted afterwards) from a detached worktree,
the branch in its own target, both `cargo build -p vibegraph --bin vibegraph`
(dev profile). Driver `oracle.sh <bin> <outdir>`, `LHAPDF_DATA_PATH=validation/pdf`,
`VIBEGRAPH_NO_NETWORK=1`, every integrate
`--fixed-budget --neval 20000 --niter 3 --seed 7 -j 2`; the reweight runs are
`generate rw/grid.bin.zst --seed 7 --nevents 500 --reweight-card <card> --reweight-helicity {event,summed}`
with the `cli_reweight_proton` card (`wz`, `nominal`, `aew`) on
`p p > l+ l- j QCD=2 QED=2` under `output/pp_to_llj_fixed/Cards/run_card.dat`.

| run | σ main | σ branch (b165dff8) | sha256 (main = branch) |
|---|---|---|---|
| `dy13_proc_card.dat` + `dy13_default_run_card.dat` | 935 ± 11 pb | 935 ± 11 pb | grid `0642e8166775ce67…` |
| `p p > e+ e- @0; add process p p > e+ e- j @1` + `pp_to_llj_mlm_run_card.dat` | 932 ± 12 pb | 932 ± 12 pb | grid `bba47f4d366b684e…` |
| `u u~ > u u~ QCD^2==2`, fixed 250+250 GeV beams | -50.024 ± 0.065 pb | -50.024 ± 0.065 pb | grid `f8c0503942d612a6…` |
| `p p > l+ l- j QCD=2 QED=2` (reweight grid) | 422.7 ± 3.8 pb | 422.7 ± 3.8 pb | grid `69a12293e19a0c2a…` |
| reweight, per-event helicity | | | LHE `b2f189f0e65e2a2e…` |
| reweight, helicity-summed | | | LHE `180a3175150942309…` |

The LHE hashes are taken with the output directory name in the header's
`artifact` line normalized (`sed 's#lz1-oracle-<side>#X#'`): that line is the
only difference, `diff` shows line 5 alone. `diff` of the two summaries with
the LHE lines dropped: empty (`SUMMARY-IDENTICAL`). Both reweight runs report
`500 events weighted in their own helicity; … largest relative deviation 0.0e0, beyond 1e-9: 0; refused: 0`
(event) and `each ratio helicity-summed` (summed) on both sides.

What it cannot see: `select_helicity`'s signed branch (no signed selection
reaches `generate`), the split `JAMP2` on a process whose events are drawn
(none of the four runs is split and generated), and a `WEIGHTED^2` selection.
The F16 change is integer-identical (`n * h` against `h * n` in `i64`) and
pinned by its new test.

## Mutations

Driver `mutate.py <repo> <log> <ids>`: apply one exact-string replacement,
run the command under `flock`, restore the file, log CAUGHT (non-zero exit with
the named test panicking), SURVIVED or COMPILE-FAIL. M1-H10 ran in a scratch
worktree at c4da11c2 (old tests); N7-N12 in this worktree on the new tests.
The first batch hit the 2 h background limit during H3, which left its
mutation applied in the scratch worktree; it was reverted and H3-H10 rerun.

| id | site | mutation | test | result |
|---|---|---|---|---|
| M1 | `squared.rs:68` | `order <= value` → `order < value` | `the_pair_selection_follows_the_summed_order` | caught |
| M2 | `squared.rs:127` | `.all(` → `.any(` | `several_constraints_must_all_accept` | caught |
| M3 | `squared.rs:424` | selection not stored | `round_trip_derives_the_squared_order_selection` | caught |
| M4 | `check.rs:738` | `ExactEq \| Gt` → `Gt` | `a_negative_squared_order_is_supported_under_le_only` | caught |
| M5 | `check.rs:674` | `in_chain = false` | `a_squared_order_in_a_decay_chain_is_refused` | caught |
| M6 | `squared.rs:175` | `is_signed_on` → `false` | `a_signed_squared_order_selection_is_refused` | caught |
| M7 | `squared.rs:198` | hierarchy dropped from `WEIGHTED` | `weighted_exact_and_lower_bounds_filter_the_weighted_order` | **survived** (F16) |
| N7 | `squared.rs` (after F16) | `order_weight(..)` → `orders.values().sum()` | `a_weighted_squared_order_weighs_each_order_by_its_hierarchy` | caught (`diagrams.rs:194`) |
| M8 | `squared.rs:339` | filter never drops | `a_squared_order_no_term_meets_leaves_no_diagrams` | caught |
| M9 | `compile.rs:820` | every class pair weighted | `tests/squared_orders` | caught (`squared_order_classes_match_madgraph_per_helicity_and_flow`) |
| M10 / M10L | `run.rs:462` | split `JAMP2` without `.abs()` | `tests/squared_orders`; lib `jamp2` | **survived** (F17) |
| M11 | `compile.rs:346` | signed draw off the signed weights | lib `helicity` (11 tests) | **survived** (F11, F17) |
| M12 / M12L | `run.rs:719` | pruning on signed `T_c` | `tests/squared_orders`; lib `helicit` (15 tests) | **survived** (F17) |
| N12 | `run.rs:719` | same | a new pruning-identity test over the three tables | **survived**; test deleted |
| — | — | — | `a_rotated_class_assignment_is_seen` | a self-test of `squared_order_classes_…`'s power (it rotates the class pairing itself); no code mutation applies |
| H1 | `engine.rs:596` | `combo` span → `0..n` | `per_event_helicity_ratios_reproduce_direct_evaluation` | caught |
| H2 | `engine.rs:512` | zero card `|M|²` not refused | `per_event_helicity_refusals_are_named` | caught |
| H3 | `engine.rs:759` | revival never found | `per_event_helicity_refuses_what_it_cannot_serve` | caught |
| H3S | `engine.rs:748` | signed not refused | same | caught |
| H4 | `engine.rs` `Unknown` message | remedy printed as `{:.0}` (empty) | `every_per_event_helicity_refusal_names_both_remedies` | caught |
| H5 | `audit.rs:92` | helicity mismatch never counted | `a_refused_helicity_removes_its_file` | caught |
| H6 | `card.rs:283` | first `change helicity` wins | `change_helicity_selects_the_ratio` | caught |
| H7 | `hadronic.rs:1696` | drawn entry → `hel_m2[0]` | `cli_reweight_helicity_follows_the_card_and_the_flag` | caught (`:105`, the run refused) |
| H8 | `audit.rs:262` | mothers never compared | `cli_a_default_per_event_refusal_names_both_remedies` | caught |
| H9 | `cli/generate.rs:322` | contradiction never refused | `cli_reweight_helicity_follows_the_card_and_the_flag` | caught (old test `:152`; new `:154`) |
| H10 | `cli/generate.rs:327` | message names `line` for `flag` | old test | **survived** (F10) |
| N10 | same | same | new test | caught (`:155`) |

Not mutated: the `extended-validation` targets these PRs touched
(`reweight_mg_oracle`, `cli_reweight_proton`); they were run as gates, below.

## Gates (at b165dff8; the doc fix F15 and F16 postdate the first runs, all rerun)

- `cargo fmt --all --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: `clippy rc=0`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: `clippy-all rc=0`.
- `cargo test --workspace`, hermetic (`research/refs/mg5amcnlo`,
  `validation/madgraph/output`, `validation/pdf` moved aside and restored by
  an `EXIT` trap): `TEST rc=0`, `RESTORED`; 79 result lines, 2353 passed,
  0 failed, 28 ignored.
- `cargo doc -p vibegraph-lib --no-deps --document-private-items`: 8 warnings,
  all in files these PRs did not touch (`coupling/`, `phasespace/channel.rs`,
  `diagram_eval.rs`, `lhef/mod.rs`); 9 before F15.
  `cargo doc -p vibegraph --no-deps --document-private-items`: 0 warnings.
- `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test reweight_mg_oracle`
  (the engine's per-event ratio against MadGraph): `test result: ok. 1 passed` (at e38a38c9;
  F16 does not reach the reweighter).
- `cargo test -p vibegraph --profile release-debug --features extended-validation --test cli_reweight_proton`
  (the audit and `select_helicity` change on hadronic and MLM samples): `test result: ok. 3 passed` (at e38a38c9).
- `python3 scripts/kb.py lint`: `0 errors, 7 warnings` (all on backlog items).

Not run: `validation/validate.sh` and `pixi run validate` (CI's banked job),
seed sweeps, MadGraph.

## Found

1. **No test reaches the signed-|M|² sites of the split** (F17), for
   validation-dev. `run.rs:462` (`|Re J_x J_y*|` in the split `JAMP2`, which
   picks the event's colour flow), `run.rs:719` (MadGraph's `DABS(TS(I))` in
   helicity pruning, live in `integrate` on a signed selection) and
   `compile.rs:346` (the signed draw, unreachable while `generate` refuses a
   signed selection). Evidence: M10, M11, M12 and N12 survive every hermetic
   target. Closing it: a `JAMP2` check against MadGraph's own split `JAMP2`
   (the banked tables carry the JAMPs, not `JAMP2`), and a pruning case with a
   combination whose `T_c` is negative at every probe point.
2. **The weighted order is computed in more places than `order_weight`**:
   `helas/eval/stitching.rs:149` (`model.order_hierarchy[order] as i64 * n`,
   indexing, so a missing order panics where `order_weight` reads zero).
   `diagrams/chain.rs:341` and `diagrams/reduction.rs:213` already call
   `order_weight`. Closing it: `stitching.rs` calls `order_weight` with its
   orders map.
3. **The `JAMP2`, signed-pruning and signed-draw branches are each written
   against `is_signed()` separately** (`compile.rs:342`, `run.rs:718`,
   `engine.rs:748`, `generate.rs:458`), with `generate` refusing first: a
   decision on whether signed selections are ever generated would let three of
   them go or become tested. For the manager.

## Brief corrections

- The brief's "a second copy of the split logic between them is the likeliest
  finding" held inside #182 (F1, the two ratio paths), not between
  `run.rs` and `engine.rs`: `engine.rs` reads `cf_weights()` and `n_jamps()`
  from the evaluator rather than rebuilding the split.
- `cargo` rejects `CARGO_PROFILE_RELEASE_DEBUG_DEBUG=0` ("invalid type: Option
  value") for the `release-debug` profile and fails the whole invocation; the
  disk-saving variables in the brief must not be extended to it.
