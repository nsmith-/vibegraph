---
type: Session Report
title: "HY1 report: hygiene review of the mixed-mass split and its gate"
description: "Two red tests after the row was unplanned: the collator's planned-row test and the mixed-mass gate on a checkout without the record. Both are fixed. The gate also gains the scale assertions its neighbour makes, and the one-per-phase-space panic gets a test. Stale docs and messages are fixed, and the cross-file duplication and stale kb sites go to Found."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-23}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-23}
---
Worktree `/home/user/wt/L`, branch `sprintd-flavour-mixed-masses`. The review
started at `e935ca3`. Mid-session the remote gained `5ee2e83` and a merge,
`3da3ed9`. On the manager's instruction the branch was fast-forwarded to
`3da3ed9` (`git merge --ff-only`) before any commit. `vegas.rs`, `budget.rs`
and the phase-space maps are untouched, and no backlog item was edited.

## Findings (written before any edit)

| id | pt | site (at `e935ca3`) | evidence | conf | disposition |
|---|---|---|---|---|---|
| H1 | 2, bug | `validation-report/src/main.rs:500` `a_planned_row_with_no_cell_waits_and_an_ordinary_one_is_missing` | `cargo test -p vibegraph-validation-report` on `e935ca3`: `FAILED`, `left: ("⏳", 0) right: ("❌", 1)`. `ee6f18c` replaced the row's `status = "planned"` with `bundled = false`, so the test's `replace` was a no-op and the unbundled branch answered both halves | checked | **fixed**, as #67's `fc2d369` on top of the remote's `5ee2e83` (the manager's call) |
| H2 | 2, bug | `vibegraph-lib/tests/validate_hadronic.rs:750-766` | `cargo test -p vibegraph-lib --features extended-validation --test validate_hadronic -- sigma_mixed` on `e935ca3` with no record: `FAILED`, `validation.rs:32: needs a banked seed record (dy13_etau_default)`. The row is `bundled = false` and not planned. The collator renders it "awaiting the bundle", and every other banked gate passes over such a row (`dyn_run_present`, `unbundled_rows()`) | checked | **fixed** |
| H3 | 1 | `validate_hadronic.rs:717-959` `sigma_mixed_outgoing_masses_vs_mg` | 243 lines | checked | **fixed**: the record reading and the seed loop extracted |
| H4 | 1 | `validate_hadronic.rs:911` | `let pull = (mean - mg) / (err * err + mg_err * mg_err).sqrt();` beside `common::seeds::SeedMean::pull`, the same formula | checked | **fixed** in this PR's copy; the older copies are Found 2 |
| H5 | 2 | `validate_hadronic.rs:837-839` | the `use_run_card_scales` report is `.expect`ed and dropped. `check_dy_run`'s path (`adapted_integrand`) asserts constant scales at `m_Z` and `depends_on_alpha_s == false` on the same run card. Mutation: `fixed_ren_scale` `True` → `False` in `dy13_default_run_card.dat` | checked | **fixed** |
| H6 | 2 | `vibegraph-lib/src/multiplicity.rs:279-286` `MultiplicitySum::new` | `grep should_panic multiplicity.rs`: none, so the new "one per phase space" assertion has no test. Mutation: make it always true | checked | **fixed**: a `should_panic` test. The ordering message said "increasing" for a `<=` check, now "non-decreasing", as is the `parts()` doc |
| H7 | 1, 2 | `multiplicity.rs:1029-1033`, doc of `mixed_mass_parts_integrate_to_one_flavour_each` | the doc claims a part "evaluated on the other's mass shell ... moves the ratio off one half by far more than its error", with the mass effect at `~1e-3`. The bound is `5σ + 0.002` with σ = 0.0043 (F1: `ratio 0.50583 ± 0.00431`), so about ±0.024 wide. F1 measured the tau-mass effect at −1.15% on the dy13 card | checked | **fixed** (doc): it names what the test sees and its blind spot |
| H8 | 1 | `multiplicity.rs:91-101`, `split_by_outgoing_masses` doc | no stated tolerance. The key is `to_bits`, exact, as `derive_flavor_groups` compares `masses != final_masses` (`proton.rs:793`) from the same `evaluated.mass(id)` (`hadronic.rs:679`). "A name the model does not know keys alone" is inexact: every unknown name maps to the one bit pattern of `f64::NAN` | checked | **fixed** (doc) |
| H9 | 4 | `multiplicity.rs:107-118` and `:634-643` | two copies of masses → `to_bits` → sort | checked | **fixed**: one `mass_multiset` |
| H10 | 3 | `multiplicity.rs:102`, `pub fn split_by_outgoing_masses` | `grep -rn split_by_outgoing_masses`: calls only in `multiplicity.rs`, and doc links from `proton.rs` | checked | **fixed**: `pub(crate)` |
| H11 | 1 | `lhef/emit.rs:244-246` (`PartsUnsupported` message), `:889`; `generate/source.rs:368-372`; `multiplicity.rs:183-185` (`multiplicity_maps` doc) | these still read "multiplicities" where a part is now a multiplicity or a set of masses. F1 updated the `generate.rs` and CLI copies of the same refusal | checked | **fixed** |
| H12 | 1 | `docs/src/guide/10-hadronic.md:55-64`, `11-unweighting-events.md:195-199` | the guide describes the grouping refusal and the stochastic-rounding refusal by multiplicity only, and says nothing of the split | checked | **fixed** |
| H13 | 4 | `vibegraph-cli/src/integrate.rs:871-895`, `vibegraph-lib/src/generate.rs:844-865`, `validate_hadronic.rs:820-842` (this PR), `:5397-5419` | four copies of `MultiplicityGroups` → per-part `ProtonIntegrand` (maps, vetoes, run-card scales) → `MultiplicitySum` | checked | **Found 1** |
| H14 | 4 | `validate_hadronic.rs` (8 sites) and `validate_sigma.rs` (2) | `grep -c "row.pull = "`: the same `IntegralsRow` fill, and a hand-rolled pull in `check_dy_run` | checked | **Found 2** |
| H15 | kb | `research/kb/hadronic/flavour-groups.md:41-44`; `phase-space/mixed-multiplicity-integrand.md:25-33, 108`; backlog `stochastic-rounding-refuses-mixed-multiplicity` | stale against the split | checked | **Found 3-5** |
| H16 | 1 | the split key's `to_bits`: `-0.0` and `0.0` differ by bits, equal by `==` | reading | checked | **leave**: no card writes a negative zero mass, and if one did, the decomposition would refuse it loudly, not merge it wrongly |

