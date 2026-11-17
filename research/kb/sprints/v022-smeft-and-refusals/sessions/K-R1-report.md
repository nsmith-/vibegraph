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

## What changed

| commit | what |
|---|---|
| `d1b3ef4` | feat(ufo): the two free spellings load, `-full`, last-dash split, the missing-entry message |
| `6d116e1`, `3ceafff` | this report's reading, and the kb index |
| (final) | the MadGraph fixture test, the integration test rewritten, the items deleted |

- `ufo/parameters.rs`: `MADGRAPH_FREE_VALUES`. `apply_restrict` gives a
  `9.999999e-1` parameter a default of 1 and a `0.000001e-99` parameter a
  default of 0, and leaves both out of `zeros`, `ones` and `merged`. The
  couplings are still judged against the card as written, so a coupling
  linear in a free zero (about 1e-105) is pruned, as in MadGraph.
- `ufo/mod.rs`: the `RestrictMadgraphFree` refusal is gone. `RestrictMissing`
  stays and now says that MadGraph refuses such a card.
- `ufo/card_classes.rs`: the two spellings are `Consumed` rows. Each example
  must differ from the card with an exact 1 or 0, which differs only in the
  ones and zeros sets. The missing-entry row is refused because "MadGraph
  refuses it".
- `diagrams/parse.rs`: the restriction is what follows the last `-`.
- `ufo/loaded.rs`, `config.rs`: `LoadedModel::from_dir_full` and
  `interned_sm_full`. `sm-full` and `<dir>-full` apply no card. A directory
  named by the whole `name-suffix` is the model, as MadGraph tries it first.
  A `--restrict` override still wins over any suffix.
- `validation/madgraph/cards/smeft/restrict_vg_special_values.dat` (new):
  the vendored SMEFTsim's `restrict_SMlimit_massless.dat` with cG and cW at
  9.999999e-1, cH at 1, cHbox at 0.000001e-99, cHDD and cHW at 0.5, cHB at
  −0.5 and cHWB at −1. Its MadGraph dump is
  `vibegraph-lib/tests/data/restrict_vg_special_values_madgraph.json`.

## Oracles

MadGraph side: `kr1_mg_dump.py` imports the model through
`MasterCmd.exec_cmd('import model <spec>')`. It wraps
`detect_identical_couplings` and `detect_special_parameters` to record their
results, and dumps the remaining `('external',)` parameters with their
`parameter_dict` values, the rule card and the generated `param_card.dat`.
Vibegraph side: a throwaway `#[ignore]` test, not committed, dumps the same
from `ParsedModel::apply_restriction`. `kr1_compare.py` diffs the two. The
scripts are in `/home/user/wt/sprintk-restriction-mg/scripts/`.

**1. The CMS store's `SMEFTsim_U35_MwScheme_UFO_b_massless-cG_cHB_massless`**
(cG and cHB at 9.999999e-1), against MadGraph:

```
$ python kr1_compare.py out/SMEFTsim_U35_MwScheme_UFO_b_massless-cG_cHB_massless.json out/vg-u35.json
free: MG 14, vibegraph 14
  only MG: []
  only vibegraph: []
  default mismatches: []
zero couplings: MG 3223, vibegraph 3223, only MG [], only vibegraph [], strict MG False vg True
vibegraph ones {} merged []
MG rule one [] identical [] opposite []
```

MadGraph's generated card has `1 1.000000e+00 # cG` and
`8 1.000000e+00 # cHB`. The `strict` cell is an instrumentation artefact. The
wrapper records the outer call's argument last, and MadGraph's outer call
returns the inner `strict_zero=True` call's result (L2557-L2558). Vibegraph
finds three couplings in the window (`GC_233`–`GC_235`, |g| ≈ 3–4e-11), so
both sides run strict, and the 3223 zero couplings agree by name.

**2. The vendored SMEFTsim under `restrict_vg_special_values`**, against
MadGraph:

