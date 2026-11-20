---
type: Session Report
title: "K-H4 report: hygiene pass on the integration PRs (#142, #155)"
description: "Review of the one-body final-state phase space (#142) and the bounded configuration-share memory (#155): findings written before any edit, the mutation table of every test those PRs added, and the fixes."
status: draft
tags: [hygiene, review, non-vacuity, one-body, vegas, configuration-shares, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-19}
---
Worktree `/home/user/wt/sprintk-hyg-integrate`, branch `sprintk-hyg-integrate`,
based on `main` at `b3bc5399`. Cargo ran with `CARGO_BUILD_JOBS=2`,
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_{DEV,TEST}_DEBUG=0` and the worktree's
own `CARGO_TARGET_DIR`.

## Findings (written before any code edit)

Class: M maintainability, N non-vacuity, V visibility, A abstraction. "Fix"
means fixed in this pass; the mutation ids refer to the table below.

| id | class | where | what is wrong | evidence | disposition |
|---|---|---|---|---|---|
| F1 | N | `vegas.rs` `share_waves_move_no_bit` | The oracle (`alone`, each block as one chunk) runs through `adapt_blocks_iteration_in_waves`, the code under test, so a fault in the block close-out or the sums is in both sides. `close_through` dropping a block's open sums (M06) and `accepted += 1` (M08) pass. | M06, M08 survive | fix: add a sequential reference (own RNG replay) for the mean, the variance and `accepted` |
| F2 | N | `proton.rs` `a_one_body_point_…`, `a_one_body_mass_beyond_the_collider_is_refused` | The point test reads `x1, x2` from the map under test and checks the value pointwise, so a rapidity range cut to half (M13) passes: the Jacobian is checked against the same point. The refusal test covers only the collider-energy side of `OneBodyOutOfReach`: dropping `mass > threshold` (M11) passes. The massive-incoming probe point (`mass.max(1.2·threshold)`, M14) is never reached by any test | M11, M13, M14 survive | fix: assert the `u₁ = 0, 1` endpoints (`x₁ = 1` or `x₂ = 1`), add a below-threshold case with massive incoming partons |
| F3 | A | `budget.rs` `ChannelHistory::combine(rule)` | The channel now stores its rule (`combination`) for the share sums, and `combine` still takes it again as an argument; the two can disagree, and then the shares' terms stop summing to the channel's term. `scaled_rel` threads the same rule | `combine(&self, rule)` beside `self.combination`; callers `budget.rs:977,1240,1480` | fix: `combine` reads the stored rule |
| F4 | A | `proton.rs` `map_point_with` | The one-body branch repeats the `y`, `x1`, `x2`, `sqrt_shat` tail of the general path | 8 duplicated lines at `proton.rs:3096-3104` and `3117-3125` | fix, `to_bits` identity: shared constructor taking the Jacobian as a function of `y_max` |
| F5 | M | `proton.rs:2509,2516` | `groups.groups()[0].final_masses()` is read twice in `ProtonIntegrand::new` | grep | fix with F4's neighbourhood only if it stays bit-for-bit (a binding, no arithmetic) |
| F6 | M | `multiplicity.rs:1417` | The doc of `ONE_BODY_AND_JET` says the one-body phase space is one "which no map here integrates": false since the one-body map exists | the quoted line | fix |
| F7 | M | `amplitude_probe.rs:36-38,161` | The module doc says a one-body final state "is never integrated" and `probe_momenta` says "none: empty for a one-body final state". A `2 → 1` is integrated at proton beams now, and its probe is empty, so `check_with` skips it (`probe.points.is_empty()` → `continue`) | `amplitude_probe.rs:285` | fix the doc; the missing probe is a physics gap, Found |
| F8 | M | `README.md:194` | The "Refused by name" column still lists "one-body final states (`p p > z'` with no decay)"; at proton beams it is integrated, and only fixed-energy beams and decays refuse it (`refuse_one_body_final_state`) | `README.md:194`, `multiplicity.rs:301` | fix |
| F9 | M | `vegas.rs` `adapt_blocks_iteration_in_waves` | The wave close-out threads an `open: Option<(usize, BlockSums)>` through a closure that takes and puts back the value (`close_through`): the invariant (the open block is always `out.len()`) is implicit | `vegas.rs:1315-1328` | fix with the `share_waves` digest and the σ oracle as identity |
| F10 | M | `vegas.rs` `adapt_blocks_iteration_in_waves` length | About 140 lines | `vegas.rs:1209-1350` | leave: the per-chunk closure (55 lines) moved intact from the pre-existing body and shares its captured state; extracting it would need a generic helper with seven parameters |
| F11 | N | `diagram_channel.rs` `a_two_to_one_diagram_is_the_one_body_channel` | The identity discriminant word of `OneBody` (`w.word(2)`) can be mutated to `word(0)` (M17) with the test green. No mutation can reach it: a one-body channel differs from every multi-leg channel by its leg count, which the identity writes earlier | M17 survives | leave, a blind spot named here |
| F12 | N | `proton.rs` `a_one_body_…` | `(TauMap::BreitWigner, None)` → `(BreitWigner, _)` (M20) builds a `TauMixture` that `map_point_with` never reads under a one-body `τ` | M20 survives, no observable change but a log line | leave, equivalent |
| F13 | M | `proton.rs:1133` `legs[2]` | Suspected panic for `legs.len() == 2` (no outgoing leg) | `parse.rs:943` refuses an empty final state (`ParseError::NoFinalState`) before any `DiagramSet` exists | not reproduced |
| F14 | N | `multiplicity.rs` `fixed_energy_refuses_a_single_outgoing_particle` | Asserts the words "single outgoing particle", which both the old and the new message print | `multiplicity.rs:1528` | leave: the only refusal that prints it is this one; the branch is a single-site `match` |

