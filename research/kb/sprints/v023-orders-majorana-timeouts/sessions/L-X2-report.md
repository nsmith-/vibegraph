---
type: Session Report
title: "L-X2 report: param-card entries the model has no parameter for bind as ignored, by block and code"
description: "A block the model does not define and a MASS or DECAY entry at a code no particle has are ignored with a warning; an unknown code in a defined block, and a value for a restriction-zeroed parameter (now naming restrict_<x>.dat), stay refused."
status: draft
tags: [param-card, refusal, madgraph-parity, cms-gridpacks]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-22}
---
Branch `sprintl-leftovers`. Session L-X2 (brief `sessions/L-X2.md`). Closed item,
file deleted: `param-card-entries-for-absent-particles-refused`.

## Can bind see the unrestricted parameter list? Yes

`ParsedModel::apply_restriction` refuses a restriction card that lacks any
external (`RestrictMissing`) and `apply_restrict` only sets defaults and fills
`ParameterSet::zeros`; no external is dropped. So `model.params.externals`
holds the full model's parameters with their blocks and codes, and a zeroed one
is in `zeros`. A block with no external is one no version of the model defines
(case 1); an external in `zeros` was removed by the restriction (case 4). No
structural change was needed. What bind lacked was the restriction's *name*:
the model does not carry it, so `bind_card_under(model, card, restriction)`
takes it, and `integrate` and the `enumerate` flavour preview pass
`LoadedModel::identity.restrict`. `bind_card` keeps its signature and says "the
model's restriction".

## What changed

1. `vibegraph-lib/src/ufo/bind.rs`: entries in a block no external sits in, and
   `MASS`/`DECAY` entries at a code no particle has, are pushed into
   `BoundCard::ignored` (always, no tolerance test). `IgnoredEntry` is now
   `{block, code, card, why}` with `IgnoredBecause::{Computed{param, computed},
   NoParameter}`. `CardError::Unknown` remains for a code absent from a block the
   model defines. `RevivesZeroed` gained `restriction` and reads "it was removed
   by restrict_<x>.dat, which fixes it to zero ...".
2. `vibegraph-cli/src/integrate.rs`, `enumerate.rs`: one warning per
   `NoParameter` entry; restriction name threaded to `bind_param_card`.
3. `ufo/card_classes.rs`: the one refused row became three (block undefined:
   benign; MASS/DECAY code absent: benign; unknown code in defined block:
   refused); MadGraph citation corrected to L656 (`for line in open(identpath)`
   in `write_inc_file`; the old row said L655).
4. Docs: `docs/src/cli/overview.md`, and the two rows and the
   ruling paragraph in `research/kb/model/param-card-classification.md`.
   The gpdb backlog item linked the deleted item; its link is replaced by text
   (the one backlog edit, needed to keep `kb-lint` at 0 errors).

## Oracles

All hermetic, debug build.

- `cargo test -p vibegraph-lib --lib ufo::` -> `190 passed; 0 failed`. Includes
  `every_param_card_construct_behaves_as_classified` (the table and its examples),
  `entries_with_no_parameter_are_ignored_and_listed`,
  `an_unknown_code_in_a_defined_block_is_refused`.
