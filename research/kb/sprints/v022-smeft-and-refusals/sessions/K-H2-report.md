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
