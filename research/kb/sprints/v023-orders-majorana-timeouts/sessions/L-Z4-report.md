---
type: Session Report
title: "L-Z4 report: hygiene pass on the multichannel transfer bound and the equal-weight recombination script"
description: "Review of PR 5 (#184: single_leg_transfer_bound, with_side_transfer_bounds and the maps.rs wiring) and PR 4's ew_recombine.py. Of thirteen mutations of PR 5's code, four survived its tests and two were caught only by a distant one; all now fail a named test. The script crashed on three empty-input shapes and had no tests; it now refuses by name and has ten."
status: draft
tags: [hygiene, multichannel, spine, cuts, validation, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-24}
---
Branch `sprintl-hyg-phasespace` off main 8b3af5ab. Source of every claim below
is a command shown with it; paths are relative to the worktree
`/home/user/wt/sprintl-hyg-phasespace`.

The findings table below was committed (ca9c7381) before any code edit; the
sections after it record the fixes, mutations, oracle and gates.

## Findings (before any edit)

Scope: `git show 8b3af5ab -- vibegraph-lib/src` (`cuts.rs`, `phasespace/diagram_channel.rs`,
`phasespace/maps.rs`) and `git show 2ac778e2 -- validation/madgraph/ew_recombine.py`.
Main equals the branch base: `git diff 8b3af5ab HEAD --stat` lists only the brief.

### Mutation batch (point 2)

`/home/user/wt/lz4-scripts/mutate.sh` applies one one-line mutation of PR 5's
code, runs `cargo test -p vibegraph-lib --lib -- cuts:: phasespace:: proton::`
(291 selected tests, 4 ignored), reverts, logs. Log:
`/home/user/wt/logs/lz4-mut1.log`.

| id | file, mutation | result | caught by |
|---|---|---|---|
| M1 | cuts.rs `Some(pt * pt)` → `Some(pt)` | caught | 4 tests, incl. `a_single_leg_side_bounds_the_transfer_at_its_own_threshold` |
| M2 | cuts.rs pairing `find(p.exempt == products)` → `find(true)` | caught | `subprocesses_whose_decays_carry_different_legs_weight_one_shared_filter` |
| M3 | cuts.rs weakest of filters `min` → `max` | caught | the same test |
| M4 | cuts.rs `count_ones() != 1` → `== 0` | caught | `a_single_leg_side_bounds_the_transfer_at_its_own_threshold` |
| M5 | diagram_channel.rs larger proven square `b > a` → `b < a` | caught, but only by `proton::tests::a_polarized_group_is_evaluated_in_the_partonic_centre_of_mass` | none of PR 5's own diagram_channel tests |
| M6 | diagram_channel.rs `min(proven, -cap)` flipped | caught | both new diagram_channel tests |
| M7 | diagram_channel.rs `limit > 0` → `limit >= 0` | caught | `a_side_bound_drops_only_what_no_single_leg_cut_proves` |
| M8 | diagram_channel.rs `side \|= node_mask(..)` → `side = node_mask(..)` (the emitted side not accumulated across rungs) | **survived** (287 passed, 0 failed) | nothing |
| M9 | diagram_channel.rs `bound(full & !side)` → `bound(full)` (the far side) | caught only by the same `proton::tests::a_polarized_group…` | none of PR 5's own diagram_channel tests |
| M10 | diagram_channel.rs drop the emitted side's bound | caught | both new diagram_channel tests, plus one more |
| M11 | maps.rs `UnionBounds::single_leg_transfer_bound` `min` → `max` | **survived** | nothing |
| M12 | maps.rs `products = forced.fold(..)` → `0` | **survived** the lib filter; its guard is the CLI test named in the brief, run separately below | |
| M13 | maps.rs delete the `with_side_transfer_bounds` call from `MapChoices::channel` | **survived** the lib filter | nothing hermetic in the lib |

### Findings

