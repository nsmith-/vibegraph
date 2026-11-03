---
type: Session Report
title: "P-F1 report: integrate --param-card and the recorded parameter point"
description: "integrate --param-card binds through ufo::bind::bind_card and records the moved externals; generate rebinds from the record (bit-identical to integrate's evaluation); reweight denominators at the recorded point; refusals for Auto widths, revived zeros, split merges and unknown entries; dependent masses ignored with a warning; the <slha> header block in MadGraph's layout; ymt = 250 informational σ."
status: draft
tags: [cli, param-card, slha, artifact, reweight, report, sprint-e]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
---
Worktree `/home/user/wt/P`, branch `sprinte-param-card`, from 511b45b8 (main
f5fb3e65 plus the sprint folder). Every cargo command ran with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`; the
release-debug gates also with `--config 'profile.release-debug.debug=0'`.
Pixi is not installed, so each pixi task's command ran directly.

Commits: `02423b2b` (code, tests, docs, concepts), then this report.

## What changed

**Binding a card** (`vibegraph-lib/src/ufo/bind.rs`, new). `bind_card(model,
card) -> Result<BoundCard, CardError>` wraps `EvaluatedModel::from_model_card`
with the checks, and returns the evaluation, the moved parameters
(`Vec<artifact::MovedParam>`) and the ignored entries. In order:

1. `DECAY <pdg> Auto` → `CardError::AutoWidth` (P1). The message says to give
   the width or use the `param_card.dat` MadGraph wrote for the run, and that
   computing Auto widths is not supported yet.
2. Each card entry, sorted by block then code:
   - a non-finite value → `NonFinite`;
   - an external of the model: a non-zero value for one the restriction
     zeroed → `RevivesZeroed`; otherwise accepted;
   - block `QNUMBERS` → skipped (MadGraph writes these for new states);
   - `MASS <pdg>` / `DECAY <pdg>` of a particle the model defines, whose mass
     or width is not an external at that code (internal such as `MW`, `ZERO`,
     or another particle's parameter) → ignored (P2), and reported in
     `BoundCard::ignored` when it differs from the computed value by more than
     `1e-6` relative (MadGraph writes these entries with `%e`, within `5e-7`);
   - anything else → `Unknown`. `from_model_card` itself ignores such entries
     silently, so this refusal is new.
3. `moved`: every external whose bound value differs from the restricted
   default, in model external order, at the card's `f64`.
4. The merge check (P4), below.

`bind_recorded(model, &[MovedParam])` rebuilds a `ParamCard` from the record
(`ParamCard::insert`, new) and calls `from_model_card`; a recorded parameter
that is not a free external at the recorded block and code is
`NotRecordable`. The CLI warns once per ignored entry naming the block, PDG
code, parameter, the card's value and the computed one.

**ParamCard** (`ufo/slha.rs`): `DECAY <pdg> Auto` (any case) is recorded in
`auto_widths()` and sets no value (it was dropped silently); `insert` and a
sorted `entries()` are new.

**Merged parameters** (`ufo/parameters.rs`). MadGraph's restriction
(`import_ufo.py` `RestrictModel.restrict_model` →
`detect_identical_parameters` → `merge_iden_parameters`) groups the externals
whose restricted value is the same, or its negative, within one LHA block,
skipping values 0 and 1 and the `DECAY` block. It keeps the first of each
group as the external and rewrites the rest as internal `±1 × first`, records
an `identical`/`opposite` rule in the process's `param_card_rule.dat`, and at
launch `check_param_card` either refuses a card that breaks the rule or (with
`modify`) resets the member to the first's value. vibegraph had no
representation of this: every external stayed independent. It now computes the
groups the same way in `ParameterSet::apply_restrict` into
`ParameterSet::merged` (`#[serde(skip)]`: derived from the restricted
defaults, which the digest already pins, so the model digest and the interned
blob are unchanged; `sm_interned_blob` passes). `bind_card` refuses a card
whose bound values break `member == factor × first` (`SplitsMerged`); moving
the whole group consistently is accepted, as MadGraph accepts it. **No model in
the repository has a merged group**: the ten interned SM restrictions
(`the_interned_sm_merges_nothing`), SMEFTsim `massless` and `SMlimit_massless`,
and every toy UFO restriction (checked with a script mirroring
`detect_identical_parameters` over `parameters.py` and the restrict cards). The
refusal is therefore tested on a synthetic parameter set and on the SM with a
merge injected, not on a real model.

