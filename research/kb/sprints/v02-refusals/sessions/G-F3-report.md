---
type: Session Report
title: "G-F3 report: param, restrict and reweight cards classified; propagators.py fields"
description: "Three classification tables (25 param, 24 restrict, 45 reweight rows), each held to its class by a test running every row's example through the real reader; 15 new refusals and 6 acceptances, each failing on the pre-change code; propagators.py name/numerator required."
status: draft
tags: [param-card, restrict-card, reweight, classification, refusal, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Branch `sprintg-refusals-f3`, worktree `/home/user/wt/G1-F3`, base f052fb9.
Implementation commit `5fcc4ee`; this report is committed on top of it.

## What landed

1. **Classification tables** (`#[cfg(test)]` specifications):
   `ufo/card_classes.rs` `PARAM_CARD` (25 rows) and `RESTRICT_CARD` (24 rows),
   `reweight/classes.rs` `REWEIGHT_CARD` (45 rows). Each row: construct,
   what MadGraph does (file and line at the pin), class (`Consumed`,
   `IgnoredBenign` with argument, `Refused` with why), and an example with an
   expectation. The tests `every_{param,restrict,reweight}_card_construct_behaves_as_classified`
   run each example through the real reader and consumer (`bind_card`;
   `ParsedModel::into_model` on the interned SM; `ReweightCard::from_str` +
   `resolve`) and hold it to the class: benign = same result as the baseline
   (bit-for-bit parameter values and moved list; vertex couplings, external
   defaults, zero/one/merged sets; resolved launches), consumed = different,
   refused = error containing a named needle. Two rows (`param_<x>.dat`,
   external mass in reweight) are `PinnedBy` a named test because they are not
   card text.
2. **Concepts**: `model/param-card-classification.md`,
   `model/restrict-card-classification.md`,
   `events/reweight-card-classification.md`, linked from
   `pipeline/release-scope.md`'s table; `restriction-semantics.md` and
   `ufo-parsing.md` updated.
3. **New refusals** (all reported at once: `SlhaError::Several`,
   `CardError::Several`, `UfoError::Several`, reweight `CardError::Several`):
   - param card: decay table (`CardError::DecayTable`), negative mass
     (`NegativeMass`), `scan:` value, fields after a `DECAY` width;
   - restrict card: missing external parameter (`RestrictMissing`),
     `9.999999e-1`/`0.000001e-99` (`RestrictMadgraphFree`), non-finite
     (`RestrictNonFinite`), `param_<x>.dat` beside `restrict_<x>.dat`
     (`RestrictParamSibling`);
   - reweight: a command after `done`/`exit`/`quit` (`AfterEnd`), a parameter
     fixed to one (`FixedOne`), a merged group split (`SplitsMerged`).
4. **Relaxations, each argued**: `XSECTION` rows are skipped up to the next
   header as MadGraph does (they were a parse error after a block and silently
   ignored after a DECAY); `change helicity False`, `change output default`,
   `change rwgt_dir`, `change multicore` are accepted.
5. **`propagators.py`** (G-F2 Found): `Propagator.__init__(self, name, numerator,
   denominator=None)` in SMEFTsim's `object_library.py` L364-L370. `name` and
   `numerator` are required via `Fields`; `denominator` is optional
   (`Option<String>`, `None` = ALOHA's standard denominator); non-foldable forms
   and duplicate names are refused naming propagator and field.

## Brief corrections

- MadGraph's `require_args` for `Propagator` lists `denominator`, but the
  constructor defaults it to `None`; G-F2's rule (required = positional without
  default) makes it optional. The loader previously *refused* a missing
  denominator.
- "XSECTION … everything after it" is not MadGraph's behaviour: `ParamCard.read`
  tests `block`/`decay` before `xsection` (L356, L362, L374), so a header ends
  the XSECTION rows. Classified benign, not refused.
- The reweight card was not much larger than the other two; all three are done.

## Gates (this worktree, `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, `--jobs 2`)

| Command | Tail |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets --jobs 2 -- -D warnings` | `Finished`, exit 0 |
| `cargo clippy --workspace --all-targets --features vibegraph/extended-validation --jobs 2 -- -D warnings` | `Finished`, exit 0 |
| `cargo test --workspace --jobs 2` | exit 0; 52 `test result` lines, summed 1855 passed, 0 failed (includes the three `*_behaves_as_classified`, `input_card_refusals` 20/20, `ufo_input_refusals` 20/20, `smeftsim` 13/13, `toy_models` 17/17) |
| `cargo test -p vibegraph-lib --features extended-validation --test sm_interned_blob` (check-sm-blob-fresh) | `ok. 2 passed; 0 failed` |
| `cargo test -p vibegraph-lib --features extended-validation --test ufo` (loop_sm, MSSM_SLHA2, taudecay, SM from source: banked cards load) | `ok. 4 passed; 0 failed; 1 ignored` |
| `cargo test -p vibegraph-lib --test coupling_oracle -- --nocapture` (validate-couplings) | `ok. 50 passed; 0 failed` |
| `cargo test -p vibegraph-lib --test reweight_mg_oracle -- --nocapture` (validate-reweight, hermetic) | `ok. 1 passed; 0 failed` |
| `cargo test -p vibegraph --profile release-debug --features extended-validation --test cli_reweight_proton -- --nocapture --test-threads 1` (validate-reweight, banked audit) | `ok. 2 passed; 0 failed` (183.5 s) |
| `python3 scripts/kb.py lint` | `1057 concepts, 337 backlog items; 0 errors, 5 warnings` (the five pre-existing `state: blocked` warnings) |

pixi is not installed; the tasks were run as the commands they wrap. Their
MadGraph dependency steps (`generate-couplings`, `generate-reweight-oracle`)
need MadGraph and did not run; the gates read the banked references. The first
gate run (before a container restart killed it) found two compile/lint breaks,
fixed before the run above: `tests/smeftsim.rs` read `denominator` as a
`String`, and a clippy `type_complexity` in a test helper.

Banked cards: no banked param card has a decay table, negative mass, scan or
extra DECAY field; no banked restrict card misses an external, uses MadGraph's
unfixed one/zero, or has a `param_<x>.dat` sibling (census script
`G-F3-scripts/restrict_census.py` over the 56 restrict cards: missing 0 on
every card). The two fixture cards in `ufo_input_refusals.rs` that omitted
`MS`/`WS` were completed.

## Mutation evidence

`vibegraph-lib/tests/input_card_refusals.rs` (20 tests, public API only) was
run on the pre-change source (`git stash push -- vibegraph-lib/src`, at f052fb9):

```
test result: FAILED. 2 passed; 18 failed
```

The two passes are the controls `the_fixture_loads` and
`a_restrict_card_decay_table_is_accepted` (benign before and after). Each
failure, as the pre-change code behaved:

| test | pre-change |
|---|---|
| `a_negative_mass_is_refused` | `"Block MASS\n 6 -1.75e2\n" binds` |
| `a_decay_table_is_refused_naming_the_particle` | binds |
| `a_decay_line_with_extra_fields_is_refused` | `"DECAY 6 1.5 0.3\n" binds` |
| `a_scan_value_is_refused` | `Parse error on line 2: expected float, got '175]'` (refused, not named) |
| `an_xsection_block_is_skipped_up_to_the_next_block` | `expected an integer code, got 'XSECTION'` |
| `every_problem_in_a_card_is_reported_at_once` | only line 2 reported |
| `a_restrict_card_missing_a_parameter_is_refused_naming_it` | loads |
| `madgraphs_unrestricted_one_and_zero_are_refused` | `9.999999e-01` loads |
| `a_non_finite_restrict_value_is_refused` | `nan` loads |
| `a_param_card_beside_the_restrict_card_is_refused` | loads beside param_test.dat |
| `reweighting_a_parameter_fixed_to_one_is_refused` | `unwrap_err()` on `Ok(1)` |
| `reweighting_one_parameter_of_a_merged_pair_is_refused` | `unwrap_err()` on `Ok(1)` |
| `a_command_after_done_is_refused` | `unwrap_err()` on `Ok(1)` (second launch dropped) |
| `inert_change_options_are_accepted` | `change helicity False` refused |
| `every_problem_in_a_reweight_card_is_reported_at_once` | only line 1 reported |
| `every_required_propagator_field_is_refused_by_name` | `name = 'S1'` removed: loads |
| `a_propagator_without_a_denominator_loads` | `Propagators(UnreadableForm { name: "S1", field: "denominator" })` |
| `a_propagator_named_twice_is_refused` | loads |

The restrict needle of `madgraphs_unrestricted_one_and_zero_are_refused` was
changed after this run (from the value text to "leaves free"); the pre-change
failure is the "loads" panic, before any needle is read.

The classification test was mutated once on the post-change code: the
`DecayTable` push in `bind_card` replaced by `let _ = card.decay_tables();`:

```
test ufo::card_classes::tests::every_param_card_construct_behaves_as_classified ... FAILED
a decay table (branching-ratio rows after `DECAY`): accepted
```

`bind.rs` was restored from a copy afterwards (`git diff` then showed only the
intended change).

**What the tests cannot detect.** The tables are written by hand from
MadGraph's source: a construct missing from a table, or a wrong class whose
example happens to behave as written, passes. Refused rows match a needle in the
message; for reweight `change` rows the needle is often in the echoed line, so
those rows prove a refusal, not its reason. The param and reweight rows run on
a test model (SM with `MT`/`ymt` merged and `WT` fixed at one, since no SM
restriction does either), not on a model whose restriction made those sets.
The restrict "benign" comparison covers vertex couplings, external defaults
and the zero/one/merged sets, not internal parameters.

## Notes for the manager

- **Behaviour changes visible to users**: a CMS gridpack's
  `Cards/param_card.dat` written by MadWidth carries decay tables; `integrate
  --param-card` now refuses it, while `CardError::AutoWidth`'s message still
  recommends that file. Removing the tables (the widths stay) makes it bind.
  Carrying the tables into the `<slha>` header would make the construct
  consumed (Found 1).
- **Reweight helicity**: a card without `change helicity` asks MadGraph for the
  per-helicity ratio; `generate` writes the summed one. Left accepted per the
  existing `reweight-helicity-summed-only` item; whether to refuse such cards is
  the manager's/user's call.
- `card_classes.rs` and `reweight/classes.rs` are `#[cfg(test)]`: tables that
  specify, not drive, behaviour (the run card's drives its refusal).

## Found

1. **Decay tables could be carried, not refused.** `lhef::write::slha_block`
   writes the `<slha>` header from the model; copying the card's decay tables
   into it (as MadGraph does) would turn the refusal into Consumed and unblock
   MadWidth-written gridpack cards. Evidence: `check_param_card.py` L385-L393,
   `CardError::DecayTable`.
2. **Negative masses are evaluated with the wrong width sign.** MadGraph's
   `write_inc_file` (L692-L694) negates a width whose card mass is negative;
   `helas` uses `C::new(q² − m², m·w)` as given. `MSSM_SLHA2`'s
   `restrict_default` has `Mneu3 = -363.756`, width 1.916, so every MSSM
   neutralino-3 propagator has the wrong-sign imaginary part today. The param
   card refuses negative masses; the model default is not refused. Close: apply
   MadGraph's rule in `EvaluatedModel` (width sign follows mass sign), pinned
   against an MSSM amplitude.
3. **`running.py` models are not refused.** MadGraph runs `running_elements`
   (block scales `Q=`, `mdl__<block>__scale`); the loader ignores `running.py`
   silently. No banked model ships one. Close: refuse a model directory with
   `running.py`, which also makes the `Q=` row's benign argument unconditional.
4. **QNUMBERS disagreeing with the model.** A param card's `QNUMBERS` block is
   ignored and the header writes the model's; MadGraph would echo the card's.
   Close: refuse a `QNUMBERS` block that disagrees with the model's particle.
5. **Optional UFO files read with `Err(_) => default`.** `ufo/mod.rs` treats any
   read error of `coupling_orders.py`/`propagators.py` (permissions, invalid
   UTF-8) as absence. Close: default only on `NotFound`.
6. **Restriction conditional simplification not checked.** MadGraph simplifies
   `if` expressions at the restriction's values
   (`detect_conditional_statements_simplifications`); whether a later param
   card can flip a condition MadGraph froze was not audited.