| id | point | site | evidence | confidence | disposition |
|---|---|---|---|---|---|
| Z4-1 | 2 | `diagram_channel.rs` tests `side_bounded_caps`, `a_side_bound_drops_only_what_no_single_leg_cut_proves` | M8 survives: every case gives the second rung's single leg (`leg2`) a threshold of 100 or more, equal to or above the rung's own scale (−100), so reading the wrong side there leaves the same caps. M9 reaches only a distant proton test for the same reason. Mutation the test must catch: `bound(full & !side)` → `bound(full)` or `side \|=` → `side =`. | checked (M8, M9 runs) | fix: add a case with `leg2` below the scale, `(100, 25)` and `(0, 25)` |
| Z4-2 | 2 | `diagram_channel.rs` same tests | M5 survives PR 5's own tests: no case gives a rung two single-leg sides, so the "larger proven square" is never chosen between two numbers (`reduce` over a one-element iterator). Mutation: `b > a` → `b < a`. | checked (M5) | fix: a two-leg final state, one rung, both sides single legs |
| Z4-3 | 2 | `maps.rs:252` `UnionBounds::single_leg_transfer_bound` | M11 survives; `grep -rn UnionBounds vibegraph-lib/src/phasespace/maps.rs` shows no test constructing a `UnionBounds` for this method. The weakest-of-filters rule (and `None` if any filter proves nothing) is the contract of the hadronic run's union of filters. | checked (M11) | fix: unit test over two compiled filters |
| Z4-4 | 2 | `maps.rs:376` wiring in `MapChoices::channel` | M13 survives: deleting the call leaves every lib test green. No test shows a built channel differs from one without the bound. | checked (M13) | fix: unit test through `MapChoices::channel` on a process where a rung's single leg is uncut |
| Z4-5 | 2 | `maps.rs:375` `products` fold | M12 survives the lib filter. It is reachable only under pairing-weighted cuts, whose guard the brief names (`cli_reweight_proton`). | suspected: settle by running M12 against `cargo test -p vibegraph --test cli_reweight_proton` | settle, then report |
| Z4-6 | 1 | `validation/madgraph/ew_recombine.py:36,45-46,73` | `python3 validation/madgraph/ew_recombine.py validation/madgraph/output/pp_to_llj_mlm` ends `ZeroDivisionError: float division by zero` (a run directory with no channel `results.dat`); an empty channel file gives `IndexError: list index out of range` at `rows[0][1]`; a total `results.dat` with one field gives `IndexError` at `fortran_float(total[1])`. The script reads a MadGraph run directory the caller names, so these are fallible-path panics where a named refusal belongs. | checked (all three reproduced, outputs in the fix section) | fix |
| Z4-7 | 1 | same, `glob.glob(...)` | The channel loop sums floats in `glob` order, which is filesystem order; a path with `[`, `*` or `?` also matches the wrong files. | checked by reading; not a measured error at the 6 significant digits it prints | fix: `sorted(glob.glob(glob.escape(run) + ...))` |
| Z4-8 | 2 | same | The script has no test; no `test_*.py` beside it, no pixi task. The sibling `validation/cms_gridpacks/test_*.py` has the `python -m unittest discover` pattern (`pixi.toml:434`). Mutations the test must catch: use the first iteration only, divide the error by `n` once, drop the tag filter, accept 3-field rows. | checked | fix: `validation/madgraph/test_ew_recombine.py`, run by a pixi task |
| Z4-9 | 4 | `diagram_channel.rs` tests (~4510-4600) | The ladder `from_topology_ladder(600.0, [0.0, 0.0], vec![0.0, 0.0, 0.0, 4.7], &[(vec![3], None, 4.7), (vec![0, 1], None, 0.0)], (vec![2], None)).with_fiducial_t_max(-100.0)` is spelled three times in the two new tests and their helper. | checked (`grep -n "from_topology_ladder" vibegraph-lib/src/phasespace/diagram_channel.rs`) | fix: one fixture function; identity: the existing assertions, incl. the map-key one |
| Z4-10 | 1 | `cuts.rs:2078`, `maps.rs:314-322` | The sentence added to `spacelike_floor`'s doc runs to 105 columns on its own line and the `MapChoices::channel` doc leaves a four-word line ("The one place a channel") where the added clause was inserted. Docs are correct, only mis-wrapped. | checked | fix: reflow |
| Z4-11 | 4 | `cuts.rs:2247` and `maps.rs:255` | The same reduction, `reduce(\|a, b\| Some(a?.min(b?)))` over the filters' `Option<f64>` bounds, is written in two modules. | checked | **Found** (across modules; both named) |
| Z4-12 | 3 | `diagram_channel.rs:983` `pub fn with_side_transfer_bounds` | Its only caller is `maps.rs` in the same crate, but every sibling builder (`with_timelike_floors`, `with_fiducial_t_max`, `with_forced_windows`) is `pub` the same way, and `with_fiducial_t_max` has integration-test users. A new `pub` with a user outside its module (`maps.rs`) passes the rule. | checked | leave |
| Z4-13 | 1 | the three source files' added lines | `git show 8b3af5ab -- vibegraph-lib/src \| grep '^+' \| grep -nE '\bT[0-9]\b\|\bC[0-9]\b\|Phase [0-9]\|note [0-9]+\|used to\|no longer\|sprint\|session\|backlog\|previously\|the old'` returns nothing. | checked | not a finding |