**Artifact** (`artifact.rs`). `ArtifactError::MovedParams` and
`RunInputs::refuse_moved_params` are gone; `params` round-trips
(`moved_parameters_round_trip`). No version change.

**integrate** (`vibegraph-cli/src/integrate.rs`). `--param-card <file>` binds
through `bind_card`; `inputs.params` is the moved list; the run log prints
`params:   MZ = 92.5` when non-empty.

**generate** (`vibegraph-cli/src/generate.rs`). Binds with
`bind_recorded(model, &artifact.inputs.params)` after the digest check, and
passes that evaluation to everything, the reweight plan included, so
hypothesis denominators and the per-event audit are at the recorded point. No
card option.

**`<slha>` header** (`lhef/write.rs` `slha_block`, written by both generate
paths, after `<MGRunCard>` when that is written and before
`<vibegraphInputs>`). Layout of MadGraph's `ParamCardWriter`
(`models/write_param_card.py`): free externals ordered by block with `DECAY`
last, then code; `Block <name> ` lines and the `INFORMATION FOR` headings;
`%3s` codes; a dependent section after `MASS` and after `DECAY` (particles
whose mass/width is not a free external, by PDG code, then duplicates); the
`aS` note; `QNUMBERS` blocks for physical particles outside MadGraph's SM PDG
list; the trailing whitespace of the last line trimmed as MadGraph's banner
does.

Compared with the `<slha>` of a banked MadGraph event file
(`output/ee_to_wpwm/Events/run_01/unweighted_events.lhe.gz`, whose content is
`validation/madgraph/dy13_param_card.dat` byte for byte apart from that
trailing space), `the_slha_block_has_madgraphs_layout` pins every data line
(spacing, value and comment up to ` : `), every heading and every `Block`
line. What differs:

- the banner (vibegraph's own two lines, not "GENERATED BY MG5");
- the dependent-section preamble (three lines of vibegraph's wording);
- a dependent entry's comment names the parameter (`# w+ : MW`) where
  MadGraph prints its expression;
- a value with more digits than `%e` keeps is written with the fewest digits
  that read back (`DECAY  25 6.38233934e-03 # WH` against MadGraph's
  `6.382339e-03`, see below);
- a merged member or a restriction-`1` parameter would still be written (none
  exists in any repository model); MadGraph removes them from its card.

## Brief corrections

- **MadGraph's own card is not the defaults.** MadGraph writes every value
  with `%e`; the SM restriction's Higgs width `6.38233934e-03` has two more
  digits, so MadGraph's `param_card.dat` (and every banked MadGraph run) is at
  `WH = 6.382339e-03`. `integrate --param-card` with MadGraph's own card
  therefore records one moved parameter, `WH`, and its artifact is not
  byte-identical to the no-card one. Pinned by
  `madgraphs_own_card_moves_only_the_width_it_rounds` and
  `madgraphs_own_card_records_the_width_it_rounds`. The byte-identity test uses
  a card that restates the defaults exactly, as the design §1 states it. The
  same rounding moves `WH` in any card P-V1 derives from MadGraph's.
- **Unknown blocks or codes**: `from_model_card` did not refuse them (it
  ignores whatever it does not look up), so the refusal was added, with the two
  exemptions MadGraph's own cards need (dependent `MASS`/`DECAY` entries and
  `QNUMBERS`).
- **P4 has no real-model test case** (above).

## Tests and mutation evidence

New tests: `ufo::bind::tests` (13), `ufo::slha::tests` (2 new),
`lhef::write::tests::the_slha_block_*` (2), `artifact::tests::moved_parameters_round_trip`,
and `vibegraph-cli/tests/cli_param_card.rs` (7, `e+ e- > mu+ mu-` at
`√s = 91.2 GeV`, `--fixed-budget --neval 8000 --niter 3`).

| Rule | Test | Mutation | Result |
|---|---|---|---|
| Auto width refused | `an_auto_width_is_refused`; CLI `cards_the_model_cannot_honour_are_refused` | `auto_widths().first().filter(\|_\| false)` | FAILED |
| revived zero refused | `reviving_a_zeroed_parameter_is_refused`; CLI same | `&& false` on the zero check | FAILED |
| split merge refused | `bind_card_refuses_a_split_merge` | `let _ = check_merged(..)` | FAILED |
| merge detection skips `DECAY` | `splitting_a_merged_pair_is_refused` | drop the `DECAY` exclusion | FAILED |
| unknown entry refused | `an_entry_the_model_lacks_is_refused`; CLI same | `None if true => {}` | FAILED |
| dependent value reported | `a_dependent_mass_is_ignored_and_reported`; CLI `a_dependent_mass_is_ignored_with_a_warning` | `(off && false).then_some` | FAILED |
| only moved externals recorded | `a_card_at_the_defaults_moves_nothing` (+6 others) | filter `b.value == b.value` | FAILED (7) |
| rebind is the record | `a_moved_parameter_is_recorded_and_rebinds_exactly` (bit-for-bit, every parameter and coupling) | `bind_recorded` ignores the record | FAILED |
| `<slha>` layout | both `the_slha_block_*` | no `MASS` dependent section | FAILED (2) |
| generate evaluates at the record | CLI `generate_evaluates_at_the_recorded_point`, `reweighting_takes_the_recorded_point_as_its_denominator` | `bind_recorded(model, &[])` in generate | FAILED: "sample σ 2053.484 ± 46.05331 against the artifact's 1003.3625 ± 2.6098"; "recorded ratio 0.44213717769981153" |
| integrate evaluates at the card | CLI `a_moved_parameter_is_recorded`, `generate_evaluates_at_the_recorded_point` | integrate binds `from_model` but records the moves | FAILED: "σ moved by 0"; "sample σ 1021.509 ± 22.88927 against the artifact's 2023.70 ± 5.23" |

Each mutation was applied to the committed text, the named test run, and the
file restored (diffed against a saved copy). Mutation runs, verbatim:

```
$ cargo test -p vibegraph-lib --lib -- ufo::bind          # auto, revive, merge-call, unknown, dependent
test ufo::bind::tests::a_dependent_mass_is_ignored_and_reported ... FAILED
test ufo::bind::tests::an_entry_the_model_lacks_is_refused ... FAILED
test ufo::bind::tests::an_auto_width_is_refused ... FAILED
test ufo::bind::tests::bind_card_refuses_a_split_merge ... FAILED
test ufo::bind::tests::reviving_a_zeroed_parameter_is_refused ... FAILED
test result: FAILED. 8 passed; 5 failed; 0 ignored; 0 measured; 1124 filtered out

$ cargo test -p vibegraph-lib --lib -- ufo::bind lhef::write::tests::the_slha   # DECAY exclusion, rebind, slha
test lhef::write::tests::the_slha_block_has_madgraphs_layout ... FAILED
test lhef::write::tests::the_slha_block_writes_moved_values_and_new_states ... FAILED
test ufo::bind::tests::a_moved_parameter_is_recorded_and_rebinds_exactly ... FAILED
test ufo::bind::tests::splitting_a_merged_pair_is_refused ... FAILED
test result: FAILED. 11 passed; 4 failed; 0 ignored; 0 measured; 1122 filtered out

$ cargo test -p vibegraph-lib --lib -- ufo::bind          # record every external
test result: FAILED. 6 passed; 7 failed; 0 ignored; 0 measured; 1124 filtered out

$ cargo test -p vibegraph --test cli_param_card            # generate ignores the record
test generate_evaluates_at_the_recorded_point ... FAILED
test reweighting_takes_the_recorded_point_as_its_denominator ... FAILED
test result: FAILED. 5 passed; 2 failed

$ cargo test -p vibegraph --test cli_param_card            # integrate evaluates at the defaults
test a_moved_parameter_is_recorded ... FAILED
test generate_evaluates_at_the_recorded_point ... FAILED
test result: FAILED. 5 passed; 2 failed
```

What these tests cannot see: they compare vibegraph with itself (the σ at a
moved card against MadGraph's is P-V1's gate); the CLI evidence that generate
evaluates at the record is statistical plus the reweighter's per-event audit;
the `<slha>` layout is compared with one MadGraph card (default SM), not with a
BSM model's MadGraph card, and Pythia's reading of it is not exercised.

## Informational σ: `e+ e- > t t~ h` at `ymt = 250`

`onshell_ee_run_card.dat` (500 + 500 GeV), `import model sm`, default
convergence target (0.1%), seeds 1–5 per side, card
`Block YUKAWA / 6 2.500000e+02`. Per seed (XSECUP, XERRUP of a one-event
`generate`, pb):

| seed | default | ymt = 250 |
|---|---|---|
| 1 | 2.014617e-03 ± 1.25e-06 | 4.090910e-03 ± 2.54e-06 |
| 2 | 2.016955e-03 ± 1.25e-06 | 4.095616e-03 ± 2.55e-06 |
| 3 | 2.017553e-03 ± 1.24e-06 | 4.097525e-03 ± 2.54e-06 |
| 4 | 2.016897e-03 ± 1.26e-06 | 4.096198e-03 ± 2.56e-06 |
| 5 | 2.017420e-03 ± 1.25e-06 | 4.098693e-03 ± 2.56e-06 |
| mean (seed s.e.) | 2.016688e-03 ± 5.3e-07, χ²/dof 0.91 | 4.095788e-03 ± 1.33e-06, χ²/dof 1.37 |

| comparison | value |
|---|---|
| vibegraph σ(250) / σ(default) | 2.03095 ± 0.00085 |
| MadGraph reweight ⟨w₂₅₀ / XWGTUP⟩ over the oracle's 200 events | 2.03726 ± 0.00550 |
| ratio pull | −1.14 |
| MadGraph's reweighted σ(250), on vibegraph's converged default σ | 4.10852e-03 ± 1.11e-05 pb |
| vibegraph σ(250) against it | −0.31 %, pull −1.14 |
| MadGraph's reweighted σ(250) as the oracle stores it (mean of w₂₅₀) | 4.07640e-03 ± 1.13e-05 pb |
| vibegraph σ(250) against vibegraph σ(default), the known-wrong comparison | +103.1 % |

The oracle's own sample was integrated at a fixed `20000 × 4`
(`gen_reweight_oracle.py`), and its `XWGTUP` (σ₀ = 2.000897e-03) sits 0.79 %
below the converged default above, which is why the ratio, not the oracle's
absolute weights, is the comparison. The reweighted σ is MadGraph evaluating
the matrix element at the moved point on vibegraph's events; a fresh MadGraph
run at the card is P-V1's gate.

Driver: `/tmp/.../scratchpad/P-F1-sigma/run.sh` (scratch, not committed), the
binary built from this tree. One earlier probe used a stale binary left by
the "integrate evaluates at the defaults" mutation and gave σ(250) = σ(0) =
σ(default); rebuilding gave the numbers above, and that stale result is the
same failure the mutation test records.

A Drell-Yan probe for the banking request (one seed each,
`dy13_proc_card.dat`, `dy13_mmll_run_card.dat`,
`LHAPDF_DATA_PATH=validation/pdf`): default card 644.126 ± 0.621 pb; MadGraph's
`dy13_param_card.dat` with `MZ = 95` 746.869 ± 0.732 pb (+16 %). That run
printed `MASS 24 sets MW to 80.419, which the model computes as
85.03597559307168; the card's value is ignored` and recorded `MZ = 95, WH =
0.006382339`.

## Proposed banking request (for P-V1)

Two new rows, each a fresh MadGraph run at the card (the reweight oracle is
not a σ reference at a moved point).

**Row 1, `dy13_mmll_60_120_mz95`**: `generate p p > e+ e-`, run card
`validation/madgraph/dy13_mmll_run_card.dat`, param card
`validation/madgraph/dy13_mz95_param_card.dat` = `dy13_param_card.dat` with the
one line

```
   23 9.500000e+01 # MZ 
```

(MW's dependent line may stay at `8.041900e+01`: MadGraph recomputes MW and
ignores it, and vibegraph warns and ignores it). `bank_seeded_sigma.sh` has
no param-card hook for the `dy13` kind; the driver change it needs is a
`row_kind` case and, in `run_row`'s `dy13` branch,

```
dy13_mmll_60_120_mz95) card="$HERE/dy13_mmll_run_card.dat"; param="$HERE/dy13_mz95_param_card.dat" ;;
```

with the `@hadronic` group left alone. Then

```
ROWS=dy13_mmll_60_120_mz95 pixi run -e madgraph bank-seeded-sigma
```

**Row 2, `ee_to_tth_ymt250`**: `validation/madgraph/scripts/ee_to_tth_ymt250.mg5`

```
# MadGraph5 batch script: e+ e- > t t~ h at ymt = 250 GeV
generate e+ e- > t t~ h
output ee_to_tth_ymt250 -nojpeg
launch
set lpp1 0
set lpp2 0
set ebeam1 500
set ebeam2 500
set use_syst False
set yukawa 6 250
done
```

the reweight oracle's `ee_tth_ymt` process and beams (its run card
`onshell_ee_run_card.dat` is otherwise MadGraph's default; if the oracle's cuts
must match exactly, launch with that card instead). The `script` kind's
`mes_check_card` reads every launch `set` as a run-card `key value` pair
(`mes_launch_sets` prints `$2, $3`), so `set yukawa 6 250` would be checked
as run-card key `yukawa` = `6` and the row refused: the driver needs
`mes_launch_sets` to skip the three-argument (param-card) `set` lines first.
Then