### Mutation table

Each mutation is applied by `kh4-mut.py` (scratchpad), the named test filter
run, and the file restored. `caught` is "the test failed".

| id | test | mutation | caught |
|---|---|---|---|
| M01 | `shares_fold_in_as_iterations_arrive_without_growing` | `acc.1 += w * w * v` → `w * v` | yes |
| M02 | same | `InverseVariance` weight `1/variance` → `1.0` | yes |
| M03 | same | fold warm-up iterations too (`if keep` → `if true`) | yes |
| M04 | `a_share_wave_keeps_within_its_budget` | drop the `max(rayon::current_num_threads())` floor | yes |
| M05 | same | per-task size `max` → `min` | yes |
| M06 | `share_waves_move_no_bit` | `close_through` discards the open block's sums | **no** |
| M07 | same | fold a wave's chunks in reverse | yes |
| M08 | same | `accepted += 1` | **no** |
| M09 | `a_share_wave_keeps_within_its_budget` | `n_shares == 0` wave of 1 task | yes |
| M10 | `a_one_body_point_is_the_narrow_resonance_…` | Jacobian `2y_max/(τs)` → `2y_max/s` | yes |
| M11 | `a_one_body_mass_beyond_the_collider_is_refused` | drop `mass > threshold` | **no** |
| M12 | same | drop `tau < 1.0` | yes |
| M13 | `a_one_body_…` | `y` range halved | **no** |
| M14 | `proton::tests` (whole module) | probe `√ŝ = mass` for a one-body final state | **no** |
| M15 | `a_massless_single_outgoing_particle_is_refused` | the massless check off | yes |
| M16 | `a_two_to_one_diagram_is_the_one_body_channel` | one-body momentum doubled | yes |
| M17 | same | `OneBody` identity word `2` → `0` | **no** (unreachable, F11) |
| M18 | same | `OneBody` map key `1` → `T` | yes |
| M19 | same | one-body density `1` → `2` | yes |
| M20 | `a_one_body_…` | Breit–Wigner `τ` mixture built under a one-body `τ` | **no** (equivalent, F12) |

### Routed and Found (so far)

- Physics: the amplitude probe records no point for a `2 → 1` subprocess
  (F7), so a changed `p p > z` matrix element is not re-checked on a later
  run. A natural probe point is `√ŝ = M` at rest. It would be a new oracle,
  so it goes to validation-dev.

## What changed

One code commit and this report. Behaviour is unchanged: no computed number
moved (oracle below).

| id | disposition | change |
|---|---|---|
| F1 | fixed | `share_waves_move_no_bit` replays each block's points one at a time from the draw (own RNG replay, no wave, no tile, no close-out) and compares the estimate, variance, `Σf²`, `accepted` and every share's integral at 1e-12 (variance 1e-9) relative |
| F2 | fixed | `a_one_body_point_…` also asserts the `u₁ = 0, 1` endpoints (`x₂ = 1`, `x₁ = 1`); new `a_one_body_mass_below_the_incoming_threshold_is_refused` (`t t~ > h`, threshold `2 m_t`); new `a_one_body_probe_point_is_the_collision_at_rest` (beams sum to the one outgoing momentum, on their own shells, for a massless and a massive incoming pair) |
| F3 | fixed | `ChannelHistory::combine`, `running_estimate` and `scaled_rel` read the channel's stored rule; no caller can pass a rule the shares were not folded under |
| F4 | fixed | `map_point_with` has one `y → (x₁, x₂, √ŝ)` tail; the Jacobian is `Some(dτ/du / τ)·2y_max` or the one-body `2y_max/(τ s)` with each original expression kept, so the bits are those of the two branches |
| F5 | fixed | `final_masses` read once in `ProtonIntegrand::new` |
| F6 | fixed | `ONE_BODY_AND_JET` doc |
| F7 | fixed | `amplitude_probe.rs` module doc and `probe_momenta` doc no longer say a one-body final state "is never integrated" |
| F8 | fixed | `README.md:194`: one-body final states are refused at fixed-energy beams only |
| F9 | fixed | `adapt_blocks_iteration_in_waves` holds `open: Option<BlockSums>`, always the block `out.len()`; `close_through` takes it or starts an empty one |
| F10 | leave | named reason in the table above |
| F11, F12 | leave | unreachable / equivalent, named above |
| F13 | not reproduced | `ParseError::NoFinalState` precedes any diagram set |
| F14 | leave | named above |
| extra | fixed | two rustdoc warnings in the PRs' code: `vegas.rs:1033` linked `adapt_blocks_iteration_shared`, which no longer exists; `proton.rs:3093` a redundant link target |

