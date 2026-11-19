---
type: Session Report
title: "K-H2 report: hygiene pass on the diagram, colour and grouping PRs"
description: "Findings of the hygiene review of #146, #147, #150 and #153 (written before any edit), the fixes with their mutations, the bit-for-bit oracles and what is left for Found."
status: draft
tags: [hygiene, colour, decay-chain, grouping, matching, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Branch `sprintk-hyg-diagrams` off main 92dee0d7 ([brief](K-H2.md)). Reviewed, as
they stand on main: `82ed311c` (#146), `15de3a8f` (#147), `0340a897` (#150) and
`74596ba3` (#153), with their direct callers and tests. This section was
committed before any code edit.

## Findings (before any edit)

Points: 1 maintainability, 2 non-vacuity, 3 visibility, 4 abstractions.
Ranked by consequence. *checked* means the quoted evidence settles it;
*suspected* rows name the mutation that settles them, run in the next section.

| id | pt | site | evidence | confidence | disposition |
|---|---|---|---|---|---|
| D1 | 1 | `vibegraph-lib/src/proton.rs:924` | #153 put `vanishes_at_card` between `worst_rel` and its doc line: the rustdoc of `vanishes_at_card` opens "Worst relative disagreement between two probe traces. Whether every diagram of `set` passes …", and `worst_rel` (`:949`) is undocumented | checked (quoted lines 924–930) | fix |
| D2 | 2 | `vibegraph-lib/src/helas/color/colorize.rs:780` `summed_fundamental_indices_outside_a_chain_are_refused` | The `anywhere != 2` clause of `check_generator_chains` (`:479`) is never the deciding condition: in the "adjoint position" case `T(-1,2,-2)*T(1,-2,-1)` and the `Epsilon` case `T(1,2,-1)*Epsilon(-1,3,4)` index `-1` is never a fundamental index, so `as_fundamental != 1` refuses both first. Mutation M2: delete `\|\| anywhere != 2`. The test also matches the substring `"summed fundamental index -1"`, which the "ends a generator chain" refusal prints as well | suspected (M2) | fix: a case only the count refuses (`T(-1,2,-1)*T(1,-1,3)`), match `"does not join"` |
| D3 | 1/4 | `colorize.rs:450` `check_generator_chains` and `:377` `merge_generator_chains` | Two walks of the same chains with the same two refusals, word for word ("summed fundamental index {next} ends a generator chain", "a closed loop … (a trace)"). `colorize_diagram` (`:537`) runs `check_slot_reps`, which calls `check_generator_chains` (`:357`), on every structure `convert_expr` (`:569`) later merges, so the walk and its loop check run twice | checked | fix: `check_generator_chains` keeps the per-index rule and hands the walk to `merge_generator_chains` |
| D4 | 3 | `vibegraph-lib/src/diagrams/chain.rs:111` | `#[cfg_attr(not(test), allow(dead_code))] pub(crate) struct Merge`: production builds a record whose fields only a test reads, and silences the lint instead | checked | fix: report through an optional callback, the record type test-only |
| D5 | 1 | `chain.rs:747` `lowest_leg_beyond` | A second copy of the tree walk in `Diagram::legs_on_start_side` (`diagrams/diagram.rs:732`): pop a vertex, collect its legs, cross every other propagator to its far end | checked | fix: read the far side off `legs_on_start_side` |
| D6 | 3 | `proton.rs:415` `pub fn FlavorGroup::vanishes` | `grep -rn "vanishes()" vibegraph-lib vibegraph-cli` outside `proton.rs`: no hit | checked | fix: narrow |
| D7 | 3 | `proton.rs:235` `pub fn Subprocess::label` | Its one caller is `member_rewgt` (`proton.rs:3643`); no use in `vibegraph-cli` or elsewhere in the library | checked | fix: narrow |
| D8 | 2 | `proton.rs` `a_group_joined_on_a_vanishing_matrix_element_is_not_clustered`, second half | Asserts the factors are zero only through `factors.iter().all(\|&f\| f == 0.0)`. Mutation M11b: return the factors gathered before the refusal (`return factors;`) instead of `vec![0.0; n]`; a short or empty vector passes `all` | suspected (M11b) | fix if it survives: assert the length too |
| D9 | 2 | `#153`'s `refuse_sampled_fault(integ.fault())?` calls on the proton paths (`vibegraph-cli/src/integrate.rs:1594,1600`, `vibegraph-lib/src/generate.rs:1174,1242`) | No card reaches a sampled fault once the vanishing group is skipped, so deleting a call passes every test | checked | leave: settled as K-H3 H13 (no mutation reaches it) |
| D10 | 4 | `proton.rs:3743`/`hadronic.rs:2780` fault slot, `refuse_sampled_fault` in two crates | — | checked | leave: already K-H3 H9's Found |
| D11 | 1 | `chain.rs` `stitch_set` `assert_eq!(*sign, d.sign, "one stitched graph reached twice with opposite signs")` | A panic on the merge path #150 widened. It predates #150 (context lines of its diff) and asserts an invariant of the graph (the sign is a function of the canonical form); no input reaches it | checked | Found (turn into a named refusal) |
| D12 | 1 | `proton.rs` `derive_flavor_groups`, 268 lines; `chain.rs` `stitch_set`, 141 | #153 grew the first from 235 lines, #150 the second by its merge record. Both were past 100 before | checked | Found (pre-existing length) |
| D13 | 1 | `vibegraph-lib/src/helas/eval/stitching.rs` `merged_stitched_diagrams_have_equal_amplitudes` | Builds a one-diagram `DiagramSet` by hand: one more site of the filed item `one-diagram-diagramset-built-by-hand` | checked | Found (add the site to that item) |
| D14 | 1 | `vibegraph-lib/src/multiplicity.rs` `equal_masses_in_different_slots_split_into_parts_of_their_own` `eprintln!("parts: …")` | Diagnostic output in a test; the module's other tests print the same way | checked | leave (house style) |

Mutation plan for every test the four PRs added (one mutation each, applied
temporarily, run, reverted):

| test | mutation |
|---|---|
| `colorize.rs` `generator_chains_between_slots_are_read` | M1: the chain walk looks up the next link by its `3̄` end (`j == next`) |
| `colorize.rs` `summed_fundamental_indices_outside_a_chain_are_refused` | M2: delete `\|\| anywhere != 2` |
| `colorize.rs` `a_generator_chain_crosses_as_one_generator` | M3: `convert_expr` reads `expr.atoms` unmerged (link-by-link crossing) |
| `standalone_jamps.rs` `mssm_contact_structures_reach_madgraphs_flows` (extended-validation) | M3 |
| `multiplicity.rs` `equal_masses_in_different_slots_split_into_parts_of_their_own` | M5: `split_by_external_masses` keys the outgoing masses as a sorted multiset |
| `stitching.rs` `merged_stitched_diagrams_have_equal_amplitudes` | M6: `slot_symmetric_key` groups boson slots by spin, not by particle |
| `stitching.rs` `decays_on_slots_of_different_particles_are_all_kept` | M6 |
| `stitching.rs` `identical_decays_on_interchangeable_slots_are_stitched_once` | M8: `slot_symmetric_key` returns `diagram.canonical(model)` unordered |
| `decay_chain_census.rs` `decay_chain_census_matches_madgraph` (symmetry-factor row) | M8 |
| `proton.rs` `a_subprocess_with_one_live_diagram_does_not_vanish` | M10: `vanishes_at_card` takes `any` diagram, not `all` |
| `proton.rs` `a_group_joined_on_a_vanishing_matrix_element_is_not_clustered` | M11: drop `g.vanishes() \|\|` from the clustering skip; M11b as D8 |

## Final dispositions

| id | disposition | commit |
|---|---|---|
| D1 | fixed: the doc line moved back onto `worst_rel` | `fbe74515` |
| D2 | fixed: M2 survived the old test (below); the new case `T(-1,2,-1)*T(1,-1,3)` is refused only by the index count, and every per-index case now matches `"summed fundamental index -1 does not join"` | `c9a21ce4` |
| D3 | fixed: `check_generator_chains` keeps the per-index rule and ends in `merge_generator_chains(expr).map(drop)`; the second walk is gone | `c9a21ce4` |
| D4 | fixed: `Merges` holds an optional `&mut dyn FnMut(..)` (`OnMerge`); `Merge` is `#[cfg(test)]` and built by the test's own callback; no `allow` left | `a07ce02c` |
| D5 | fixed: `lowest_leg_beyond` reads `Diagram::legs_on_start_side` | `a07ce02c` |
| D6 | fixed: `FlavorGroup::vanishes` and `vanishes_at_card` private | `fbe74515` |
| D7 | fixed: `Subprocess::label` private | `fbe74515` |
| D8 | not reproduced: M11b fails the test (`assertion failed: factors.iter().all(\|&f\| f == 0.0)` at `proton.rs:8263`): an earlier member's factor is non-zero, so the partial vector is caught | — |
| D9 | leave (K-H3 H13) | — |
| D10 | leave (K-H3 H9, Found there) | — |
| D11–D13 | Found | — |
| D14 | leave | — |

## Mutations

Driven by `/home/user/wt/kh2-mut/drive.py` (apply one textual mutation, run
`cargo test -p vibegraph-lib <target> <filter>`, restore, log). Every run below
is the mutated build; the clean build passes every listed test (run `CLEAN`:
15 lib tests and `decay_chain_census` green before the edits, 16 after, with
the added identity test).

| id | mutation | test(s) | before the fixes | after |
|---|---|---|---|---|
| M1 | chain walk reads only the first link (`links.iter().take(1)`) in `check_generator_chains` | `generator_chains_between_slots_are_read` | FAILED (caught) | M1′, the same in `merge_generator_chains`: FAILED |
| M2 | delete `\|\| anywhere != 2` | `summed_fundamental_indices_outside_a_chain_are_refused` | **ok (survived)** | FAILED: `unwrap_err()` on `Ok` for the new case |
| M3 | `convert_expr` reads `expr.atoms` unmerged (link-by-link crossing) | `a_generator_chain_crosses_as_one_generator` | FAILED | FAILED |
| M3x | M3 | `standalone_jamps -- mssm` (extended-validation) | — | `mssm_contact_structures_reach_madgraphs_flows` FAILED ("flow 0 (MadGraph's JAMP(1)) takes structure 1"); `gg_to_t1t1x_mssm` and `uux_to_t1t1x_mssm` ok, the blindness their doc states |
| M5 | outgoing key sorted (a multiset) in `split_by_external_masses` | `equal_masses_in_different_slots_split_into_parts_of_their_own` | FAILED | (code untouched) |
| M6 | boson slots grouped by spin, not particle | `stitching::` | ok (survived) | see below |
| M6b | boson slots grouped as all bosons (`spin % 2`) | `stitching::`, `decay_chain_census`, the identity digest | ok, and every diagram and merge digest identical to the clean build | — |
| M7 | stitch only the first permutation | `decays_on_slots_of_different_particles_are_all_kept` | FAILED (`left: 1, right: 4`) | (code untouched) |
| M8 | `slot_symmetric_key` returns `canonical` unordered | `identical_decays_on_interchangeable_slots_are_stitched_once`, `merged_stitched_diagrams_have_equal_amplitudes`, `decay_chain_census_matches_madgraph` | all three FAILED | all three FAILED |
| M5d | `lowest_leg_beyond` reads the near side (`end == 0`) | `stitching::`, `decay_chain_census` | — | FAILED (both) |
| M10 | `vanishes_at_card` takes `any` diagram | `a_subprocess_with_one_live_diagram_does_not_vanish` | FAILED | (code narrowed only) |
| M11 | drop `g.vanishes() \|\|` from the clustering skip | `a_group_joined_on_a_vanishing_matrix_element_is_not_clustered` | FAILED | — |
| M11b | `return factors;` for `vec![0.0; n]` | same | FAILED | — |

**Equivalent mutants (M6, M6b).** `slot_symmetric_key` restricts its reordering
to slots of one particle, and its doc says slots of different particles are not
interchangeable. No test can see that restriction, and none could: a line
moved from a `Z` slot to an `h` slot is not a diagram of the vertex, so two
valid stitched graphs never differ by such a move, and widening the grouping
only renames every key the same way. M6b leaves every census and stitching
card's diagram sets and merge records byte-identical. The restriction is
defensive; `decays_on_slots_of_different_particles_are_all_kept` guards a lost
arrangement (M7), not the grouping.

Sites no mutation reaches: the "summed fundamental index … ends a generator
chain" refusal in `merge_generator_chains` (after the per-index rule every
summed index has a `3` end, so the walk always finds the next link); #153's
`refuse_sampled_fault` call sites (K-H3 H13).

## Identity oracles

- **Colour (D3).** A temporary test compared the old `check_generator_chains`,
  copied verbatim, with the new one on 19 structures (the test cases, three- and
  four-link chains in both orders, two traces, chains beside unlinked `T`, `f`
  and `Identity`): equal `Ok`/error message on all 19 (5 traces, 5 per-index
  refusals, 9 accepted). Deleted before the commit.
- **Stitching (D4, D5).** A temporary test printed, for every census card
  (`validation/madgraph/decay_chain_census.json`, 33 cards) and every
  `stitching.rs` `CASES` card, the sha256 of the `Debug` form of the
  stitched sets and of every merge record (`particles_in`, `particles_out`,
  `polarizations`, `kept`, `merged`), all under the default SM (a deterministic
  identity check, not the census's own models): 34 distinct cards, 32 with a
  merge record, plus a digest of all of them. Every line is identical before
  and after the edits (`/home/user/wt/kh2-mut/identity-before.txt`, `CLEAN2.log`).
- **σ and grid, bit for bit against main 92dee0d7** (main built from `git
  archive` in its own target dir, deleted after its runs).
  `vibegraph --no-network integrate <proc> --run-card validation/madgraph/<card>_run_card.dat --fixed-budget --neval 20000 --niter 3 --seed 7 -j 2 --no-tui`,
  `LHAPDF_DATA_PATH=validation/pdf`:

  | run card | proc card | main | this branch |
  |---|---|---|---|
  | `pp_to_llj_mlm` | `import model sm` / `generate p p > e+ e- j` | 6.856446e-7 ± 6.495532e-9, grid `95f0c1bf…1d084d` | identical |
  | `pp_to_ttx_0j1j_mlm` | `generate p p > t t~ @0` / `add process p p > t t~ j @1` | 2.786081e-6 ± 1.603551e-8, grid `99718440…dc079e` | identical |
  | `dy13_default` | `generate p p > e+ e-` | 2.402473e-6 ± 2.742323e-8, grid `b5321668…3300f0` | identical |

  These cards hold no decay chain and no summed colour index, so they see
  only the narrowing and doc edits; the two digests above are the oracle for
  the stitching and colour refactors.

## What each test provably cannot detect

- `summed_fundamental_indices_outside_a_chain_are_refused`: which refusal a
  structure meets when it breaks two rules (the per-index rule runs first).
- `a_generator_chain_crosses_as_one_generator`: a crossing error that the
  single-generator comparison shares (both sides go through `convert_expr`);
  `mssm_contact_structures_reach_madgraphs_flows` is the external reference.
- `decays_on_slots_of_different_particles_are_all_kept`: the particle
  restriction of the slot grouping (equivalent mutant, above).
- `merged_stitched_diagrams_have_equal_amplitudes`: a merge of two diagrams
  equal at its one point (its doc says so).

## Found

1. **A panic on the stitch merge path** (D11). `diagrams/chain.rs` `stitch_set`:
   `assert_eq!(*sign, d.sign, "one stitched graph reached twice with opposite signs")`.
   No input reaches it today (the sign is a function of the graph), but it is
   a panic in diagram generation; close by returning the decay-chain refusal
   the same block already returns for an ambiguous forced line.
2. **Two long functions grown further** (D12). `proton.rs` `derive_flavor_groups`
   is 268 lines (235 before #153's vanishing block); `chain.rs` `stitch_set` is
   141. Close by extracting the per-group assembly and the vanishing report
   from the first and the dedup loop from the second.
3. **One more hand-built one-diagram `DiagramSet`** (D13).
   `helas/eval/stitching.rs` `merged_stitched_diagrams_have_equal_amplitudes`
   assembles `DiagramSet { …, diagrams: vec![m.kept.clone()], mirrored: false }`:
   add the site to `hygiene/one-diagram-diagramset-built-by-hand`.

## Corrections to the brief

- The brief gives no proc cards for the three run cards; the ones used are in
  the table above, read from `validation/madgraph/scripts/*.mg5` and
  `output/dy13_default/Cards/proc_card_mg5.dat`.
- Disk: the container had 3.2 GB free at the start (other worktrees' finished
  `target/` dirs hold ~8.5 GB), not the ~24 GB the discipline file states. The
  main baseline and the final gates were built in `/dev/shm` (tmpfs) rather
  than the worktree's `target/`, which was deleted before the gates.
- The three fix commits were made before the gate run below; the gates ran on
  that tree.

## Gates

Run on `a07ce02c` (lean cargo variables, `CARGO_BUILD_JOBS=3`, target in
`/dev/shm/kh2-target`, driver `/home/user/wt/kh2-gates.sh`):

| command | tail |
|---|---|
| `cargo fmt --all --check` | rc 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | `Finished dev profile … in 1m 36s`, rc 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | `Finished dev profile … in 31.76s`, rc 0 |
| `cargo test --workspace` | 69 test binaries: 2241 passed, 0 failed, 26 ignored; rc 0. No `#[test]` was added or removed (`git diff 92dee0d7 -- '*.rs'` holds none), so the count is main's |
| `cargo doc -p vibegraph-lib --no-deps --document-private-items` | 7 warnings, none in a file this branch touches (`configs.rs`, `channel.rs`, `alphas.rs`, `scales.rs`, `lhef/mod.rs`); rc 0 |
| `pixi run kb-lint` | `1307 concepts, 433 backlog items; 0 errors, 5 warnings` (all backlog items this branch does not touch) |
| `cargo test -p vibegraph-lib --features extended-validation --test standalone_jamps -- mssm` | the colour change's banked rows: `uux_to_t1t1x_mssm`, `gg_to_t1t1x_mssm`, `mssm_contact_structures_reach_madgraphs_flows` ok (dev profile, not `release-debug`) |

The brief closes no backlog item, so no item file is deleted.
