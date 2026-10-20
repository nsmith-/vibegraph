---
type: Session Report
title: "F1 report: the automatic WEIGHTED search is MadGraph's loop"
description: "The search starts at MadGraph's get_minimum_WEIGHTED, steps by one, stops at (n_ext-2) x int(max interaction WEIGHTED per equivalent cubic vertex) and generates at the bound it stopped at; a top-level 1->n process runs no search. A QED+NP toy finds its diagrams; the SM cap does not move (the brief's x4 was wrong); the SM search got cheaper, not dearer."
status: draft
tags: [diagrams, coupling-orders, weighted, madgraph-parity, sprint-d, report]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-20}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: d6ca65a, resource: "https://github.com/nsmith-/vibegraph/commit/d6ca65a", title: "diagrams: run the automatic WEIGHTED search as MadGraph runs it"}
---
Host: the 4-core cloud container, shared with sibling worktrees (load average
13-19 throughout). Worktree `/home/user/wt/E`, branch `sprintd-weighted-cap`,
verified before any action. MadGraph read at the pinned commit
`b7687064b9a013317ca164aa1395bc9c0e39ae1e` (raw.githubusercontent; the
submodule is empty here).

## MadGraph's loop, element by element

All `diagram_generation.py` lines are `MultiProcess.find_optimal_process_orders`
(:1928-2115) unless named; `base_objects.py` as named.

| Element | MadGraph | vibegraph before | vibegraph now |
|---|---|---|---|
| When it runs | skipped if the process has `orders`, `overall_orders` or NLO virt (:1972-1975), **and for a one-particle process that is not a decay-chain decay** (:1978-1980: `get_ninitial() == 1 and not is_decay_chain` returns the empty orders: no WEIGHTED bound at all). Decays inside a chain are flagged `is_decay_chain` (:1381-1382) and do run it | ran for every process with no orders, including a top-level `z > u u~ d d~` | skipped for a top-level 1->n; still runs for chain decays and for the core of a 2->n chain |
| Start | `ProcessDefinition.get_minimum_WEIGHTED` (:1986-1987; `base_objects.py:3910-3966`): each leg takes the value of the first particle hierarchy class (`Model.get_particles_hierarchy`, `base_objects.py:1432-1479`) holding any of its ids; drop the two smallest, sum; with required s-channels, `max(sum, 2*value(s-channel ids) + sum(rest[:-2k]))` over the lightest alternative | `(n_ext-2) * min(hierarchy)` | MadGraph's, `WeightedSearchModel::minimum_weighted` |
| Step | `+1` (:2111) | `+1` | `+1` |
| Cap | `(len(legs)-2) * int(model.get_max_WEIGHTED())` (:1998-1999), loop `while max_order_now < cap` (:2013) | `(n_ext-2) * max(hierarchy)`, inclusive | MadGraph's, `WeightedSearchModel::maximum_weighted` |
| Interaction WEIGHTED | `Interaction.get_WEIGHTED_order` (`base_objects.py:893-899`): `float(sum(hierarchy[k]*orders[k])) / max(n_particles-2, 1)`, i.e. **per equivalent three-point vertex**; `Model.get_max_WEIGHTED` (:1481-1486) takes the max; `int()` at :1999 truncates | (not used) | the same, as integer division (floor commutes with max); computed once per model in `ParsedModel::into_model` after restriction |
| Order missing from the hierarchy | `KeyError` at `base_objects.py:897` (and at :1448 for the classes). A UFO with `all_orders` lists every order it declares (`import_ufo.py:650-660`); a model without `coupling_orders.py` gets all-1 with QED=2 only if its orders are exactly {QCD, QED} (`base_objects.py:1379-1386`) | the search's diagram filter ignores it | weighs 0 and joins no class, as the filter ignores it (stated in `ufo/weighted.rs`) |
| Cap reached, nothing found | returns `{WEIGHTED: max_order_now}` (:2115), i.e. `max(start, cap)`, and the whole multiprocess is then generated at that bound (:1683-1686); if that is empty, `NoDiagramException` (:1907-1912) | tried the cap itself inside the loop and returned whatever it gave; empty -> `DiagramError::NoDiagrams` | generates at `max(start, cap)` after the loop; empty -> `NoDiagrams` (unchanged) |
| Trial per subprocess | returns on the first concrete subprocess with a diagram (:2101-2107) | any set non-empty | unchanged (equivalent) |
| Crossing skip | skips a subprocess whose crossing already failed at this bound (:2090-2095) | none | none: an optimisation with the same outcome |
| **Final-state gluon removal** | when QCD is the lowest class, each trial drops the final-state gluons and lowers the bound by `nglue * hierarchy['QCD']` (:2040-2059) | not done | **not done** -- see Found |

