---
type: Algorithm
title: Restrict-card semantics
description: "Restriction drops zero couplings, then empty vertices, then unreferenced Lorentz structures; a card's non-zero values become defaults and its zeros lock, as in MadGraph's generated param_card."
status: draft
tags: [ufo, restrict-card, parameters, madgraph-parity, smeftsim]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
sources:
  - {id: n35-l1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L604-L675", title: "Note 35 §4 L1, loader and model-topology surface"}
  - {id: n35-c, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L788-L841", title: "Note 35 §4 C, SMEFT cross section and the restricted-defaults fix"}
  - {id: n35-close, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1281-L1358", title: "Note 35 §10.1, what the sprint leaves gated"}
  - {id: mg-remove-interactions, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/import_ufo.py#L2876", title: "MadGraph import_ufo.py RestrictModel.remove_interactions"}
  - {id: mg-fix-params, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/import_ufo.py#L2980", title: "MadGraph import_ufo.py RestrictModel.fix_parameter_values"}
  - {id: code-params, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/ufo/parameters.rs#L97-L131", title: "vibegraph-lib/src/ufo/parameters.rs apply_restrict"}
measured:
  - {commit: 412bc68, pr: 5, landed_in: e73b158, command: "cargo test -p vibegraph-lib --test smeftsim restricted_defaults_are_madgraphs_generated_param_card"}
---

A restrict card is an SLHA parameter card (`ufo/slha.rs`, `ParamCard`) that MadGraph
applies when a model is imported. `import model <name>-<variant>` selects
`restrict_<variant>.dat` from the model directory; a bare `import model <name>` uses
`restrict_default.dat` when the directory has one (`UFOModel::load_with_digest`). The
Standard Model is interned with ten restrict cards baked in as text: MadGraph's
nine and CMS genproductions' `ckm_no_b_mass` (`ufo/sm.rs`, `SMRestrict`;
`SMRestrict::ships_with_madgraph` tells them apart).

A restriction does two things: it sets the model's parameters, and it prunes what
vanishes under them.

## Parameters: non-zero values are defaults, zeros lock

`ParameterSet::apply_restrict` (`vibegraph-lib/src/ufo/parameters.rs`) reads every
external parameter the card names: [^code-params]

- the card's value becomes the parameter's **default**, whether zero or not;
- a parameter set to exactly **zero** is also **locked** (`ParameterSet::zeros`): a later
  param card cannot revive it (`integrate --param-card` refuses a card that tries), and
  `dependents()` reports nothing downstream of it.

This is MadGraph's semantics. MadGraph assigns every external the card names
(`model_reader.set_parameters_and_couplings`) and writes those values into the
generated process's `param_card.dat`, so a run of a restricted model with no card of its
own runs at the restriction's values. Zeros lock because the restriction is also what
pruned the vertices and diagrams; reviving a zeroed parameter would evaluate couplings
the pruned vertex set no longer matches. Non-zero values stay overridable, which is
what makes the generated card an editable card. [^n35-c]

The failure this rule prevents is silent. Baking in only the zeros would leave every other
coefficient at `parameters.py`'s default, which for SMEFTsim is zero, so a card-less
run of a `restrict_massless` SMEFTsim model would evaluate as its **Standard-Model
limit**: `e+ e- > mu+ mu- a NP<=1` gives the pure-SM amplitudes, neither zero nor SMEFT. No Standard-Model process could expose it; the falsifier is
`restricted_defaults_are_madgraphs_generated_param_card` (`vibegraph-lib/tests/smeftsim.rs`),
which compares 421 external parameters over 13 gated rows against MadGraph's own
generated cards at ≤ 1e-12 and fails when the fix is reverted. [^n35-close]

A consequence for the SM: card-less defaults take `restrict_default.dat`'s values
(`Gf` 1.16639e-5, `WZ` 2.441404, `WT` 1.4915, `WW` 2.0476), as MadGraph's do. [^n35-c]

**Ones are fixed as well.** MadGraph's `RestrictModel.fix_parameter_values` takes
both zero-valued and one-valued parameters [^mg-fix-params], so a parameter a restrict
card sets to exactly `1` is fixed too. The loader records those externals in
`ParameterSet::ones` (`apply_restrict`), and `bind_card` refuses a card that moves
one (`CardError::MovesFixedOne`), as MadGraph's `param_card_rule.dat` check refuses
it. A one prunes nothing, unlike a zero, so it is held at binding and not locked
in evaluation. MadGraph's merge of identical values (`detect_identical_parameters`)
is `ParameterSet::merged`, held the same way (below). Every toy-UFO restrict card
leaves the width `WO8` at one, and `bind.rs` pins both rules.

## Pruning, in MadGraph's order

`ParsedModel::apply_restriction` (`ufo/mod.rs`) follows
`import_ufo.RestrictModel.remove_interactions` [^mg-remove-interactions]:

1. Evaluate every coupling at the card's values; remove each **zero coupling** from
   every vertex's `(colour, Lorentz) → coupling` map.
2. Remove **vertices left with no coupling**.
3. Remove the **Lorentz structures** no surviving coupling of a vertex references, and
   reindex the remaining coupling keys onto the shortened list.

The restriction acts on interactions already split by coupling-order tuple
([coupling orders](coupling-orders.md)), so an SMEFTsim `FFV` vertex reaches the
evaluator in the SM limit with `FFV1` alone, its dipole and current-shift structures
gone with their zero couplings. Removing zero couplings, not only empty vertices, is
what makes that true.

**Colour strings are not pruned.** MadGraph prunes only the Lorentz list, and every
consumer reaches a colour structure through the coupling keys, so an unreferenced one
is never read. [^n35-l1]

MadGraph's `RestrictModel` also merges couplings whose values, rounded to about ten
significant digits, are equal or opposite (`detect_identical_couplings`, L2525;
`merge_iden_couplings`, L2780), so its generated code calls one coupling under another's
name, with a sign where they are opposite. That rewrites names, not values; the loader
keeps such couplings separate.

**What counts as zero is MadGraph's rule**, kept as a MadGraph-compatibility
choice (`ZeroRule`, `ufo/mod.rs`). Over every coupling of the model, on a vertex or
not, evaluated at the card: a coupling with `|g| < 1e-13` is zero, unless some
coupling lies in `[1e-13, 1e-10)`, in which case only an exact zero is
(`detect_identical_couplings`, L2549–L2558, whose `strict_zero` rerun this is). The
thresholds are absolute: a model whose couplings are all naturally tiny is pruned to
nothing, and one coupling at `1e-12` saves another at `1e-14`. A rule judging each
coupling against its own scale would be cleaner; it is what a mode without
MadGraph's compatibility choices would use. `tests/ufo_input_refusals.rs` pins both
branches and both edges of the window. Both branches are live on banked models: every
SMEFTsim card has couplings in the window (`GC_309`–`GC_311`, 3–4e-11, on all of
them), so SMEFTsim is always restricted under the strict rule, and `MSSM_SLHA2` has two
rounding residues (`GC_857`, `GC_888`, below 1e-15) that the threshold rule prunes. No
SM, `loop_sm` or toy card has a coupling in `(0, 1e-10)`.

MadGraph's coupling merge (`limit_to_6_digit`, rounding to about ten significant
digits) changes no vertex set: it renames a coupling inside a vertex that survives,
and `optimise_interaction` then combines the Lorentz structures that share one
coupling. Neither removes nor adds an interaction.

**A restrict card's `DECAY … Auto` row is refused** (`UfoError::RestrictAutoWidth`).
MadGraph keeps such a width free through the restriction and writes it to the
process card as `Auto`, for MadWidth to compute at launch (`modify_autowidth`,
`restrict_model`). The loader computes no width and would leave the `parameters.py`
default in its place. `integrate --param-card` refuses the row the same way
(`CardError::AutoWidth`), a reweight card's `set … Auto` is not a number to its
parser, and `EvaluatedModel::from_model_card` refuses a card carrying one with the same
`CardError::AutoWidth`, so no path reads a width from an `Auto` row. No banked restrict card has one.

## Where it applies, and what is identified

- `ParsedModel::parse` builds the pre-restriction model; the interned SM blob stores
  that form, and each variant's card is applied at load (`ufo::sm::sm_model`).
- The restriction is baked in before the model digest is taken, so the card contributes
  to [model identity](model-identity-digest.md) through its effect, not its text.
- A CLI run computes at the selected restriction's values unless `integrate
  --param-card` binds a card (`ufo::bind::bind_card`, through
  `EvaluatedModel::from_model_card`). The binding refuses a non-zero value for a
  locked parameter rather than ignoring it, and refuses a card that moves one
  parameter of a group MadGraph's restriction merges
  (`detect_identical_parameters`: same block, defaults equal up to sign, not 0, 1 or a
  width; `ParameterSet::merged`) without the others. The artifact records the
  parameters the card moved, and `generate` rebinds from that record
  ([run inputs](../pipeline/run-inputs-in-the-artifact.md) §1).

## Evidence on a non-SM model

The [vendored SMEFTsim UFO](smeftsim-topu3l.md) is where these rules are exercised: its
two restrict cards switch every Wilson coefficient off (`SMlimit_massless`) or on at
fixed values (`massless`). The `ee_to_ttx_smeft` σ gate is not blind to the
restriction: `e+ e- > t t~ NP<=1` at 500 GeV reads 2.2230 pb under `-massless`
(MadGraph 2.2223 pb) and 0.5496 pb under `-SMlimit_massless`, a factor 4.04 below.
[^n35-c] The gated SMEFTsim rows are listed in
[non-SM rows](../validation/non-sm-rows.md).

[^n35-l1]: Note 35 §4, the loader and model-topology surface: MadGraph's order of operations, colour strings left unpruned.
[^n35-c]: Note 35 §4, the SMEFT cross-section section: the card-less SM-limit defect, its fix, the falsifier, the SM default shift, the unit-value divergence.
[^n35-close]: Note 35 §10.1: the restricted-defaults rule as a pinned convention and the bug only a non-SM row could show.
[^mg-remove-interactions]: `models/import_ufo.py` `RestrictModel.remove_interactions`, L2876.
[^mg-fix-params]: `models/import_ufo.py` `RestrictModel.fix_parameter_values(zero_parameters, one_parameters, …)`, L2980.
[^code-params]: `vibegraph-lib/src/ufo/parameters.rs` `apply_restrict` and its documentation.