Brief questions:
- **Is the split done once, in the library, for every entry point?** Yes. The two production paths that build integrands, CLI `integrate` (`integrate.rs:858`) and library `generate` (`generate.rs:806`), both go through `multiplicity_groups`. The validation gate does too. The other `derive_flavor_groups` callers are tests and probes on single-mass cards. The `validate_hadronic.rs:5347` matched-sample gate is one of them: it hand-splits by multiplicity only, and its rows have one mass set.
- **Is the per-part bookkeeping reused?** Yes. The gate, the CLI report and the unit test all read `MultiplicitySum::part_results`.
- **Is the planned-row handling local?** The test side's was local to this PR. After H2 nothing reads `planned_rows()`, so it and the test reader's `status` field are deleted, and `tests/common/manifest.rs` is back to `origin/main`'s text. The collator's `Planned` branch stays: it is the manifest's documented vocabulary, and #67 adds the same branch.
- **Is the refusal text removed everywhere it was quoted?** `grep -rn "no single phase-space map"` finds it only in `proton.rs`, where the refusal stays as the decomposition's invariant, with its new ending "split them by outgoing masses first". The kb concept that describes the refusal without the split is Found 3.

## Fixes and evidence

Every cargo command below ran with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`. Mutations were run by a script that applies the edit, runs `cargo test`, and restores the file.

**H1** (commit `b63ecd4`). `.replace("bundled = false\n", "")` is now in the stripping chain for both halves, as #67's `fc2d369` does.

```
=== if false && process.status == Some(ProcessStatus::Planned), test as on 5ee2e83
test tests::a_planned_row_with_no_cell_waits_and_an_ordinary_one_is_missing ... ok
=== same mutation, after the fix
test tests::a_planned_row_with_no_cell_waits_and_an_ordinary_one_is_missing ... FAILED
  left: ("❌", 1)
 right: ("⏳", 0)