Tests added: `a_one_body_mass_below_the_incoming_threshold_is_refused` and
`a_one_body_probe_point_is_the_collision_at_rest` (`vibegraph-lib` lib suite
1427 → 1429). No test deleted or weakened.

## Mutations against the strengthened tests

`kh4-mut.py`, same driver. "Old" is the result against the test as merged.

| id | mutation | old test | new test |
|---|---|---|---|
| M06 | `close_through` discards the open block's sums | passes | **fails** (`share_waves_move_no_bit`) |
| M08 | `accepted += 1` per point | passes | **fails** |
| M11 | drop `mass > threshold` in `OneBodyOutOfReach` | passes | **fails** (`a_one_body_mass_below_…`) |
| M13 | `y` range halved | passes | **fails** (`a_one_body_point_…`, endpoints) |
| M14 | one-body probe `√ŝ = mass` | passes | **fails** (`a_one_body_probe_point_…`) |
| M01–M05, M07, M09, M10, M12, M15, M16, M18, M19 | see first table | caught | caught |

Not reachable: M17 (the `OneBody` identity word: every one-body identity
already differs from a multi-leg one by its leg count, which is written
first) and M20 (the Breit–Wigner mixture built and then unread under a fixed
`τ`).

What the new tests cannot see: `share_waves_move_no_bit`'s replay shares the
draw (`draw_point`, the stream and word position) with the code under test, so
a fault in the draw is in both; `a_one_body_probe_point_…` pins the probe
point, not the amplitude evaluated there.

## Bit for bit against main

`main` at `b3bc5399` (this branch's base, detached worktree, own release
build, deleted afterwards) against this branch, both `vibegraph integrate <card>
--run-card <card> --no-network --fixed-budget --neval 20000 --niter 3 --seed 7
-j 2`, `LHAPDF_DATA_PATH` at the worktree's `validation/pdf` (script
`kh4-oracle.sh`):

```
card (proc, run card)                                   σ, both         grid.bin.zst sha256 (first 16), both
dy13_default  (p p > e+ e-, dy13_default)              935 ± 11 pb     063e6ff624eba7cd
z_one_body    (p p > z, dy13_default)                  42.126 ± 0.071 nb fa74752346a6e9c9
pp_to_llj_mlm (p p > e+ e- j, pp_to_llj_mlm)           267.0 ± 2.5 pb  168ec7ef694648eb
z_zj_matched  (p p > z + p p > z j @1, pp_to_llj_mlm)  57.08 ± 0.15 nb 60541ba1e5cc97d5
```

The grids are identical, so the channels, the shares' adaptation and every σ
are. The matched card is the one that folds shares and the one-body part
together.

## Gates

On the final tree: `cargo fmt --all --check` clean; `cargo clippy --workspace
--all-targets -- -D warnings` and the same with `--all-features` finished
clean; `cargo test --workspace` 0 failed (library suite 1429 passed, 15
ignored); `cargo doc -p vibegraph-lib --no-deps --document-private-items`
reports no warning in a file this pass touched; `pixi run kb-lint`: 1301
concepts, 433 backlog items, 0 errors, 5 warnings (none in this pass's files).
The banked layer is CI's.

## Found

- **No amplitude probe for a `2 → 1`** (`amplitude_probe.rs:162`,
  `probe_momenta` returns no points for fewer than two outgoing legs and
  `check_with` skips an empty probe). A `p p > z` card's matrix element is
  therefore not re-checked on a later run. A probe at `√ŝ = M`, at rest, is a
  new oracle: validation-dev.
- **Release-only dead code**: `NodeAnalysis::resolve_mom`
  (`helas/eval/analysis.rs:208`) is never used, a `dead_code` warning in the
  release build that `cargo clippy` does not report. Delete it or gate it.
- **Brief**: the brief names the K-P1 2 → 1 card without its proc card; the
  oracle used `generate p p > z` with `dy13_default_run_card.dat`, K-P1's own
  measurement card. The base build needs `LHAPDF_DATA_PATH` for the local
  `validation/pdf` sets; a copy into `~/.vibegraph/pdf` was not accepted
  (`--no-network` reported the set missing).
