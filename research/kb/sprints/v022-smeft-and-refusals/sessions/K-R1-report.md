---
type: Session Report
title: "K-R1 report: restriction cards as MadGraph applies them"
description: "MadGraph's restriction path read at 3.7.1 and reproduced: 9.999999e-1 and 0.000001e-99 load as free one and zero, -full loads unrestricted, the model name splits at the last dash. A card that omits an external parameter stays refused, because MadGraph refuses it too."
status: draft
tags: [ufo, restriction, refusal, smeftsim, madgraph-parity, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Worktree `/home/user/wt/sprintk-restriction`, branch `sprintk-restriction`,
based on `main` at `703f57f`. MadGraph is the pinned 3.7.1 source
(`research/refs/mg5amcnlo`, b7687064), copied to a private work area and run
with the `madgraph` pixi environment's Python.

## MadGraph's restriction path (3.7.1)

All citations are `models/import_ufo.py` unless another file is named.

**Model name and suffix** (`get_path_restrict`, L201-L241). The whole
argument is tried as a model first (L204). Only if that fails is it split at
`-`: the restriction is the part after the *last* `-`, and the model name is
everything before it rejoined (L210-L211, L218). The card is
`restrict_<suffix>.dat` (L220). A suffix of `full` sets the card to `None`
(L223-L224), so no restriction is applied, not even `restrict_default.dat`. A
bare name applies `restrict_default.dat` when the directory has one
(L228-L229). With no card, `import_model` only adjusts the complex-mass
setting (L312-L321): every vertex and parameter of the model is kept.

**Reading the card** (`model_reader.py`, `set_parameters_and_couplings`,
L58-L205). Before any value is read, the card's block set must equal the
model's external blocks (L102-L106). A missing block is a `MadGraph5Error`
"Invalid restriction card (not same block)" (L115-L172), except for three
convention conversions (a `loop` block, `fralpha`/`alpha`, SLHA1 to SLHA2).
An unknown extra block is only a warning (L126-L128). Within a present block,
an external parameter the card does not set raises `MadGraph5Error` "<block>
<code> not define" (L178-L184): `Block.get` without a default raises
`KeyError` (`check_param_card.py` L177-L189). **MadGraph never falls back to
the UFO default for an absent entry.**

**Zero and one** (`detect_special_parameters`, L2615-L2627). Exact Python
float equality: `value == 0` (not the `ZERO` parameter) and `value == 1`,
over every entry of `parameter_dict`, internal parameters included. There is
no tolerance. `-1` is not special. `9.999999e-1 == 1` is false, so that value
is not fixed. `fix_parameter_values` (L2980-L3087) sends a zero mass or width
to `ZERO` on the particles. It keeps a mass or width at one as a parameter
(L2992-L2995). It adds a `zero` or `one` rule to the process's
`param_card_rule.dat` for every *external* parameter at exactly 0 or 1
(L3005-L3012). It then folds the remaining zero/one parameters into constants
or removes them.

**Couplings** (`detect_identical_couplings`, L2525-L2594). The couplings are
evaluated at the card's values *as written*: a `9.999999e-1` is 0.9999999
here. A coupling is zero if `|g| < 1e-13`, unless some coupling lies in
`[1e-13, 1e-10)`, which reruns the whole check with exact zeros only. Zero
couplings and their vertices are removed. Identical couplings (equal up to
sign at 6+ digits, same orders) are merged, a renaming with no numerical
effect.

**Identical parameters** (`detect_identical_parameters`, L2709-L2735). These
are external, non-`DECAY` parameters in the same block whose values are equal
or opposite. Values in `[0, 1, 0.000001e-99, 9.999999e-1]` are skipped
(L2724). The first is kept; the others become `±first` (`merge_iden_parameters`,
L2825-L2874), with an `identical`/`opposite` rule.

**The two spellings** (L2454-L2462). After all of the above, every
`parameter_dict` entry equal to `9.999999e-1` becomes `1` and every one equal
to `0.000001e-99` becomes `0`. The comment says these values "are used to
avoid restriction". `ParamCardWriter` does the same for the generated
`param_card.dat` (`write_param_card.py` L253-L256). Such a parameter is
therefore a free parameter with a default of 1 or 0. It is not fixed, not
merged, and has no rule. Its couplings were judged at the spelled value, so
a coupling linear in a `0.000001e-99` parameter (~1e-105) is pruned as zero.

**A `param_<x>.dat` beside `restrict_<x>.dat`** (L2491-L2496) replaces the
defaults after the restriction. The loader already refuses that.

### What this means for the four items

- `restrict-card-unit-values-not-fixed`: `ParameterSet::ones` and the
  `bind_card` refusal (from the param-card sprint) are MadGraph's `one` rule.
  The rule is exact equality and covers masses and widths too. This session
  excludes the free spelling from it and compares against MadGraph's
  restricted model.
- `restriction-card-near-one-values-refused`: MadGraph loads these cards.
  The parameter is free, with a default of 1. Reproduced.
- `restriction-card-missing-ckm-entry-refused`: **MadGraph refuses these
  cards too.** The brief's premise, that MadGraph falls back to the UFO
  default, is wrong (L178-L184 above). See the oracle below for what the 22
  restrictions are. The refusal is kept with that reason, and its message now
  says MadGraph refuses the card.
- `ufo-full-restriction-unsupported`: reproduced, for the interned SM and
  for a UFO directory.