```
pixi run -e madgraph build-diagrams            # builds only the missing output/ee_to_tth_ymt250
ROWS=ee_to_tth_ymt250 pixi run -e madgraph bank-seeded-sigma
```

The vibegraph side of each row is `integrate --param-card` with the card the
banked run's `Cards/param_card.dat` holds (expect `WH = 6.382339e-03` recorded
beside the moved parameter).

## Gates

Commands as run, with the tail of each output.

```
$ cargo fmt --all --check
(no output, exit 0)

$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 1m 17s
exit=0

$ cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
exit=0

$ cargo test --workspace
(49 test binaries; summed over their result lines)
passed 1728, failed 0, ignored 19; exit=0
cli_param_card: test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s

$ python3 scripts/kb.py lint
1000 concepts, 333 backlog items; 0 errors, 4 warnings
```

(The four warnings are `state: blocked` backlog items this session did not
touch.) The first clippy run failed on `clippy::type_complexity` at
`parameters.rs` (`Vec<Vec<(String, f64)>>`); `MergedGroup` fixed it.

The extended-validation skill's gates for a model-binding, reweighting and
LHEF-writer change, run detached (`P-F1-gates.sh`): `validate-couplings`
(`coupling_oracle`, hermetic, in `cargo test --workspace`: 50 passed),
`check-sm-blob-fresh`, `validate-reweight` (both halves),
`validate-lhef`, `validate-generate-proton` and `validate-unweighting`, plus
the banked CLI tests that write event files (`cli_integrate`,
`cli_decay_chain_events`, `validate_samples_proton`).