Where vibegraph differed: the start, the cap's quantity (largest hierarchy, not
largest interaction WEIGHTED), the cap's inclusivity (cosmetic: it tried the cap
inside the loop, MadGraph generates there after it), and the 1->n rule.

### The brief's SM claim is wrong

The brief says the SM cap rises from `(n-2)*2` to `(n-2)*4` because a QED=2
quartic weighs 4. MadGraph divides by `n_particles - 2`, so the SM's QED=2
quartics (WWWW, WWZZ, WWAA, WWZA) weigh 2 and `gggg` (QCD=2) weighs 1: the SM's
largest interaction WEIGHTED is a cubic QED vertex's, 2, and **the SM cap does
not move**. Pinned by `ufo::weighted::tests::the_sm_cap_is_a_cubic_qed_vertex`
(which also asserts a QED=2 four-point vertex exists, so the pin is not vacuous).
What the SM does see is the start: MadGraph's estimate is never below
`(n-2)*min(hierarchy)`, so the SM now makes fewer trials (cost section).

## What landed

- `vibegraph-lib/src/ufo/weighted.rs` (new): `order_weight` (one coupling-order
  tuple's WEIGHTED, a missing order weighs 0); `WeightedSearchModel` with
  `max_interaction_weighted` and `particle_classes`, built from the restricted
  interactions; `minimum_weighted` (MadGraph's start, including the required
  s-channel term and Python's `rest[:-0]` slice) and `maximum_weighted`.
- `UFOModel.weighted_search` (`ufo/mod.rs`), computed once in `into_model`.
- `generate_undecayed` (`diagrams/mod.rs`): MadGraph's loop; the
  `is_decay_chain` flag threaded through `generate_from_process`, false from
  `enumerate`, `node.0 != 0` from `chain_sets` (`diagrams/chain.rs`).
- `validation/ufo/vibegraph_toy_weighted_UFO` (new, authored; documented in
  `validation/ufo/README.md`): hierarchies QED=2, NP=1 (QCD=1 declared, unused,
  because MadGraph reads `order_hierarchy['QCD']` at :2059). Vertices
  `fw~ fw vw` (QED, W 2), `fw~ fw xw` (QED+NP, W 3), `xw^4` (QED, 2/2 = 1),
  `xw xw yw` (QED, W 2), `yw^4` (QED=2 NP=3, 7/2 = 3.5). int(max) = 3.
- `vibegraph-lib/tests/weighted_search.rs` (new) and four unit tests in
  `ufo/weighted.rs`.

**The fourth WEIGHTED computation.** The cap needs a per-interaction WEIGHTED
normalised per cubic vertex; none of the three existing ones
(`diagrams/mod.rs`'s feyngraph filter closure, `chain.rs::within_orders`,
`stitching.rs::weighted`) computes it or sits where the model is built.
`ufo::weighted::order_weight` is therefore a fourth site. It states its
missing-order rule (0) and is the natural single helper for
`weighted-order-computed-three-ways` to collapse the other three onto; that item
is not absorbed here.

## Tests and hand counts

Each count is derived in the test's doc comment; summary:

| Test | Process | Count | MadGraph's path | Before |
|---|---|---|---|---|
| `a_two_order_vertex_is_found_above_the_largest_hierarchy` | `fw~ fw > xw xw` | 2 (t, u fermion exchange; W 6) | start 4, tries 4, 5, leaves at cap 6, generates there | `NoDiagrams` (cap 4) |
| `a_two_order_vertex_is_found_by_the_search_below_the_cap` | `fw~ fw > xw xw vw` | 6 (3! emission orders on the fermion line; W 8) | start 6, tries 6, 7, finds 8 < cap 9 | `NoDiagrams` (cap 6) |
| `the_search_starts_where_madgraph_starts` | `xw xw > xw xw` | 4 (contact W 2 + s/t/u `yw` W 4) | start 4: first trial holds all four | 1 (start 2: the contact alone) |
| `a_process_without_diagrams_at_the_cap_is_refused` | `vw vw > yw` | `NoDiagrams` | `NoDiagramException` | same (regression guard) |
| `a_top_level_decay_runs_no_weighted_search` | SM `z > u u~ d d~` | = unbounded explicit orders (17), > `WEIGHTED<=4` (4) | no search: all orders | 4 |
| `a_decay_inside_a_chain_runs_the_search` | SM `e+ e- > z h, z > u u~ d d~` | 4 (1 core x 4) | decay searched, stops at 4 | 4 (same) |

Before-run (the new tests against the unchanged library, command
`cargo test -p vibegraph-lib --test weighted_search`):

```
test a_process_without_diagrams_at_the_cap_is_refused ... ok
test a_decay_inside_a_chain_runs_the_search ... ok
test a_two_order_vertex_is_found_above_the_largest_hierarchy ... FAILED
test the_search_starts_where_madgraph_starts ... FAILED
test a_two_order_vertex_is_found_by_the_search_below_the_cap ... FAILED
test a_top_level_decay_runs_no_weighted_search ... FAILED
  diagrams at WEIGHTED 6: NoDiagrams { process: "fw~ fw > xw xw" }
  diagrams at WEIGHTED 8: NoDiagrams { process: "fw~ fw > xw xw vw" }
  the_search_starts_where_madgraph_starts: left: 1 right: 4
  a_top_level_decay_runs_no_weighted_search: left: 4 right: 17
test result: FAILED. 2 passed; 4 failed
```

The SM census is unmoved: `validate_madgraph_diagrams` 57 passed, 0 failed,
with its two pre-existing informational rows printing what they printed before
(`gg_to_gg`-type 4 vs 6 and SMEFTsim 21 vs 27 topologies, both four-gluon-contact
counting conventions); `diagrams.rs` 20 passed; `decay_chain_census` 1 passed;
`toy_models` 6 passed.

## Mutation evidence

Driver: `scratchpad/wcapE-F1-scripts/mutate.py` -- for each mutation, apply,
run `cargo test -p vibegraph-lib --lib ufo::weighted` and
`cargo test -p vibegraph-lib --test weighted_search`, restore, `touch`. (The
sweep was killed mid-M5 by the 30-minute background limit; M5 was restored by
hand, verified with `git diff`, and the sweep resumed from M5.)

| Mutation | Fails | Reads |
|---|---|---|
| M1 no `/ (n_particles-2)` | `the_sm_cap_is_a_cubic_qed_vertex`, `the_toy_cap_truncates_...` | SM 4 vs 2; toy 7 vs 3 |
| M2 round up (`div_ceil`) | `the_toy_cap_truncates_...` | 4 vs 3 |
| M3 heaviest s-channel alternative (`max_by_key`) | `the_start_sums_the_largest_legs_and_the_lightest_s_channel` | 4 vs 2 |
| M4 old start `(n-2)*min(hierarchy)` | `the_search_starts_where_madgraph_starts` | 1 vs 4 |
| M5 old cap `(n-2)*max(hierarchy)` | `a_two_order_vertex_is_found_above_the_largest_hierarchy`, `..._below_the_cap` | `NoDiagrams` |
| M6 no generation at `max(start, cap)` | `a_two_order_vertex_is_found_above_the_largest_hierarchy`, `a_decay_inside_a_chain_runs_the_search` | the second because `e+ e- > z h` starts at its cap (4 = 4) and is generated only there |
| M7 top-level decays searched | `a_top_level_decay_runs_no_weighted_search` | 4 vs 17 |
| M8 chain decays not searched | `a_decay_inside_a_chain_runs_the_search` | 17 vs 4 |
| M9 an all-empty process accepted (`enumerate`) | `a_process_without_diagrams_at_the_cap_is_refused` | -- |

`a_particle_takes_the_lowest_class_it_interacts_in` is the classes' direct pin;
M4 (which bypasses the classes entirely) is what shows the classes reach the
search.

## Search cost

The SM cap did not move, so no SM process searches further; the start rose,
so SM processes search less. Wall time of `generate_from_proc_card` (test
profile, optimised), five repetitions each, three runs alternated
after/before/after back to back on the loaded host (load 13.6-14.8):

| Process | before (min / median / max, s) | after-1 | after-2 |
|---|---|---|---|
| `g g > a a a a a a` (8 legs, no diagrams; the largest multiplicity the tests use is 8) | 3.23 / 3.85 / 4.09 | 1.99 / 2.20 / 2.30 | 2.37 / 2.59 / 2.77 |
| `g g > a a a a a` (7, none) | 0.205 / 0.227 / 0.303 | 0.116 / 0.117 / 0.123 | 0.113 / 0.147 / 0.179 |
| `g g > a a a a` (6, none) | 0.0099 / 0.0112 / 0.0143 | 0.0072 / 0.0076 / 0.0081 | 0.0069 / 0.0104 / 0.0169 |
| `p p > e+ e- j j` (6, 736 diagrams) | 0.727 / 0.819 / 0.890 | 0.245 / 0.265 / 0.343 | 0.251 / 0.303 / 0.341 |
| `fw~ fw > xw xw vw` (toy, 5) | 0.0012 / 0.0013 / 0.0109 (`NoDiagrams`) | 0.0018 / 0.0054 / 0.0109 (6 diagrams) | 0.0013 / 0.0019 / 0.0101 |

An earlier before-run of the same harness (load ~14) read 3.05 / 3.69 / 3.78 s
for the 8-leg process and 0.716 / 0.741 / 0.778 s for `p p > e+ e- j j`. The
8-leg no-diagram process made seven trials (WEIGHTED 6..12) and now makes one
generation at 12 (start = cap = 12); `p p > e+ e- j j` made three (4, 5, 6) and
now one (start 6). What remains of the 8-leg time is the topology generation,
paid once. The toy process costs three generations now (6 and 7 empty, 8 finds)
where it cost four before (3 to 6, all empty) and failed.

## Gate

Final run on the committed tree (the scratch timing harness deleted first),
commands as below with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`:

```
$ cargo fmt --all --check
(no output)  fmt exit=0

$ cargo clippy --workspace --all-targets -- -D warnings
Finished `dev` profile [optimized] target(s) in 46.48s
exit=0

$ cargo test --workspace
41 test binaries: 1548 passed, 0 failed, 18 ignored
  lib (vibegraph-lib):           test result: ok. 1013 passed; 0 failed; 12 ignored; 0 measured; 0 filtered out; finished in 101.48s
  validate_madgraph_diagrams:    test result: ok. 57 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.32s
  diagrams:                      test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
  decay_chain_census:            test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
  toy_models:                    test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
  weighted_search:               test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
  (last binary, its tail):       test result: ok. 1 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.48s
exit=0

$ python3 scripts/kb.py lint      (after `python3 scripts/kb.py index`)
831 concepts, 291 backlog items; 0 errors, 4 warnings
```

## Banking request (M3, MadGraph 3.7.1)

What it would give: MadGraph's own diagram counts and its own search trace on
the toy model, so the hand counts above become a census row.

1. Model: `validation/ufo/vibegraph_toy_weighted_UFO` (repo-relative;
   `build.sh` stages model directories under `output/models/` as for the other
   toy rows). No restrict card.
2. Three scripts under `validation/madgraph/scripts/`, one row each, no
   `launch` needed (counts only):
   ```
   import model validation/ufo/vibegraph_toy_weighted_UFO
   display interactions
   generate fw~ fw > xw xw
   output ffx_to_xwxw_toy_weighted -nojpeg
   ```
   and likewise `generate fw~ fw > xw xw vw` (`ffx_to_xwxwvw_toy_weighted`),
   `generate xw xw > xw xw` (`xwxw_to_xwxw_toy_weighted`). Optionally a fourth,
   SM: `generate z > u u~ d d~` (`z_to_uuxddx_decay`) for the 1->n rule.
3. Extract: `NGRAPHS` per P-directory into `validation/madgraph/diagrams.json`
   the way `extract_diagrams.py` does for every row (expected 2, 6, 4, and 17
   for the SM decay); and from the build log the `Trying coupling order
   WEIGHTED<=N` lines (`diagram_generation.py:2014`), which should read 4, 5 /
   6, 7, 8 / 4 / none -- that trace is the direct evidence for start, step and
   cap, and is worth keeping in the row's note. `display interactions` should
   list 5 interactions.
4. Manifest: one `[[process]]` row each, `model =
   "validation/ufo/vibegraph_toy_weighted_UFO"`, `categories.diagrams = { tier =
   "hermetic", mode = "gate" }`. The test that reads it is
   `vibegraph-lib/tests/validate_madgraph_diagrams.rs` (one trial per
   `diagrams.json` row, model import checked against the manifest's
   `model`).

## What remains for the item

`closes_when` asks for the cap (done) and a census test finding MadGraph's
diagrams on a card whose lowest order lies above the old cap. The hermetic
hand-checked counts are here; MadGraph's own counts need the banking run above.
Whether to close before it is the user's call (sprint note). The CMS cards
(060, 067, 050, 071) themselves were not run: they live on a CERN host.

## Found

1. **MadGraph's final-state gluon removal in the search is not reproduced.**
   `diagram_generation.py:2040-2059`: when the gluon is in the lowest hierarchy
   class, each trial removes final-state gluons and lowers the bound by
   `nglue * hierarchy['QCD']`. Where the reduced process has no diagrams at all
   (a final state of gluons only, e.g. `u u~ > g g`, or one the gluons are
   needed to produce), MadGraph finds nothing below the cap and generates at the
   cap, so it keeps every diagram up to `(n-2) * max interaction WEIGHTED`.
   In the SM that is harmless (an all-gluon final state from quarks is pure QCD
   at tree level), but in a model where gluons couple at another order too
   (HEFT's `HIG`, SMEFT's `O_G`) the two sides can keep different orders.
   Closing it: either reproduce the reduced-process trial, or show by a
   machine-checked census on such a model that the found order agrees.
2. **Default hierarchy without `coupling_orders.py` differs.** MadGraph gives
   every order 1, and QED 2 only when the orders are exactly {QCD, QED}
   (`base_objects.py:1379-1386`); vibegraph falls back to the SM's QCD=1, QED=2
   regardless, so an NP order in such a model weighs 0 here and 1 there.
   Closing it: make `ParsedModel::parse`'s fallback MadGraph's rule, with a
   test on a model directory lacking the file.
3. **An interaction order missing from the hierarchy.** MadGraph raises
   `KeyError` (`base_objects.py:897`, `:1448`); here it weighs 0 in the search's
   cap, classes and filter. Belongs with `weighted-order-computed-three-ways`,
   which should also adopt `ufo::weighted::order_weight` as its one helper
   (it is now the fourth site).
