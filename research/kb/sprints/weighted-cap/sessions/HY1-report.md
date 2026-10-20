---
type: Session Report
title: "HY1 report: hygiene review of the MadGraph WEIGHTED search loop"
description: "Review of F1 and V1: a named chain role replaces the positional bool, MadGraph's lines are cited once each, a blind spot on a chain's top-level core and on unlisted orders is pinned with mutation evidence."
status: draft
tags: [diagrams, weighted, hygiene, sprint-d, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-20}
---
Worktree `/home/user/wt/E`, branch `sprintd-weighted-cap`; fix commit `2f31cce`.
Host load 11-13 throughout (4 cores, shared).

## Findings (written before any edit)

| id | pt | site | evidence | conf. | disposition |
|---|---|---|---|---|---|
| H1 | 1/4 | `diagrams/mod.rs` `generate_from_process`, `generate_undecayed`, `chain.rs:96`, `enumerate` | a positional `bool` (`generate_from_process(process, model, false)`, `..., node.0 != 0)`) through two layers; the local `is_decay` named the opposite of what `n_in == 1 && !is_decay_chain` meant | checked | fixed |
| H2 | 1 | `ufo/weighted.rs` module doc, `order_weight`, `WeightedSearchModel` | the missing-order rule is stated in the module doc and again on `order_weight`; no MadGraph file:line appears for the cap, start, per-vertex value or the one-particle skip (`grep -n "base_objects.py:\|diagram_generation.py:" ufo/weighted.rs diagrams/mod.rs` shows none of F1's), though every other MadGraph reference in the crate cites `path:line` | checked | fixed |
| H3 | 1 | `diagrams/mod.rs` search loop doc | comment said "MadGraph tries every bound below the cap" without naming the start, step and generation lines | checked | fixed |
| H4 | 3 | `ufo/weighted.rs` `WeightedSearchModel` | `#[derive(Clone, Debug, Default, PartialEq, Eq)]`; `UFOModel` derives only `Clone`; removing `Debug, Default, PartialEq, Eq` compiles (`cargo clippy --all-targets --all-features`) | checked | fixed |
| H5 | 1 | `weighted.rs` `minimum_weighted`, `if n == 0 { 0 } else { ... }` | n = 0 gives `w = 0`, so `w + sum(rest[..kept]) <= legs_sum` for every `kept`: the branch cannot change `max(legs_sum, ...)`. An equivalent mutant: no test can reach it | checked (argued, and the five unit tests and `weighted_search` pass without it) | fixed (deleted) |
| H6 | 2 | `weighted_search.rs` `a_decay_inside_a_chain_runs_the_search` and the `node.0 != 0` | mutation: chain core flagged a chain decay (`let role = if false`) passes all 6 old tests; no test has a chain whose core is itself a one-particle process | checked (mutation run) | fixed (new test) |
| H7 | 2 | `ufo/weighted.rs` tests | `order_weight`'s missing-order rule has no test: mutations `unwrap_or(0)` -> `unwrap_or(1)` and the class filter `is_some_and` -> `is_none_or` both pass all four old unit tests and all integration tests | checked (mutation run) | fixed (new test) |
| H8 | 4 | `weighted_search.rs` `generate`/`total` | duplicate of `common::generate_with` plus a local count | checked | fixed |
| H9 | 2 | ignored SMEFTsim test | `#[ignore]` reason said what is wrong, not what un-ignores | checked | fixed |
| H10 | 4 | `diagram_count` in `tests/toy_models.rs:59`, `tests/smeftsim.rs:48` (identical bodies), `total_diagrams` in `tests/diagrams.rs:7`, `total` formerly in `weighted_search.rs` | one pattern across four test files | checked | Found |
| H11 | - | `validation/ufo/vibegraph_toy_weighted_UFO` | five vertices, four fields; QCD declared and unused with the reason in the README and `coupling_orders.py` | checked | leave (minimal) |
| H12 | 1 | `generate_undecayed` length | it was already >100 lines; the change nets +0 lines in it | checked | leave |
| H13 | 2 | the other F1/V1 tests | `a_process_without_diagrams_at_the_cap_is_refused` matches the `NoDiagrams` variant; counts are hand-derived in the doc comments, not from the input; F1's mutation table M1-M9 stands | checked | leave |

Backlog check: items naming the reviewed paths were listed with
`grep -rln "diagrams/mod.rs\|diagrams/chain.rs\|ufo/mod.rs\|ufo/weighted.rs\|weighted_search.rs" research/kb/backlog`.
None has a site this PR makes wrong, except `hygiene/weighted-order-computed-three-ways`
(see Found) and `feature/auto-weighted-search-cap-uses-hierarchy`, which this PR closes.

## Fixed (commit `2f31cce`)

- **H1** `enum ProcessRole { TopLevel, ChainDecay }` in `diagrams/mod.rs`,
  threaded in place of the bool; `chain_sets` derives it from `node.0 == 0`;
  `top_level_decay = n_in == 1 && role == TopLevel`. Identity oracle: the diagram
  sets must not move, so `validate_madgraph_diagrams` (57/57), `diagrams`,
  `decay_chain_census` and the six old `weighted_search` tests, all green.
- **H2, H3** MadGraph lines now sit once each: per-vertex value and cap on the
  `max_interaction_weighted` field doc (`base_objects.py:893-899`, `:1481-1486`,
  `diagram_generation.py:1999`), the classes (`:1432-1479`), the start
  (`:3910-3966`) on `minimum_weighted`, the cap (`:1998-1999`) on
  `maximum_weighted`, the loop (`:2013`, `:2111`, `:2115`, `:1683-1686`) in
  `generate_undecayed`, the one-particle skip (`:1978-1980`) and the chain flag
  (`:1381-1382`) at `generate_from_process` and the enum. The missing-order rule
  and its MadGraph `KeyError` (`:897`, `:1448`) are on `order_weight` alone, for
  `weighted-order-computed-three-ways` to adopt.
- **H4, H5, H8, H9** as in the table.
- **H6** `a_chain_whose_core_is_a_decay_runs_no_search_on_the_core`:
  `z > t t~ u u~` has 13 diagrams unbounded and 4 at `WEIGHTED<=4`; the chain
  `z > t t~ u u~, t > b w+, t~ > b~ w-` must keep 13.
- **H7** `an_unlisted_order_weighs_nothing_and_joins_no_class`.

Suite count: 1548 -> 1550 passed (the two tests above, no others added or
deleted).

## Mutations

Driver `scratchpad/wcapE-HY1-scripts/mutate.py` (apply, run
`cargo test -p vibegraph-lib --lib ufo::weighted` and `--test weighted_search`,
restore, `touch`); log `scratchpad/wcapE-HY1-mut.log`.

| Mutation | old tests | new test |
|---|---|---|
| R1 chain core flagged `ChainDecay` (`if node.0 == 0` -> `if false`) | all 6 `weighted_search` + 4 unit pass | `a_chain_whose_core_is_a_decay_runs_no_search_on_the_core` FAILS |
| R2 unlisted order weighs 1 (`unwrap_or(0)` -> `unwrap_or(1)`) | 4 unit + 7 integration pass | `an_unlisted_order_weighs_nothing_and_joins_no_class` FAILS |
| R3 unlisted order joins classes (`is_some_and` -> `is_none_or`) | 4 unit + 7 integration pass | same test FAILS |
| R4 top-level decays searched (`top_level_decay = false`) | `a_top_level_decay_runs_no_weighted_search` fails (F1's M7) | that and the new chain test FAIL |

No mutation reaches the deleted `n == 0` branch (H5: it is an equivalent
mutant). No test reaches `minimum_weighted` with a required s-channel naming a
particle in no class (the case the branch handled); the result there equals the
legs' sum either way.

## Routed

None.

## Gate

All run in `/home/user/wt/E` with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`; log `scratchpad/wcapE-HY1-gate.log`.

```
cargo fmt --all --check                                    exit 0
cargo clippy --workspace --all-targets -- -D warnings      Finished `dev` profile ... exit 0
cargo clippy --workspace --all-targets --all-features -- -D warnings   Finished ... exit 0
cargo doc -p vibegraph-lib --no-deps --document-private-items
    8 warnings, none in a file this change touches (configs.rs, channel.rs, alphas.rs, scales.rs, parameters.rs, lhef/mod.rs); exit 0
cargo test --workspace      0 FAILED; test results summed: 1550 passed, 0 failed, 19 ignored
    lib: 1014 passed, 12 ignored; validate_madgraph_diagrams: 57 passed; diagrams: 20 passed;
    weighted_search: 7 passed, 1 ignored
python3 scripts/kb.py lint  834 concepts, 291 backlog items; 0 errors, 4 warnings
```

`validate_madgraph_diagrams` (57/57) is the banked target this change's search
touches; it ran inside `cargo test --workspace` against the bundled reference
rows. The `extended-validation` feature targets (MadGraph amplitude gates) were
not run: nothing here changes a diagram set, amplitude, colour or coupling. The
"19 ignored" is my sum over all binaries; F1's report says 18, the difference
is not from a test I added (I added no `#[ignore]`), and I did not chase it.

## Method

About 40% reading the diff against the checklist (history-word grep, derive and
`pub` narrowing by compiling without them, the repeated-helper grep across
`tests/`), 30% mutation runs of each test I judged weak (R1-R3 found two
real blind spots: the chain's root core and the unlisted-order rule), 20% the
equivalent-mutant argument that found the dead `n == 0` branch (found by asking
what mutation would reach it, not by running one), 10% scratch probing for a
chain whose root core has diagrams at more than one order (`z > t t~ u u~`;
`z > t t~ d d~` was a false start, its counts coincide). The history-word grep
(`T[0-9]|used to|no longer|sprint|F1`) over the diff's code found nothing.

## Found

1. **Test helper duplicated across four files.** `diagram_count(model, process)`
   in `vibegraph-lib/tests/toy_models.rs:59` and `tests/smeftsim.rs:48` are
   identical; `total_diagrams` in `tests/diagrams.rs:7` is the same sum. Closing
   it: one `common::diagram_count` (and a `common::try_generate` returning the
   `Result`, the refusal-carrying form `weighted_search.rs` now keeps locally).
2. **`weighted-order-computed-three-ways` is now four ways, and stale.** Its
   title and `closes_when` name three sites; `ufo::weighted::order_weight` is a
   fourth with the missing-order rule 0 (the doc on it is the single statement
   to adopt). Closing it should collapse all four, and the item should be
   updated by the manager.
3. **`feature/auto-weighted-search-cap-uses-hierarchy`** is closed by this PR
   (cap, start, 1->n rule) except the final-state-gluon trial (the ignored test
   and V1's Found); the manager decides whether it is deleted or narrowed.

## Brief corrections

None. The brief's check that "the toy UFO is minimal" holds; the unused QCD
order is documented in `validation/ufo/README.md` and `coupling_orders.py`.