```
$ cargo test -p vibegraph-lib --test reweight_mg_oracle -- --nocapture
ee_ttx_smeft [joint]    pm111: 200 events, largest |w/w_MG - 1| 2.45e-8, 0 beyond 2e-7
test reweighting_matches_madgraphs_reweight_module_event_by_event ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s

$ cargo test -p vibegraph-lib --features extended-validation --test sm_interned_blob
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

$ cargo test --config 'profile.release-debug.debug=0' -p vibegraph --profile release-debug \
    --features extended-validation --test cli_reweight_proton -- --nocapture --test-threads 1
test hadronic_reweighting_is_taken_in_the_drawn_subprocess ... Audit { events: 5000, ..., largest: 0.0, m2_mismatches: 0, flavour_mismatches: 0, non_finite: 0 }
test matched_mixed_multiplicity_reweighting_is_taken_in_the_drawn_subprocess ... Audit { events: 3000, ..., largest: 0.0, m2_mismatches: 0, flavour_mismatches: 0, non_finite: 0 }
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 162.18s

$ cargo test ... -p vibegraph-lib --profile release-debug --features extended-validation --test validate_lhef -- --nocapture
test banked_files_round_trip_byte_for_byte ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 65.94s

$ cargo test ... -p vibegraph --profile release-debug --features extended-validation --test cli_generate_proton -- --nocapture --test-threads 1
test result: ok. 5 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 47.40s

$ cargo test ... -p vibegraph-lib --profile release-debug --features extended-validation --test validate_unweighting -- --nocapture
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.06s

$ cargo test ... -p vibegraph --profile release-debug --features extended-validation --test cli_integrate -- --nocapture
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.46s

$ cargo test ... -p vibegraph --profile release-debug --features extended-validation --test cli_decay_chain_events -- --nocapture
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 221.10s

$ cargo test ... -p vibegraph --profile release-debug --features extended-validation --test validate_samples_proton -- --nocapture
test result: ok. 12 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out; finished in 922.78s
```

