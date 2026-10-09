---
type: Algorithm
title: Restrict-card semantics
description: "Restriction drops zero couplings, then empty vertices, then unreferenced Lorentz structures; a card's non-zero values become defaults and its zeros lock, as in MadGraph's generated param_card."
status: draft
tags: [ufo, restrict-card, parameters, madgraph-parity, smeftsim]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n35-l1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/35-ufo-lorentz-sprint-plan.md#L604-L675", title: "Note 35 §4 L1, loader and model-topology surface"}
  - {id: n35-c, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/35-ufo-lorentz-sprint-plan.md#L788-L841", title: "Note 35 §4 C, SMEFT cross section and the restricted-defaults fix"}
  - {id: n35-close, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/35-ufo-lorentz-sprint-plan.md#L1281-L1358", title: "Note 35 §10.1, what the sprint leaves gated"}
  - {id: mg-remove-interactions, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/models/import_ufo.py#L2876", title: "MadGraph import_ufo.py RestrictModel.remove_interactions"}
  - {id: mg-fix-params, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/models/import_ufo.py#L2980", title: "MadGraph import_ufo.py RestrictModel.fix_parameter_values"}
  - {id: code-params, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/ufo/parameters.rs#L97-L131", title: "vibegraph-lib/src/ufo/parameters.rs apply_restrict"}
measured:
  - {commit: 412bc68, pr: 5, landed_in: e73b158, command: "cargo test -p vibegraph-lib --features extended-validation --test smeftsim restricted_defaults_are_madgraphs_generated_param_card"}
---

A restrict card is an SLHA parameter card (`ufo/slha.rs`, `ParamCard`) that MadGraph
applies when a model is imported. `import model <name>-<variant>` selects
`restrict_<variant>.dat` from the model directory; a bare `import model <name>` uses
`restrict_default.dat` when the directory has one (`UFOModel::load_with_digest`). The
Standard Model is interned with its nine restrict cards baked in as text
(`ufo/sm.rs`, `SMRestrict`).

A restriction does two things: it sets the model's parameters, and it prunes what
vanishes under them.

## Parameters: non-zero values are defaults, zeros lock

`ParameterSet::apply_restrict` (`vibegraph-lib/src/ufo/parameters.rs`) reads every
external parameter the card names: [^code-params]

- the card's value becomes the parameter's **default**, whether zero or not;
- a parameter set to exactly **zero** is also **locked** (`ParameterSet::zeros`): a later
  param card cannot revive it, and `dependents()` reports nothing downstream of it.

This is MadGraph's semantics. MadGraph assigns every external the card names
(`model_reader.set_parameters_and_couplings`) and writes those values into the
generated process's `param_card.dat`, so a run of a restricted model with no card of its
own runs at the restriction's values. Zeros lock because the restriction is also what
pruned the vertices and diagrams; reviving a zeroed parameter would evaluate couplings
the pruned vertex set no longer matches. Non-zero values stay overridable, which is
what makes the generated card an editable card. [^n35-c]

The failure this rule prevents is silent. Baking in only the zeros leaves every other
coefficient at `parameters.py`'s default, which for SMEFTsim is zero, so a card-less
run of a `restrict_massless` SMEFTsim model evaluated as its **Standard-Model limit**:
`e+ e- > mu+ mu- a NP<=1` gave amplitudes equal to the pure-SM ones, not zero and not
SMEFT. No Standard-Model process could expose it; the falsifier is
`restricted_defaults_are_madgraphs_generated_param_card` (`vibegraph-lib/tests/smeftsim.rs`),
which compares 421 external parameters over 13 gated rows against MadGraph's own
generated cards at ≤ 1e-12 and fails when the fix is reverted. [^n35-close]

A consequence for the SM: card-less defaults take `restrict_default.dat`'s values
(`Gf` 1.16639e-5, `WZ` 2.441404, `WT` 1.4915, `WW` 2.0476), which moved them toward
MadGraph's. [^n35-c]

**Known divergence.** MadGraph's `RestrictModel.fix_parameter_values` takes both
zero-valued and one-valued parameters [^mg-fix-params]: a parameter a restrict card sets
to exactly `1` is fixed too. The loader locks only zeros. No restrict card in the
repository sets a non-`QNUMBERS` parameter to 1, so the gap is latent; it is the
[unit-values backlog item](../backlog/feature/restrict-card-unit-values-not-fixed.md),
which also asks whether MadGraph merges identical values there.

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

MadGraph's `RestrictModel` also merges couplings that are equal up to sign, which shows
in its generated code as renamed couplings (`GC_4` called as `-GC_3`). That rewrites
names, not values; the loader keeps such couplings separate.

## Where it applies, and what is identified

- `ParsedModel::parse` builds the pre-restriction model; the interned SM blob stores
  that form, and each variant's card is applied at load (`ufo::sm::sm_model`).
- The restriction is baked in before the model digest is taken, so the card contributes
  to [model identity](model-identity-digest.md) through its effect, not its text.
- Every CLI run computes at the selected restriction's values: there is no param-card
  option yet ([backlog](../backlog/feature/cli-has-no-param-card-option.md)). The
  library binds a separate card through `EvaluatedModel::from_model_card`.

## Evidence on a non-SM model

The [vendored SMEFTsim UFO](smeftsim-topu3l.md) is where these rules are exercised: its
two restrict cards switch every Wilson coefficient off (`SMlimit_massless`) or on at
fixed values (`massless`). The `ee_to_ttx_smeft` σ gate is not blind to the
restriction: `e+ e- > t t~ NP<=1` at 500 GeV reads 2.2230 pb under `-massless`
(MadGraph 2.2223 pb) and 0.5496 pb under `-SMlimit_massless`, a factor 4.04 below.
[^n35-c] The gated SMEFTsim rows are listed in
[non-SM rows](../validation/non-sm-rows.md).

[^n35-l1]: Note 35 §4 L1 (landed `00858a8`): MadGraph's order of operations, colour strings left unpruned.
[^n35-c]: Note 35 §4 C (landed `412bc68`): the card-less SM-limit defect, its fix, the falsifier, the SM default shift, the unit-value divergence.
[^n35-close]: Note 35 §10.1: the restricted-defaults rule as a pinned convention and the bug only a non-SM row could show.
[^mg-remove-interactions]: `models/import_ufo.py` `RestrictModel.remove_interactions`, L2876.
[^mg-fix-params]: `models/import_ufo.py` `RestrictModel.fix_parameter_values(zero_parameters, one_parameters, …)`, L2980.
[^code-params]: `vibegraph-lib/src/ufo/parameters.rs` `apply_restrict` and its documentation.
