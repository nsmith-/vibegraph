---
type: Session Report
title: "G-HY1 report: hygiene review of the v0.2 refusals PR"
description: "Fifteen findings on #88's change. Three tests shown vacuous by run mutations and tightened: the Lorentz C census read the submodule from the hermetic layer and skipped a missing directory; thirteen reweight-table needles were words of the row's own example, which every refusal quotes; the UFO field tests accepted a missing field reported as malformed and the reverse. Ten removal mutations show the remaining card-refusal needle tests are not vacuous. Docs, one test-only wrapper, one pub and two duplicate readers fixed; the four Several-joining copies, two scan predicates and unnameable field errors go to Found."
status: draft
tags: [hygiene, refusal, non-vacuity, mutation-testing, layer-registration, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-11-04, shared with sessions G-V1 and G-V2; cargo --jobs 1, CARGO_INCREMENTAL=0, debug info off"
  commit: "b91f0a1 on sprintg-refusals-hy1; base bbe993f"
  command: "see Gate and Mutations; logs under the session scratchpad, prefixed G-HY1-"
sources:
  - {id: brief, resource: "G-HY1.md", title: "G-HY1 brief"}
  - {id: procedure, resource: "../../../workflow/hygiene-review.md", title: "Running a hygiene review"}
---
Branch `sprintg-refusals-hy1`, worktree `/home/user/wt/G1-HY1`, base
`bbe993f` (all five feature sessions merged). Scope: `git diff
86b8e5c...bbe993f`, code and tests. The findings table was written before the
first edit; dispositions are final.

## Findings

| id | pt | site | evidence | conf | disposition |
|---|---|---|---|---|---|
| H1 | 2 | `helas/eval/root_lorentz.rs` `in_tree_models_never_reach_the_charge_conjugation_refusal` | read `research/refs/mg5amcnlo/models` from the hermetic `--lib` layer, `let Ok(entries) = read_dir(..) else { continue }`; M13 (path pointed at a missing directory) passes | checked | **fixed** `cd2db3f` |
| H2 | 2 | `reweight/classes.rs` 13 rows; `ufo/card_classes.rs` 4 rows | `reweight::card::CardError::{Unsupported, Malformed}` print `text`, the card line, and the needles (`mode`, `model`, `scan`, …) are words of that line; M1 and M2 pass | checked | **fixed** `42c749f` |
| H3 | 2 | `tests/ufo_input_refusals.rs` `assert_names`; `tests/input_card_refusals.rs` `every_required_propagator_field_is_refused_by_name` | both `FieldError::Missing` and `::Malformed` print `'<object>'` and `` `<field>` ``; M3 passes all 44 tests | checked | **fixed** `41e25e4` |
| H4 | 3, 1 | `ufo/bind.rs:306` `check_merged` | `#[cfg(test)]` one-line wrapper over `merge_splits`, carrying the merge contract's doc; the production item had a one-line doc | checked | **fixed** `b91f0a1` |
| H5 | 1 | `ufo/parameters.rs:294` | production doc of `parse_parameters_with_variables` links `parse_parameters`, which is `#[cfg(test)]` | checked | **fixed** `f63f23f` |
| H6 | 1 | `reweight/mod.rs:255` | `check_merged` was inserted under `external_name`'s doc line: its doc began "The external parameter a change addresses." and `external_name` had none | checked | **fixed** `b91f0a1` |
| H7 | 1 | `ufo/mod.rs` `apply_restriction` | doc named only the `Auto` refusal; the body also refuses a missing value, MadGraph's free one/zero and a non-finite value | checked | **fixed** `b91f0a1` |
| H8 | 1 | `ufo/bind.rs` `bind_card` `# Errors` | omitted `NegativeMass`, which is checked only after every entry binds, so it is not listed with them | checked | **fixed** `b91f0a1` |
| H9 | 3 | `ufo/slha.rs` `ParamCard::decay_tables` | `pub`; readers `ufo/bind.rs`, `ufo/card_classes.rs` only (`grep -rn 'decay_tables()'`) | checked | **fixed** `b91f0a1` (`pub(crate)`) |
| H10 | 3, 4 | `support.rs` `PointCensus::merged`; the folds in `hadronic.rs`, `proton.rs`, `multiplicity.rs` | `merged` `pub`, readers all in the crate; the same fold three times | checked | **fixed** `f63f23f` (`impl Sum`) |
| H11 | 4 | `ufo/lorentz.rs` `extract_spins`, `ufo/parameters.rs` `extract_lhacode` | identical bodies, field name aside | checked | **fixed** `f63f23f` (`Fields::int32_list`) |
| H12 | 4 | `ufo/slha.rs:33,198`, `ufo/bind.rs:150,159`, `ufo/mod.rs:240,501`, `reweight/card.rs:134,272` | four copies of a `join("; ")` formatter and the `0 / 1 / Several` match | checked | **Found** |
| H13 | 1, 4 | `ufo/slha.rs:225`, `reweight/card.rs:357` | two predicates for MadGraph's `scan` marker: any token starting `scan`, against `scan[digits]:` | checked | **Found** |
| H14 | 2 | `tests/input_card_refusals.rs` param and restrict cases | the brief's "pin the variant": the needles read text where `CardError`/`UfoError::Restrict*` are nameable | checked | **leave**: M4–M12 each fail the needle test that names the refusal, so the needles are not vacuous; pinning would change wording-robustness, not power |
| H15 | 3 | `ufo/ast_util.rs` `FieldError`; `CouplingError`, `VertexError`, `PropagatorError` | `pub` types in `pub(crate)` modules, carried by the `pub` `UfoError`: integration tests cannot name them, so the field tests can only read text | checked | **Found** |

Also checked and left: the `unreachable!` arms (`hadronic.rs` `spin_state_count`,
`analysis.rs` `external_out_type`, `run.rs` `build_external_core`) take a compiled
`AmplitudeEvaluator`'s particles, whose spins `refuse_unsupported_spins` has
checked, so the invariant holds by construction; the `from_model_card(..).unwrap()`
sites are all in test code, where a panic printing the `CardError` is the right
failure. `MIN_EFFECTIVE_POINTS` is read only in `support.rs` but the public
`check_support` doc names it as the contract: left `pub`. The `#[cfg(test)]`
tables (`PARAM_CARD`, `RESTRICT_CARD`, `REWEIGHT_CARD`) beside the driving
`FIELD_CLASSES`: the split is justified, since the param, restrict and reweight
refusals live in the readers' code paths and a table driving them would mean
restructuring the readers; what each table can then check is the examples, and
H2 is where that check was weakest.

Backlog: of the filed items naming a reviewed path, none has a site this change
makes wrong. `restrict-card-unit-values-not-fixed` describes a gap the code
shows already closed (Found 4).

## Fixed

- **H1** (`cd2db3f`). `check_c_free_models_under(parent)` lists `parent`,
  panicking if it cannot, and returns the models checked and the ones the
  loader refused. `authored_models_never_reach_the_charge_conjugation_refusal`
  (hermetic) runs it on `validation/ufo` and requires `vibegraph_toy_UFO`
  among the models and nothing refused;
  `shipped_models_never_reach_the_charge_conjugation_refusal`
  (`#[cfg(feature = "extended-validation")]`) runs it on the submodule and
  requires `sm`, with only `taudecay_UFO` refused.
- **H2** (`42c749f`). `assert_rows` refuses a needle the example text contains,
  for all three tables. Thirteen reweight needles became their refusal's reason
  (`"mode"` → `"only the leading-order ratio is computed"`, `"scan"` → `"write
  one `launch` block per value"`, …); the param and restrict `scan` and `Auto`
  rows read `"runs once per value"` and `"computes at launch"`. No other row
  tripped the guard.
- **H3** (`41e25e4`). `assert_names` takes a `Form` (`Missing`: `'S' has no
  `spin``; `Malformed`: `'S': `spin` must be`); mixed loops derive it from the
  edit (`Form::of`: malformed when the replacement still assigns the field).
  The propagator test asserts the same two wordings.
- **H4–H9** (`f63f23f`, `b91f0a1`): docs and the one narrowing, as in the table.
- **H10–H11** (`f63f23f`). Identity oracle chosen before editing: the census
  test (`proton::tests::the_census_separates_cut_rejections_from_zero_weights`),
  the CLI support-refusal tests (exact counts in their messages), and every UFO
  field test (`a list of integers` is the message of both old readers); all
  pass unchanged.

## Mutations

Driver `G-HY1-mutate.py`: apply, `cargo test --jobs 1 <target>`, restore from
the saved text, log. Old tests = `bbe993f`; new = `b91f0a1`'s tests.

| id | mutation | old tests | new tests |
|---|---|---|---|
| M1 / N1 | delete the `"mode" =>` arm of `reweight::card::change` | PASS (`every_reweight_card_construct_behaves_as_classified` ok) | FAIL |
| M2 / N2 | `if false && tokens[1..].iter().any(is_scan_value)` in the reweight `set` reader | PASS | FAIL |
| N2b | restore the echoed needle `Error("mode")` | — | FAIL (the guard) |
| M13 | old census: submodule path → `models_absent` | PASS | — |
| N4 | new extended test: submodule path → `models_absent` | — | FAIL (`shipped_…` panics listing it) |
| N5 | new hermetic test: `validation/ufo` → `validation/ufo_absent` | (old: same `continue`, then only the toy assert) | FAIL |
| M3 / N3 | `Fields::required` returns `malformed(field, "present")` | PASS: 24 + 20 ok | FAIL: `every_required_propagator_field_is_refused_by_name`; and 7 of 20 in `ufo_input_refusals` (particle, parameter, coupling, vertex, Lorentz, coupling-order and unnamed-vertex cases) |
| N3b | `Fields::str` returns `Missing` for a non-string | (old: passes by M3's argument, the two forms share the needles) | FAIL: `a_malformed_particle_field…`, `a_malformed_parameter_field…`, `every_required_coupling_field…` |
| M4 | `negative_masses` returns no error | FAIL: `a_negative_mass_off_a_majorana_fermion_is_refused`, `a_negative_default_or_restricted_mass_is_refused_at_binding` | — |
| M5 | `unwritable_tables` returns no error | FAIL: `a_decay_table_under_a_negative_code_is_refused` | — |
| M6 | `RestrictMissing` not pushed | FAIL: `a_restrict_card_missing_a_parameter_is_refused_naming_it` | — |
| M7 | `RestrictMadgraphFree` not pushed | FAIL: `madgraphs_unrestricted_one_and_zero_are_refused` | — |
| M8 | `RestrictNonFinite` not pushed | FAIL: `a_non_finite_restrict_value_is_refused` | — |
| M9 | `RestrictParamSibling` check off | FAIL: `a_param_card_beside_the_restrict_card_is_refused` | — |
| M10 | `SlhaError::Several` → first error | FAIL: `every_problem_in_a_card_is_reported_at_once` | — |
| M11 | `bind::CardError::Several` → first error | FAIL: same | — |
| M12 | `reweight::card::CardError::Several` → first error | FAIL: `every_problem_in_a_reweight_card_is_reported_at_once` | — |

M4–M12 are H14's evidence and needed no change.

**What the tightened tests still cannot see.** The reweight and card tables
check that a refusal fires with its reason; a construct missing from a table, or
a class whose example happens to behave as written, still passes. `Form::of`
reads the expected form from the edit, so a fixture edit that both drops and
re-adds a field would be classified wrongly (none does). The C census proves no
in-tree structure carries `C` and that rooting does not panic; it cannot see a
wrong tree. The authored-model test cannot see a model directory deleted from
`validation/ufo` other than the toy one.

## Gate

| command | result (tail) |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, `Finished` |
| `cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings` | exit 0, `Finished` |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0, `Finished` |
| `cargo test --workspace` | exit 0; 1883 passed, 0 failed, 25 ignored, summed over every `test result` line |
| `cargo doc -p vibegraph-lib --no-deps --document-private-items` | exit 0, 7 warnings, all in files this change does not touch (`coupling/cluster/configs.rs`, `phasespace/channel.rs`, `coupling/alphas.rs`, `coupling/scales.rs` ×2, `lhef/mod.rs` ×2) |
| `cargo test -p vibegraph-lib --features extended-validation --lib -- charge_conjugation_refusal` | 2 passed (the hermetic and the shipped census) |
| `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test ufo` | 5 passed, 1 ignored (the target the manager moved the MSSM_SLHA2 test into) |
| hermetic layer as CI runs it: `cargo test --workspace` in a scratch clone of `b91f0a1` with no `research/refs/mg5amcnlo` content and no fetched data | exit 0; 1883 passed, 0 failed, 25 ignored, the same count as the worktree's run; `authored_models_never_reach_the_charge_conjugation_refusal` ran and passed |
| `python3 scripts/kb.py lint` | 0 errors, 5 warnings (all on backlog items this session does not touch) |

Suite count: one hermetic test deleted
(`in_tree_models_never_reach_the_charge_conjugation_refusal`), one added
(`authored_models_never_reach_the_charge_conjugation_refusal`), and one added to
the extended layer (`shipped_models_never_reach_the_charge_conjugation_refusal`):
the hermetic count is unchanged.

The brief's banked list (`amplitude_oracle`, `standalone_jamps`,
`input_card_refusals`, `cli_support_refusals`, `cli_param_card`) is registered in
the hermetic layer and ran inside `cargo test --workspace`: H10 touches
`integrate`'s census, H4/H8/H9 the card binding (docs and visibility only).

## Method

- Reading the diff against the reports' "cannot detect" sections and the brief's
  attention points, about 35%: produced H1, H2 (G-F3 had named the echo blind
  spot itself), H4–H8.
- Grep of each new `pub` item's readers, about 10%: H9, H10, H15.
- Grep of `read_dir`/`continue`/submodule paths in the diff's tests, about 5%:
  H1, and confirmed no other hermetic test reads the submodule or fetched data
  (the toy and CLI tests read only tracked `validation/ufo` and
  `validation/madgraph/*.json`).
- Mutation runs, about 40% of wall time: 13 on the old tests, 7 on the new.
  The removal batch (M4–M12) is what decided H14: the brief's "pin the variant"
  lead did not hold up as a power problem, while the "which refusal fired" lead
  did, in two shapes the brief had not named (echoed needles, two error forms
  sharing a needle).
- Grep for duplicate bodies (`join("; ")`, the list-of-integers reader, scan
  detection), about 10%: H11–H13.

## Found

1. **Four copies of the several-refusals plumbing.** `ufo/slha.rs` (`several`,
   L33; match L198), `ufo/bind.rs` (`several` L150, `refuse_all` L159),
   `ufo/mod.rs` (`several` L240; match L501), `reweight/card.rs` (`several`
   L134; match L272). Close: one generic `fn several<E: Display>(&[E]) ->
   String` and `fn refuse_all<E>(Vec<E>, impl FnOnce(Vec<E>) -> E) ->
   Result<(), E>` in a shared module, which this crate has none of yet.
2. **Two scan-marker predicates.** `ufo/slha.rs:225` refuses any token starting
   `scan`; `reweight/card.rs:357` (`is_scan_value`) requires `scan[digits]:`.
   Both refuse every scan MadGraph writes, but they disagree on what one is.
   Close: one predicate, with the rule cited from MadGraph's reader.
3. **Field errors cannot be named outside the crate.** `FieldError` is `pub` in
   the `pub(crate)` `ufo::ast_util`, and `CouplingError`, `VertexError` and
   `PropagatorError` are `pub` in `pub(crate)` modules, yet `pub` `UfoError`
   variants carry them. Integration tests can only match their text (H3 pins
   the text's form for that reason). Close: make the error types nameable
   (a `pub` module) so a test can match `FieldError::Missing { field, .. }`.
4. **Backlog item `restrict-card-unit-values-not-fixed` is stale.** It says the
   loader locks only zeros (`ufo/parameters.rs:115`, `:125`); `ParameterSet`
   has had a `ones` set since #82 (`3fbb177`), `bind_card` refuses moving one
   (`CardError::MovesFixedOne`), and `bind.rs`'s
   `only_a_restricted_value_of_exactly_one_is_fixed` pins it. Close: delete the
   item, or re-scope it to whatever remains.
5. **Reweight `check_merged` is stricter than the card's.** A hypothesis moving
   one member of a merged group to the value it already has is refused, because
   every member must be listed (`reweight/mod.rs` `check_merged`: an unlisted
   member is `false`), while `bind_card` compares values. Harmless (the user
   lists both), but the two rules disagree. Close: compare against the run's
   value for an unlisted member, or say in the refusal that every member must
   be listed.

## Brief corrections

- "the submodule's models move to an `extended-validation` target": the test
  drives private `LorentzEvalTree::build_at_leg` and the module's own `flow_of`
  helper, so it cannot leave the `--lib` target. It is gated with
  `#[cfg(feature = "extended-validation")]`, the mechanism `ci.yml` names for
  library unit tests with banked inputs.
- The brief's "where a variant exists, pin it" assumed needle tests were
  vacuous where variants exist. For the param and restrict cases the removal
  mutations (M4–M12) show they are not; the vacuous needle tests were the ones
  whose needle the example or a sibling refusal also prints (H2, H3).
