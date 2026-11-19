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
