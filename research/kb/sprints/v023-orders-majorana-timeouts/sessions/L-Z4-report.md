---
type: Session Report
title: "L-Z4 report: hygiene pass on the multichannel transfer bound and the equal-weight recombination script"
description: "Review of PR 5 (#184: single_leg_transfer_bound, with_side_transfer_bounds and the maps.rs wiring) and PR 4's ew_recombine.py. Thirteen mutations of PR 5's code: nine caught by its own tests, four survived or were caught only by a distant test; the script crashed on three empty-input shapes and had no tests."
status: draft
tags: [hygiene, multichannel, spine, cuts, validation, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-24}
---
Branch `sprintl-hyg-phasespace` off main 8b3af5ab. Source of every claim below
is a command shown with it; paths are relative to the worktree
`/home/user/wt/sprintl-hyg-phasespace`.

This section is the findings table, committed before any code edit. Fix, mutation,
oracle and gate sections follow in later commits.

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