```
free: MG 19, vibegraph 19
  only MG: []
  only vibegraph: []
  default mismatches: []
zero couplings: MG 1126, vibegraph 1126, only MG [], only vibegraph []
vibegraph ones {"cH"}   merged [[("cHDD", 1.0), ("cHW", 1.0), ("cHB", -1.0)]]
MG rule one [['smeft', [3]]] identical [['smeft', [7], [5]]] opposite [['smeft', [8], [5]]]
```

MadGraph's card: `1 1.000000e+00 # cG`, `2 1.000000e+00 # cW`,
`4 0.000000e+00 # cHbox`, `5 5.000000e-01 # set of param :1*cHDD, 1*cHW, -1*cHB`,
`9 -1.000000e+00 # cHWB`. The test
`ufo::tests::the_special_values_card_restricts_as_madgraph_does` holds this
against the committed fixture. It checks the free set and defaults, the
ones, the merge, and the zero couplings by name.

**3. `-full`**, against MadGraph (`sm` and `sm-full`):

```
== sm        free: MG 15, vibegraph 15, only MG [], only vibegraph [], default mismatches []
             zero couplings: MG 36, vibegraph 36, only MG [], only vibegraph []
== sm-full   free: MG 26, vibegraph 26, only MG [], only vibegraph [], default mismatches []
```

`config::tests::sm_full_applies_no_restriction` pins that `sm-full` has no
zeros, ones or merges, frees every zero of `sm`, keeps more vertices and has
`parameters.py`'s externals. `full_on_a_ufo_directory_skips_its_default_restriction`
does the same on a UFO directory that ships `restrict_default.dat` (the toy
model, with its dipole card copied there): the bare name equals `-dipole`,
`-full` differs and frees `gtens`, and `toy-x` (a directory) is taken whole.

**4. A card that leaves out a parameter**, in MadGraph:

```
SMEFTsim_U35_MwScheme_UFO_b_massless-kr1nockmlambda_massless  MadGraph5Error: ckmblock (2,) not define
SMEFTsim_U35_MwScheme_UFO_b_massless-cG_cHQ1_massless         MadGraph5Error: Invalid restriction card (not same block)
    {'mass', 'sminputs', 'yukawa', 'smeftcpv', 'smeftcutoff', 'decay', 'smeft', 'switches'} != {..., 'ckmblock', ...}
```

`kr1nockmlambda_massless` is `cG_cHB_massless` without its `CKMlambda` line.

**5. The store's 9739 `SMEFTsim_U35_MwScheme_UFO_b_massless` cards**,
classified by what MadGraph does with them (`kr1_missing.py`):

| class | cards | MadGraph | vibegraph |
|---|---|---|---|
| every external set | 3565 (3472 use a free spelling) | loads | loads (one measured in full, oracle 1) |
| no `CKMBLOCK` | 6173 | "Invalid restriction card" (one measured, oracle 4) | refused, `RestrictMissing` ×4 |
| 118 entries missing (`cHt_cjtQd1Im_massless`) | 1 | refused | refused |

No card leaves out only `CKMlambda`. simshop's 22 named restrictions are not
on record here. By this census, a restriction refused for `CKMlambda` lacks
the whole block, and MadGraph refuses it. The 6173 cards also number their
`SMEFT` block for another model: `cG_cHQ1_massless` labels code 27 `cHQ1`,
which is `cHd` in this model. They were not built for this directory's
`parameters.py`. `restriction-card-missing-ckm-entry-refused` therefore
closes as "refusal kept with a recorded reason".

**6. Internal parameters MadGraph folds into constants.**
`detect_special_parameters` also catches internals that are exactly 0 or 1,
and `fix_parameter_values` makes them constants. MadGraph folded 37 on `sm`,
66 on `vg_special_values` and 64 on `cG_cHB_massless` (`mdl_CKM1x2`,
`mdl_dGf`, `mdl_propCorr`, ...). A throwaway test checked whether any of them
moves in vibegraph. For every free external with one of them downstream, it
moved that external alone (×1.1 + 0.01) and re-evaluated:

```
== sm                          checked 59 (internal, free external) pairs; none moves
== vg_special_values           checked 29 pairs; none moves
== cG_cHB_massless             checked 46 pairs; none moves
```

Every folded internal is 0 or 1 in vibegraph at the restriction's own
values. The 11, 5 and 15 names that are not UFO parameters
(`conjg__CKM1x2`, `lamWS__exp__2`, `propCorr__exp__2`, ...) are MadGraph's
abbreviations of conjugates and powers of folded parameters. On these three
cards, nothing MadGraph folds can be moved by a card here, so no amplitude
differs.

## Blind spots

- Oracles 1 to 3 compare the restriction's decisions: free set, defaults,
  fixed ones, merges, zero couplings. They do not compare amplitudes. Those
  follow from the same couplings and parameters, which the banked rows
  already gate.
- MadGraph's merging of identical couplings is a renaming that is not
  reproduced here. The comparisons are taken before it.
- Oracle 6 moves one external at a time, on three cards. An internal that is
  0 only for a combination of free values, or one on another card, is not
  covered.
- Only one card of the 3565 complete ones is compared in full against
  MadGraph.

## Corrections to the brief

- The brief asked for "a card that uses 9.999999e-1 and omits a CKM entry",
  so that MadGraph's restricted model could be compared on it. No such card
  loads in MadGraph. A card that leaves out an entry is refused
  (oracle 4), and the store has none that leaves out only one CKM entry
  (oracle 5). The comparison ran on a complete card that uses
  9.999999e-1 (oracle 1).
- "All 22 restrictions named in `restriction-card-missing-ckm-entry-refused`
  load" cannot hold, because MadGraph refuses them. The item names none.
  Oracle 5 classifies every card in that directory instead.
- The `madgraph` pixi environment ships MadGraph 3.5.7. The oracles ran the
  pinned 3.7.1 source with that environment's Python.

## Found

- **6173 of the store's 9739 `SMEFTsim_U35_MwScheme_UFO_b_massless` cards
  are for another model.** They have no `CKMBLOCK`, and their `SMEFT` codes
  follow another numbering (`cG_cHQ1_massless`: code 27 is `cHQ1` on the
  card and `cHd` in the model). MadGraph refuses them (oracle 4), and so does
  vibegraph. This is store data for simshop to report upstream, not loader
  work. It closes when simshop's re-scan names each refused U35 probe with
  its MadGraph verdict.
- **MadGraph folds internal parameters that are exactly 0 or 1 into
  constants.** The loader does not. No difference was found on three cards
  (oracle 6), but the general case is unchecked: an internal that is 0 only
  for some combination of free values would stay 0 in MadGraph and move here
  when a card moves its inputs. It closes with a check at load (refuse by
  name, or fold) for any internal that is exactly 0 or 1 while a free
  external is upstream of it.

## Gates

With `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=2`:

```
$ cargo clippy --workspace --all-targets -- -D warnings      # rc=0, no warnings
$ cargo test --workspace                                     # rc=0
  58 test binaries: 2119 passed, 0 failed, 27 ignored
  test ufo::tests::the_special_values_card_restricts_as_madgraph_does ... ok
  test madgraphs_free_one_and_zero_load_as_free_defaults ... ok
$ cargo fmt --all --check                                    # clean
$ pixi run kb-lint                                           # 0 errors, 5 warnings (pre-existing)
```

`madgraphs_free_one_and_zero_load_as_free_defaults`
(`tests/input_card_refusals.rs`) replaces the test that pinned the old
refusal. `validate.sh` was not run locally; CI's banked job is that gate. No
banked restrict card uses either spelling or `-full`, and no banked model
name holds a `-`, so no banked row's model should change.