=== unmutated, cargo test -p vibegraph-validation-report
test result: ok. 45 passed / 20 passed / 12 passed; 0 failed
```

**H2-H5** (`validate_hadronic.rs`, `tests/common/manifest.rs`):
- The absent-record branch passes over the measurement only for a row in `unbundled_rows()`, printing `AWAITING BUNDLE`. Otherwise it calls `validation::require`.
- `mixed_mass_directory_means` reads MadEvent's two directories.
- `mixed_mass_runs` is this crate's side.
- The pull comes from `SeedMean::pull`.
- The scale report is asserted through `assert_scale_report`, extracted verbatim from `adapted_integrand`, which now calls it too.

Identity oracle, chosen before the edit: the gate's 14 `eprintln!` lines on a **synthetic** seed record. It has five seeds and two directories, `P1_qq_ll` and `P1_qq_taptam`, with made-up values, and it was written by a script and deleted afterwards. It says nothing about MadGraph. Before and after:

```
$ diff fmL-HY1-synth-before.lines fmL-HY1-synth-after.lines && echo IDENTICAL
IDENTICAL 14 lines
(before: test result: ok. 1 passed ... 6.70s; after: ok. 1 passed ... 13.33s)
```

Mutations on the gate:

```
=== no record, manifest as committed (after the fix)
test sigma_mixed_outgoing_masses_vs_mg ... ok          (prints AWAITING BUNDLE)
=== no record, the same test before the fix (e935ca3)
test sigma_mixed_outgoing_masses_vs_mg ... FAILED      (validation.rs:32 needs a banked seed record)
=== MB1: no record, `bundled = false` removed from pp_to_ll_etau (after the fix)
test sigma_mixed_outgoing_masses_vs_mg ... FAILED      (validation.rs:32 needs a banked seed record)
=== MS1: synthetic record, dy13_default_run_card.dat `True = fixed_ren_scale` -> `False`
old test (3da3ed9's file): test sigma_mixed_outgoing_masses_vs_mg ... ok   (68.71s; muR does not enter Drell-Yan, so σ cannot see it)
new test:                  FAILED at validate_hadronic.rs:481 "the reference run card no longer fixes both scales"
```

**H6** (`multiplicity.rs`): the new test is `a_sum_refuses_two_parts_of_one_phase_space`, a `should_panic` on "the parts are one per phase space".

```
=== fixed code: cargo test -p vibegraph-lib --lib multiplicity::
test result: ok. 7 passed; 0 failed
=== MP1 `true || sorted_final_masses(a) != sorted_final_masses(b)`, the module's other tests (as before the fix)
test result: ok. 6 passed; 0 failed
=== MP1, the new test
test multiplicity::tests::a_sum_refuses_two_parts_of_one_phase_space - should panic ... FAILED
```

**H9** (`multiplicity.rs`): one `mass_multiset`. F1's M2 rerun on it (`bits.sort_unstable()` → `bits.clear()`) fails four tests: both mixed-mass tests, `the_sum_s_channels_are_its_parts_channels_offset` and `the_sum_s_cross_section_and_sample_split_are_its_parts`. `a_sum_of_one_multiplicity_is_that_multiplicity`, the module's bit-for-bit test, passes on the fixed code.

**H7, H8, H10-H12**: docs, messages and visibility, with no test change. No test asserts on the `PartsUnsupported` text; `emit.rs:1728` matches the variant only.

Sites no mutation reached:
- **The gate's three bounds and the control.** They run only on a seed record. The record exists only on the M3, so here they ran on the synthetic one alone.
- **The `ta_part_on_massless_channels` direction.** A mutation there would need MadGraph's real directory values.

## Gate

All on the final tree (log `fmL-HY1-gate.log`), with no seed record in the checkout:

```
### fmt                 cargo fmt --all --check                                         fmt exit 0
### clippy              cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 39.40s                              clippy exit 0
### clippy ext          ... --features vibegraph/extended-validation -- -D warnings
    Finished `dev` profile [optimized] target(s) in 37.72s                              clippy-ext exit 0
### clippy all-features ... --all-features -- -D warnings
    Finished `dev` profile [optimized] target(s) in 45.57s                              clippy-all exit 0
### test                cargo test --workspace                                          test exit 0
47 suites, 1650 passed, 0 failed, 19 ignored
### banked target touched: cargo test -p vibegraph-lib --features extended-validation --test validate_hadronic -- sigma_mixed lepton_part --nocapture
[pp_to_ll_etau] AWAITING BUNDLE (the manifest marks this row bundled = false and this checkout has no seed record for dy13_etau_default; the split into two parts is checked, no cell is written)
test result: ok. 3 passed; 0 failed
### cargo doc -p vibegraph-lib --no-deps --document-private-items
warning: `vibegraph-lib` (lib doc) generated 8 warnings   (all in coupling/, phasespace/channel.rs:774, ufo/parameters.rs, lhef/mod.rs: none in a file this PR touches)
### cargo doc -p vibegraph-validation-report --no-deps --document-private-items   no warnings
### cargo doc -p vibegraph-lib --no-deps (public)   6 warnings, none about split_by_outgoing_masses
### python3 scripts/kb.py index / lint
wrote 1 index files
918 concepts, 295 backlog items; 0 errors, 4 warnings   (existing `state: blocked` items without `blocked_by`)
```

Suite count reconciliation:
- F1's tip counted 1648 passed.
- V1 added one hermetic test, the collator test. At `e935ca3` it failed (H1), so the tip read 1648 passed and 1 failed.
- This session adds one, `multiplicity::tests::a_sum_refuses_two_parts_of_one_phase_space`, and deletes none. That makes 1650.

The gated cells were not re-measured here: the seed record exists only on the M3.

## Method

- **Running the PR's own gates first: about 15%.** This found H1 and H2, both red at the branch tip. The unplan commit `ee6f18c` changed a manifest state that two PR-local tests read.
- **Diffing the new gate against its neighbour (`check_dy_run` / `adapted_integrand`): about 20%.** This found H4 and H5, and H13 for Found.
- **Grepping the diff's vocabulary ("multiplicities") across the crate, the book and the kb: about 20%.** This found H11, H12 and H15.
- **Testing each new assertion and doc claim against a test or a measurement: about 25%.** This found H6 and H7.
- **The backlog path-tail check: about 10%.** No filed item is made wrong by this PR, except the stochastic-rounding item (Found 5). The `validation-test-docs-glued-and-vegas-claim` item's "near line 731" now sits at line 1190; the drift is noted, and the item is still valid.
- **Leads that did not hold:**
  - a second production grouping path (there is none);
  - a mass tolerance mismatch between the split and the decomposition (both are exact, on the same mass source).
  - a rustdoc `private_intra_doc_links` warning after narrowing `split_by_outgoing_masses`: the plain `cargo doc` shows none.

## Found

1. **Four copies of the `MultiplicitySum` construction.**
   - Where: `vibegraph-cli/src/integrate.rs:871-895`, `vibegraph-lib/src/generate.rs:844-865`, `vibegraph-lib/tests/validate_hadronic.rs` `mixed_mass_runs` (this PR), and the matched-sample gate near `validate_hadronic.rs:5397`. Each builds per-part `ProtonIntegrand::new_with_maps` with on-shell vetoes and run-card scales, then calls `MultiplicitySum::new`.
   - Closed by: a library constructor on `MultiplicityGroups`, which the four call.
2. **The σ-gate row fill and pull are written out by hand per gate.**
   - Where: the `IntegralsRow` field-by-field fill, 8 times in `validate_hadronic.rs` and 2 in `validate_sigma.rs`, and pulls computed inline (`check_dy_run`) beside `SeedMean::pull`.
   - Closed by: `IntegralsRow` taking a `SeedMean`, a reference and the gate's bounds.
3. **`research/kb/hadronic/flavour-groups.md` rule 2 (lines 41-44)** still presents `UnequalFinalMasses` as the card's fate.
   - Closed by: one sentence saying a card is split by outgoing masses first (`multiplicity::split_by_outgoing_masses`), so the refusal is the decomposition's invariant. F1's Found 2 (MadGraph keys on the mass parameter's name, vibegraph on its value) belongs in the same paragraph.
4. **`research/kb/phase-space/mixed-multiplicity-integrand.md`** does not describe the split.
   - Lines 25-33 describe parts as one per multiplicity, with "`part_results` gives σ per multiplicity".
   - Line 108 describes `MultiplicityChannel`'s `group` without the offset across mass parts.
   - Closed by: adding the split, the non-decreasing order with distinct mass sets, and the group offset.
5. **Backlog `stochastic-rounding-refuses-mixed-multiplicity`** describes only `@N` multiplicity parts. A mixed-mass card such as `p p > l+ l-` with `ta` has two parts under one `@0`, so its `closes_when` ("each `@N` part's event count follows its integrated σ") does not cover it.
   - Closed by: restating it per part.
## Brief corrections

- **The tip moved.** The brief says the branch is "at `e935ca3` (pushed)". The remote moved to `3da3ed9` mid-session. Its `5ee2e83` fixed H1 only in part: the planned half kept `bundled = false`.
- **The gate list omits the banked target the PR adds.** The brief's gate has no `--features extended-validation` test run, and that run is the only one that shows H2.
- **H2 is a real bug, not hygiene.** The brief frames the planned-row handling as a consolidation question. On the branch tip it was a failure: the gate failed on every checkout without the record.
