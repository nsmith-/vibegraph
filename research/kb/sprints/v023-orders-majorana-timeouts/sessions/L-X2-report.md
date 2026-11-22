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
- `cargo test --workspace` (log `lx2-full.log`): 19 suites `ok`, then
  `committed_cli_reference_matches_the_binary` FAILED: I had hand-edited the
  generated `docs/src/cli/reference.md`, which `scripts/gen-cli-docs.sh`
  builds from the `--param-card` doc comment in `integrate.rs`. I reverted that
  edit (`git checkout`), so the file is the committed one again, but I have
  **not re-run** it, and cargo stopped at that binary, so the suites after
  `cli_reference_docs` in cargo's order did not run. The gate is therefore not
  fully green on this commit. Needed from the manager (cores permitting):
  update the `--param-card` doc comment (integrate.rs, ~L429-435: "and an entry
  the model has no parameter for" -> a code the model lacks inside a defined
  block; ignored: an undefined block and an absent-particle MASS/DECAY),
  run `scripts/gen-cli-docs.sh`, then `cargo test --workspace --no-fail-fast`.
  I did not do it because the manager asked for no further cargo runs.

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
- A decay *table* under an absent particle's `DECAY` line is still refused
  (`DecayTableParticle`): the ruling covers entries, not tables, and no test
  here has a gridpack card with one.

## Validator request (manager sends; not run here)

Branch `sprintl-leftovers` at the commit of this session.
1. HNL gridpack cards 84, 85, 438-442 on `SM_HeavyN_Gen3Mass_NLO`: bind with
   `CKMBLOCK` listed as ignored (one warning per entry); then enumerate,
   `integrate --check`, and a small sigma (>=5 seeds, unweighted mean) against the
   gridpack's MG sigma. Expected: refusals gone; sigma/MG consistent with the
   rows' other cards.
2. The DarkPhoton cards named in the item, untrimmed (MASS 1000022 present):
   they should bind with that entry ignored and the MZd of the card used; compare
   with the hand-trimmed runs (0.985 and 0.971 of MG). If a card carries a
   decay table under 1000022, expect `DecayTableParticle` (see above).

## Found

- A `DECAY <absent pdg>` line followed by a decay table is refused
  (`DecayTableParticle`) although the entry itself now binds. Close by
  deciding whether gridpack cards carry such tables; if so, carry or drop
  them as MadGraph's header does.
- `bind_card_under` names `restrict_<x>.dat` from the directive's variant;
  for an explicit `--restrict` path override the label is the variant, not the
  file. Close by passing the file name from `LoadedModel`.