**Not run**: `validate-pythia` and `validate-mlm-pythia` (no Pythia 8 here),
though every event file now carries an `<slha>` block a Pythia run reads;
`generate-reweight-oracle` (needs MadGraph); the full banked
`pixi run --skip-deps validate` (the session changes no amplitude or
evaluator code; the gates above are the skill's map for what it touches).

## Found

- **MadGraph's restriction fixes `1`-valued parameters too; vibegraph leaves
  them free.** `import_ufo.py` `detect_special_parameters` returns the
  parameters the restriction sets to exactly 1, `fix_parameter_values` replaces
  them by 1, and `check_param_card` refuses (or resets, with `modify`) a card
  that moves one ("is not at one"). `ParameterSet::apply_restrict` locks only
  zeros, so `integrate --param-card` moving such a parameter evaluates a point
  MadGraph would not run. Closing it: lock the ones as the zeros are locked
  and refuse a move, with a test on a restriction that sets a parameter to 1
  (SMEFTsim's restrictions set many Wilson coefficients to 1).
- **MadGraph's anti-restriction values are not read.** A restrict card value
  of `9.999999e-1` or `0.000001e-99` means "keep this parameter" to MadGraph,
  which then stores 1 or 0 (`restrict_model`, the loop after the merges).
  `apply_restrict` takes them literally: defaults of 0.9999999 and 1e-105
  where MadGraph runs at 1 and 0. No repository model uses them.
  Closing it: map the two values as MadGraph does, with a toy restrict card.
- **Couplings MadGraph merges at the restriction point.** `restrict_model`
  also merges couplings that are equal at the restriction's values
  (`detect_identical_couplings`, `merge_iden_couplings`) and evaluates the
  merged ones through the first. A card that moves a parameter so that two
  such couplings differ is evaluated differently by MadGraph and vibegraph.
  Not checked for the SM default with moved `MZ` or `ymt`; P-V1's gate would
  show it as a σ disagreement. Closing it: list MadGraph's merged couplings
  per restriction (its `import model` log prints "Fuse the Following
  coupling") and check each pair's expressions differ only by a factor the
  card cannot move.
- **The SLHA parser drops lines silently.** A `DECAY` line whose width is
  neither a number nor `Auto`, a block entry whose leading tokens are not
  integers, and a repeated `(block, code)` (the last wins; MadGraph raises
  "is already define") are all skipped without an error. Closing it: make each
  a `SlhaError::Parse` at its line, after checking every restrict card the
  loader reads still parses.
- **The `<slha>` block is unexercised by a shower.** Pythia's default
  `SLHA:readFrom` reads SLHA from the LHEF header, so the top and Higgs masses
  and widths of every vibegraph sample now reach Pythia from the header (at
  the defaults they equal what MadGraph's files carry). `validate-pythia` and
  `validate-mlm-pythia` have not run against files with the block.
- **The reweight oracle's `ee_tth_ymt` sample is integrated at a fixed
  `20000 × 4`** and its σ₀ (2.000897e-03 pb) is 0.79 % below a converged
  default run (2.016688e-03 ± 5.3e-07, five seeds): any comparison of
  MadGraph's absolute reweighted σ against a converged vibegraph σ inherits that
  offset. Closing it: either regenerate the oracle sample at the converged
  budget or document that only per-event ratios are comparable.
- **Banking driver**: `bank_seeded_sigma.sh` has no param-card hook for the
  `dy13` kind, and `mes_launch_sets` misreads a launch block's param-card
  `set <block> <code> <value>` (see the banking request).