Backlog check. Items naming a path under review: `hygiene/cut-threshold-rule-spelled-inline`
(cuts.rs: the default-off rule; `single_leg_transfer_bound` reads `pt_min_of`, which is
that rule's reader, not a new copy), `performance/spacelike-floor-too-loose`,
`performance/spacelike-floor-largest-threshold-inflates-seed-spread`,
`performance/ht-cuts-give-channels-no-regulator` (cuts.rs `spacelike_floor`: performance
topics about the scale this PR's bound lowers per rung, none a site of this change),
`hygiene/kinematic-helpers-hand-rolled` (diagram_channel.rs boost; not touched by the
change), `performance/sampler-tunings-predate-pooled-stop` (maps.rs `resolve`; not touched).
None is a finding here.

Z4-5 settled: M12 (`products` fold → `0`) is caught by
`cargo test -p vibegraph --features extended-validation --test cli_reweight_proton`
(`reweighting_a_pairing_weighted_card_is_refused ... FAILED`, 2 passed 1 failed;
`/home/user/wt/logs/lz4-m12.log`), a target that needs `extended-validation` (reference
data), so no hermetic test reaches that line. Z4-5 is *checked, guarded by the banked
target only*.

## Fixed

Code commit: see the git log of this branch (commit after ca9c7381).

| id | what changed | evidence |
|---|---|---|
| Z4-1, Z4-2 | `diagram_channel.rs`: new tests `the_second_rung_reads_the_single_leg_across_its_line` (cases `(100, 25)`, `(100, 0)`, `(0, 0)`) and `a_line_between_two_single_legs_takes_the_larger_square` (a two-leg final state, both sides single legs) | mutation table |
| Z4-3 | `maps.rs`: `the_union_of_filters_bounds_a_single_leg_by_its_weakest` (default, soft and lepton-free filters; one filter equals that filter bit for bit) | M11 |
| Z4-4 | `maps.rs`: `a_built_channel_carries_the_transfer_bounds_the_cuts_prove` builds every `g g > b b~ g` channel under the real cuts and under a wrapper that proves nothing; some but not all map keys differ | M13 |
| Z4-6, Z4-7, Z4-8 | `ew_recombine.py`: `RunError`, three named refusals plus a non-numeric field, exit status 1, `sorted(glob.glob(glob.escape(run)...))`, a `main(argv)`; new `validation/madgraph/test_ew_recombine.py` (10 tests) and pixi task `test-ew-recombine` in the madgraph feature | python mutation table |
| Z4-9 | `diagram_channel.rs`: one `two_rung_chain()` fixture and a `rung_caps` helper replace three spellings; the existing assertions, map-key equality included, are unchanged and pass | the tests pass |
| Z4-10 | `cuts.rs`, `maps.rs`: two doc paragraphs reflowed | `cargo fmt --check` clean; text unchanged |

## Mutations (Rust, `cargo test -p vibegraph-lib --lib -- cuts:: phasespace:: proton::`)

Old = PR 5's tests at base (`/home/user/wt/logs/lz4-mut1.log`); new = after the fixes
(`/home/user/wt/logs/lz4-mut2.log`). Driver `/home/user/wt/lz4-scripts/mutate.sh`
(applies, runs, restores from a copy on every exit path).

| mutation | old tests | new tests |
|---|---|---|
| M5 larger square `b > a` → `b < a` | 1 failure, `proton::…partonic_centre_of_mass` only | also `a_line_between_two_single_legs_takes_the_larger_square` |
| M8 emitted side not accumulated | **survived** (287 passed) | `the_second_rung_reads_the_single_leg_across_its_line` fails |
| M9 far side read as `full` | `proton::…` only | also `the_second_rung…` and `a_line_between…` |
| M11 `UnionBounds` `min` → `max` | **survived** | `the_union_of_filters_bounds_a_single_leg_by_its_weakest` fails |
| M13 wiring call deleted | **survived** | `a_built_channel_carries_the_transfer_bounds_the_cuts_prove` fails |
| M12 `products` → 0 | survived the lib filter | caught by `cli_reweight_proton` (extended-validation) only |
| M1–M4, M6, M7, M10 | caught by PR 5's own tests | unchanged |

Python mutations (`/home/user/wt/lz4-scripts/pymut.sh`, log `/home/user/wt/logs/lz4-pymut.log`):
the script as published has no test, so every one survives it. With the new tests, 15
mutations (first-iteration-only mean, error divided once, tag filter dropped, 3-field rows
accepted, empty-file and short-total checks removed, exit status forced to 0, `D` exponent
dropped, no `glob.escape`, ...) are all caught; the run before `glob.escape` was tested had
P15 survive and a path-with-`[` test was added.

## Identity oracle

Base binary: this tree at HEAD fde61c2b built before any edit (`git diff 8b3af5ab HEAD
--stat` lists only the brief, so the source is main 8b3af5ab); new binary built from the
final tree. `vibegraph integrate <proc_card> --run-card <run_card> --fixed-budget --neval
20000 --niter 3 --seed 7 -j 2 --no-network --force --output-ldjson` (`oracle.sh`), cards
`pp_to_llj_mlm` (the banked run card, `p p > e+ e- j`) and `pp_to_ttx_0j1j_mlm` (the L-P1
card pair).

| row | σ, base and new (identical) | grid.bin.zst sha256, base and new (identical) |
|---|---|---|
| llj | 266.9758473984698 ± 2.529226338807175 (`0x1.0af9d12296508p+8`) | c700745387ad09cf8580a933aca3a47cad25e9f6746708e0c45d1d286c34bb13 |
| ttx 0j1j | 1085.3542015092012 ± 6.28037355746103 (`0x1.0f56ab3cce8dap+10`) | 01bf2e2d87976131ce3f41f0a86ba0f234a9178e41511eb62fbbbf1121223994 |

`ew_recombine.py` on `/home/user/wt/sprintl-vbs-mg/mg371/c700A_s5` (an L-V1 seed run):
base and new output both sha256 6109d4162fda765d4111b9a9be139fb2bd4007c86cab4008bb5d824525758c0a.

## Gate

Logs `/home/user/wt/logs/lz4-gates.*.log`, status `/home/user/wt/logs/lz4-gates.status`.

- `cargo fmt --all --check`: rc 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: rc 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: rc 0.
- `cargo test --workspace` with `research/refs/mg5amcnlo`, `validation/madgraph/output`,
  `validation/pdf` moved aside (restored afterwards, checked): rc 0; summed `test result`
  lines 2356 passed, 0 failed, 28 ignored. The suite grew by the four Rust tests named under
  Fixed (two in `diagram_channel.rs`, two in `maps.rs`).
- `cargo doc -p vibegraph-lib --no-deps --document-private-items`: rc 0, 13 warnings, none in a
  file this change touches (`coupling/`, `helas/eval/`, `diagrams/squared.rs`, `lhef/mod.rs`,
  `phasespace/channel.rs`).
- `cargo test -p vibegraph --features extended-validation --test cli_reweight_proton`
  (PR 5's pairing-weighted guard): 3 passed, 0 failed.
- `python3 scripts/kb.py lint`: 1394 concepts, 480 backlog items; 0 errors, 7 warnings (none in
  files touched here).
- `python3 -m unittest discover -s validation/madgraph -p test_ew_recombine.py`: 10 tests OK.

Banked `validate_*` targets were not run: nothing here moves a computed number (σ and grids above),
and the brief assigns the banked job to CI.

## What the tests cannot detect

- The `ew_recombine.py` sort fix changes only the float summation order; at the six digits it
  prints no test can see it.
- `a_built_channel_carries_the_transfer_bounds_the_cuts_prove` shows that some channels' bounds
  move and some do not, not which rung of which diagram: an error in *which* side a bound is read
  off is caught by the diagram_channel tests, not by it.
- The `products` fold (M12) is reachable only through the extended-validation CLI target.
- The bound's physics (that a single-leg side proves `|t| >= pT_min²`) is not tested by any of
  these; that is the validation of PR 5, not hygiene.

## Found

1. `cuts.rs:2247` and `maps.rs:255` write the same weakest-of-filters reduction
   `reduce(|a, b| Some(a?.min(b?)))` over `Option<f64>` bounds. Close with one helper in `cuts.rs`
   that both call.
2. `cli_reweight_proton` (the only guard of the `products` argument) needs `extended-validation`;
   a hermetic test of a pairing-weighted channel's bound (a forced decay under pairing weights,
   through `MapChoices::channel`) would guard it in the `test` job.
3. The new `test-ew-recombine` pixi task is not run by CI. Close by adding it to the workflow that
   runs `test-gpdb`, if one exists.

## Brief corrections

- The brief names `cli_reweight_proton` as a gate of this change; it compiles only under
  `extended-validation` (`error: target ... requires the features`), so it is not in the hermetic run.
- `-p vibegraph-cli` is not a package name; the CLI package is `vibegraph` (binary `vibegraph`).
- The brief says to delete closed items' files; `closes: []`, nothing to delete.
- `integrate --out` is a directory (`<out>/grid.bin.zst`), not a file path.
- The ldjson result embeds `vibegraph_version` (git describe), so the whole output differs between
  builds; the oracle compares `sigma`, `sigma_err` and the grid file.