- `cargo test -p vibegraph-lib --test input_card_refusals` -> `25 passed; 0 failed`
  (after fixing my own test's needle: the block name prints lower-cased). The
  new `an_entry_without_a_parameter_is_ignored_or_refused_by_what_removed_it`
  runs the four cases on the fixture UFO restricted so its whole `NEWCOUP`
  block is zero: `NOSUCH` block and `MASS`/`DECAY 1000022` ignored (3 entries,
  none moved); `NEWCOUP 99` refused as unknown; `NEWCOUP 1 0.5` refused naming
  `restrict_test.dat`; `NEWCOUP 1 0.0` binds. The last pair is the case-1 vs
  case-4 distinction.
- `cargo test -p vibegraph --test cli_param_card` -> `9 passed; 0 failed`. New
  `entries_the_model_has_no_parameter_for_are_ignored_with_warnings`: three
  warnings on stderr and the artifact record equals the default run's.
- `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt
  --all` applied; `pixi run kb-lint` -> `0 errors, 5 warnings` (pre-existing).
- First `cargo test --workspace` (before the follow-up): 19 suites `ok`, then
  `committed_cli_reference_matches_the_binary` FAILED, because I had hand-edited
  the generated `docs/src/cli/reference.md`. Fixed properly in the follow-up:
  the `--param-card` doc comment in `integrate.rs` is updated and
  `scripts/gen-cli-docs.sh` regenerated the file.
- Follow-up (commits `45e38a9d` and merge `9bf5f34f`): a decay table below a
  `DECAY` line at an absent code is now ignored with that entry
  (`IgnoredBecause::NoParticleTable`, warned, not carried to the `<slha>`
  header; `ParamCard::without_decay_tables`). Tests:
  `a_decay_table_for_an_absent_particle_is_ignored` (bind.rs: the table is
  listed with 2 channels and sum 1.0, the table at code 6 is still carried, the
  table at -24 still refused), the classification row's example now carries a
  table (`Same` against the empty card, so a carried table would fail it), and
  the CLI test counts four warnings.
- Main merged (`origin/main` 4b132cc, squared orders). One conflict: main had
  appended the HNL `ckmblock` note to the backlog item this session deletes;
  resolved as deleted (the ruling answers that note). One missing
  `squared_orders: None` in a literal PR 11 added
  (`diagrams/chain.rs`, `opposite_sign_routes_to_one_graph_refuse_by_name`),
  `None` being what main's own test literals use.
- Hermetic suite with reference data hidden (`research/refs/mg5amcnlo`,
  `validation/madgraph/output`, `validation/pdf/NNPDF23_lo_as_0130_qed` moved
  to the scratchpad by `lx2-hidden.sh`, restored by its EXIT trap; confirmed
  back with `ls -d` afterwards): `cargo test --workspace --no-fail-fast` ->
  68 suites `ok`, 1 FAILED: `cli_param_card::
  entries_the_model_has_no_parameter_for_are_ignored_with_warnings`, my own
  test still expecting 3 warnings where the table adds a fourth (the
  behaviour was right). Fixed; `cargo test -p vibegraph --test cli_param_card`
  -> `9 passed; 0 failed` (run with the data back in place, not re-run hidden;
  the test reads none of it).
- `cargo fmt --all --check` OK; `cargo clippy --workspace --all-targets -- -D
  warnings` clean, and clean again with `--features
  vibegraph/extended-validation,vibegraph-lib/extended-validation`;
  `pixi run kb-lint` -> `1351 concepts, 462 backlog items; 0 errors, 5 warnings`.
- Not re-run after the last test fix: the full workspace suite (only the one
  file changed, a test).

**No banked sigma moves.** The change only turns entries that were refused into
ignored ones; evaluation (`EvaluatedModel::from_model_card`) and every consumed
entry are untouched. Evidence: `a_card_at_the_defaults_moves_nothing` and
`madgraphs_own_card_moves_only_the_width_it_rounds` still pass unchanged, and
the CLI test asserts the artifact with ignored entries equals the default run.
I did not run `validate` (CI's banked job does).

## What the tests cannot detect

- All ignore tests compare vibegraph with itself: that MadGraph's sigma at such a
  card equals the default is the validator request below.
- A typo'd MASS/DECAY code (e.g. 42 for 24) now binds with only a warning;
  that is the ruling, and no test can distinguish it from an intended absent
  particle.
- Case 4 is exercised on a toy restriction and the SM's zeroed charm mass; the
  real HNL restrictions are not loaded here.
- The ignored table is dropped from the `<slha>` header, which MadGraph would
  copy verbatim; no test compares the header with MadGraph's for such a card.

## Validator request (manager sends; not run here)

Branch `sprintl-leftovers` at the commit of this session.
1. HNL gridpack cards 84, 85, 438-442 on `SM_HeavyN_Gen3Mass_NLO`: bind with
   `CKMBLOCK` listed as ignored (one warning per entry); then enumerate,
   `integrate --check`, and a small sigma (>=5 seeds, unweighted mean) against the
   gridpack's MG sigma. Expected: refusals gone; sigma/MG consistent with the
   rows' other cards.
2. The DarkPhoton cards named in the item, untrimmed (MASS 1000022 present):
   they should bind with that entry ignored and the MZd of the card used; compare
   with the hand-trimmed runs (0.985 and 0.971 of MG). A decay table under 1000022 is ignored too.

## Found

- `bind_card_under` names `restrict_<x>.dat` from the directive's variant;
  for an explicit `--restrict` path override the label is the variant, not the
  file. Close by passing the file name from `LoadedModel`.
