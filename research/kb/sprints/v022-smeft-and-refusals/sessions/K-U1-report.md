---
type: Session Report
title: "K-U1 report: UFO value forms and import model options"
description: "Seven of eight UFO-loader refusal classes closed against MadGraph's import with failing-first tests; the coupling-order-expression item is not closed, because the model it names is refused earlier by configuration-dependent parameters. MadGraph does not sum a vertex coupling list."
status: draft
tags: [ufo, loader, refusal, cms-gridpacks, report]
verified: [{by: claude-code/claude-sonnet-5-5, at: 2026-11-17}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-17}
---
Worktree `/home/user/wt/sprintk-ufo-values`, branch `sprintk-ufo-values`, off
`main` at `703f57f`. The host has 4 cores shared with sibling sessions; every
cargo command ran with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0` and the worktree's own `CARGO_TARGET_DIR`. No
MadGraph run and no `validate.sh`: the banked gate is CI's `banked` job. The
models below were downloaded from the CMS model store into
`/tmp/ku1-dl1` and symlinked under `/tmp/ku1-models`.

| commit | what |
|---|---|
| `71a3eb0` | fix(gridpacks): read the model of an import line that leads with `--modelname` |
| `c57eb47` | feat(ufo): read the value forms CMS-store models write (items 1, 3-7) |
| (this commit) | the report, seven closed items deleted, three census links turned to text |

Two of the commits are one commit each because items 1 and 3-7 edit the same
hunks of `ufo/mod.rs` and `ufo/parameters.rs`; they are separated below by
item, not by commit.

## Brief corrections

- **`ufo-vertex-coupling-lists-refused` says MadGraph sums the list. It does
  not.** `import_ufo.py` `add_interaction` (L1818-L1862) loops
  `for coupling in couplings` and files each member under
  `order_to_int[tuple(coupling.order.items())]`: a list is one interaction per
  distinct coupling order, exactly the split `split_vertices_by_coupling_order`
  already makes for separate keys. Two members of the *same* order at one
  `(colour, lorentz)` key overwrite each other
  (`order_to_int[order].get('couplings')[key] = ...`), the later winning
  silently. All 25 lists in EWdim6NLO pair an SM-order coupling with an `NP`
  one (checked by a script over its `vertices.py` and `couplings.py`).
- **`proc-card-import-model-options-refused`: the Rust half was already done.**
  `parse.rs` takes `-modelname`/`--modelname` out wherever it stands
  (`check_import`) and `check.rs` accepts it, with tests. What was left was
  gpdb's Python `parse_import`.
- **`ufo-coupling-order-expressions-refused` is not closeable by its own
  oracle** (see Found 1).

## What changed, by item

1. **`coupling-orders-not-inferred-without-coupling-orders-py`.** A model with
   no `coupling_orders.py` (or one declaring no order) takes the hierarchy of
   `Model.get_order_hierarchy` (`madgraph/core/base_objects.py` L1379): every
   order the vertices' couplings carry (`get_coupling_orders`, L1374) at 1,
   QED at 2 when the set is exactly `{QCD, QED}`. The old fixed
   `QCD=1, QED=2` default is gone. `ufo/mod.rs` `default_hierarchy`.
2. **`proc-card-import-model-options-refused`.** `validation/cms_gridpacks/
   parse.py` `parse_import` skips flags wherever they stand and returns them
   in order; `vgrun.py` `with_gridpack_model` (lifted out of a closure to be
   testable) keeps them and replaces only the model. The flag's reading is
   MadGraph's: it keeps the model's own particle names instead of renaming by
   PDG code, which the loader already did (`ufo/names.rs`), so for vibegraph it
   selects the naming and changes nothing else. `check.rs` pins the flag-first
   spelling on the HAHM line.
3. **Parameter value expressions and bare names** (`ufo/expr.rs`,
   `ufo/parameters.rs`, `ufo/ast_util.rs`). The grammar reads Python's
   `a if c else b` (only the selected branch evaluated), `== != < <= > >=`
   (one comparison, a chain is refused), `and or not` returning operands as
   Python's do, `max`/`min` (two or more arguments, first of equals kept) and
   `cmath.log10 sinh cosh tanh asinh acosh atanh`. A value written
   `'a' + 'b'` is one string (VPrime's `WZp`). A `value = NAME` reads an
   earlier module-level number (external) or an earlier `Parameter` variable,
   written as its UFO name (internal, as `str(Parameter)` does); a name nothing
   defined by that point is refused naming parameter and name.
4. **`ufo-vertex-coupling-lists-refused`.** A list entry is read as above:
   each member joins the interaction of its order; two members of one order at
   one entry are refused naming vertex, both couplings and the entry (MadGraph
   keeps the later without a word). Empty, mixed or non-`C.` lists are
   malformed-field refusals. `RawVertex`/`UnsplitVertex`, and
   `split_vertices_by_coupling_order` now returns a `Result`.
5. **`ufo-coupling-order-expressions-refused`, the coupling half.** An order
   value may be an integer or a `A if configuration.flag else B`, `not
   configuration.flag`, or `configuration.flag` expression, nested, with the
   flags read from `configuration.py`'s integer/boolean literals. Anything
   else, an unknown flag, a missing file, or a negative value is refused.
   **Not closed**, see Found 1.
6. **`ufo-particle-attribute-parameter-references-refused`.** `charge =
   n.value` (with `from parameters import n [as m]`) and `charge =
   Param.n.value` resolve to the external parameter's default in
   `parameters.py`, which is what MadGraph's `particle.set(key, value)` sees
   when `particles.py` is read. Parameters are now parsed before particles for
   this. An internal parameter, an unknown name, or any other attribute is
   refused naming the particle.
7. **`smeftsim-function-library-real-part`.** A new `ufo/functions.rs` reads
   `function_library.py` and selects `SecRe/CscRe/ASecRe/ACscRe` (appended to
   `Func`, so the interned SM's indices do not move) when the library writes
   `z.real`. A governed function (`complexconjugate re im sec csc asec acsc`)
   defined in a spelling outside the table, or with arguments other than `z`,
   is refused. **No banked SMEFTsim value moves:** its `parameters.py` and
   `couplings.py` call none of the four (test below), so the two evaluations
   cannot differ on it.

## Oracles

Every command below was run in the worktree.

**Failing first.** The Rust tests in `tests/ufo_value_forms.rs` against `src`
as of `71a3eb0` (before the Rust changes):

```
$ git checkout 71a3eb0 -- vibegraph-lib/src
$ cargo test -p vibegraph-lib --test ufo_value_forms
test a_bare_name_nothing_defines_is_refused_naming_it ... FAILED
test a_coupling_order_may_switch_on_a_configuration_flag ... FAILED
test a_coupling_list_is_one_interaction_per_order ... FAILED
test a_conditional_value_selects_by_the_default_of_its_condition ... FAILED
test a_model_with_exactly_qcd_and_qed_weights_qed_at_two_without_the_file ... ok
test a_particle_charge_that_is_a_parameters_value_loads ... FAILED
test a_value_that_is_a_bare_name_reads_that_name ... FAILED
test a_value_split_over_lines_with_plus_is_one_string ... FAILED
test a_particle_charge_naming_nothing_is_refused ... ok
test a_width_written_with_max_re_and_cmath_sqrt_evaluates ... FAILED
test two_couplings_of_one_order_in_a_list_are_refused ... FAILED
test orders_come_from_the_couplings_when_coupling_orders_py_is_absent ... FAILED
test a_coupling_order_outside_the_read_forms_is_refused ... ok
test result: FAILED. 3 passed; 10 failed
```

The three that pass before are guards, not failing-first tests: the QCD+QED
default is unchanged by the new rule, and two refusals existed already (the
new tests pin their message and that they stay refusals). Unit tests inside
`src` (the expression grammar, `functions.rs`, `particles.rs`, `vertices.rs`)
cannot be run against old source; they exercise new functions. The Python
tests, run with the old `parse.py`:

```
FAIL: test_import_flags_may_precede_the_model
AssertionError: Tuples differ: ('-', 'modelname', 'HAHM_MG5model_v3/HAHM_variableMW_v3_UFO') != ('HAHM_MG5model_v3/HAHM_variableMW_v3_UFO', None, '--modelname')
FAIL: test_the_model_rewrite_keeps_the_flags_and_replaces_only_the_model
AssertionError: 'impo[16 chars]abcd HAHM_MG5model_v3/HAHM_variableMW_v3_UFO\n...' != 'impo[16 chars]abcd --modelname\n...'
```

**After:**

```
$ pixi run --skip-deps test-gpdb
Ran 34 tests ... OK
$ cargo test -p vibegraph-lib --lib --test ufo_value_forms --test ufo_input_refusals
test result: ok. 1364 passed; 0 failed; 15 ignored   (lib, before the later unit tests)
test result: ok. 20 passed; 0 failed                  (ufo_input_refusals)
test result: ok. 13 passed; 0 failed                  (ufo_value_forms)
$ cargo test -p vibegraph-lib --lib ufo::
test result: ok. 185 passed; 0 failed
$ cargo fmt --all --check            -> clean
$ cargo clippy --workspace --all-targets -- -D warnings  -> rc=0
```

```
$ cargo test --workspace            (after the last source change; rc=0)
62 "test result: ok" lines, 0 result lines with a failure
```

**The real models** (`vibegraph --no-network enumerate <card> --ufo-dir
/tmp/ku1-models --svg none`, model directories from the CMS store):

| model | card process | result |
|---|---|---|
| `Bprime_no_b_mass` (no `coupling_orders.py`, couplings carry `QAD`) | `g b > t w- QAD=2` | `3 diagrams, 3 configurations`; before: refused as "QAD is not an order" |
| `EWdim6NLO` (lists in 25 vertices) | `e+ e- > w+ w- NP=0 / 1 / 2` | 3 / 3 / 5 diagrams; the NP=2 ones come from the split-by-order interactions |
| `loop_qcd_qed_sm_Gmu` (`aEW0 = '0.0 if aEW0M1==0. else 1./aEW0M1'`) | `e+ e- > mu+ mu-` | 2 diagrams |
| `DMPseudo` (`DMPseudoScalar_MonoB`, `WPhiee = max(1e-5, re(...))`) | `u u~ > b b~` | 2 diagrams |
| `VPrime_Allanach_DY3_UFO` (`WZp = '...' +'...'`) | `u u~ > e+ e-` | 4 diagrams |
| `axial_monoW_model` (`MZp value = MED`, `MED` bound nowhere) | `u d~ > w+ x x~` | `parameter 'MZp': value is the name MED, which parameters.py has not defined as a number or a parameter by that point` |
| `heco_spinhalf_withZ0` (`charge = n.value`) | `a a > HECO HECO~` | past model load; refused later at the custom propagator `F1` |
| `dim6top_LO_UFO` | `g g > t t~` | refused earlier, in parameters, see Found 1 |
| `EffDM_UFO` | `u u~ > e+ e-` | `parameter 'mDM' is defined twice`, see Found 3 |

`DMAxial_monow012j` is refused by name, which is the item's accepted outcome.
`DMScalar` and `DMsimp_s_spin0_4f` (same `WPhiee` form), `NMSSMHET`,
`NMSSMHET-NoY` and the gridpack cards themselves were not run.

## What each test provably cannot detect

- The `ufo_value_forms` loads and the CMS-model enumerations show a form is
  read and gives *a* number. They do not compare the number with MadGraph's:
  no amplitude, width or coupling was checked against a MadGraph param card
  or `model_reader` evaluation. The expression unit tests compare against
  Python-semantics values computed by hand, so a misreading of Python
  precedence that the author shares would pass.
- Ordered comparisons compare the real parts of complex values, where Python
  raises `TypeError`; a model that orders complex numbers would be accepted
  here and fail in MadGraph.
- Default hierarchy: `get_coupling_orders` reads the interactions MadGraph
  keeps after import, and `RestrictModel` later removes the hierarchy of an
  order no surviving interaction carries (`import_ufo.py` L2478-L2488). The
  loader derives the set from every vertex before restriction, so a restricted
  model can know an order MadGraph has dropped. The tests use no restriction.
- A particle charge is the `parameters.py` default, as MadGraph's import sees
  it. A card that sets `n` (the HSCP gridpacks do, one charge per point) does
  not change the charge in either program, if MadGraph indeed does not
  re-read it; this was not checked against a MadGraph run.
- The coupling-list test uses lists of differing order; it cannot say what a
  same-order list should mean, only that it is refused.
- The SMEFTsim "no value moves" test walks the expression trees for the
  governed calls. It would not see a model that reaches them through a
  function it does not evaluate, and the claim covers only the vendored
  `SMEFTsim_topU3l_MwScheme_UFO`, not other SMEFTsim variants in the store.
- Items 2 and its real oracle: neither HAHM gridpack's card was fetched, so
  "gets past proc-card parsing" is shown on the line the item quotes
  (`import model --modelname HAHM_MG5model_v3/HAHM_variableMW_v3_UFO`), not on
  the card.

## Where K-R1's files were touched

`ufo/parameters.rs`: `parse_parameters_with_variables` gained a
`&FunctionLibrary` argument and a loop over `named_assignments`; a method
`default_of_variable` was added to `ParameterSet`. `apply_restrict` is not
touched. `ufo/mod.rs`: `ParsedModel::parse` parses parameters before particles
and reads two more optional files; the `import model` suffix handling and the
restrict path are not touched.

## Found

1. **`dim6top_LO_UFO` is refused before its coupling orders are read.**
   `parameters.py` defines `MW`, `aEWM1`, `aEW` under `if
   configuration.mw_ew_input:` / `else:` (L3818) and writes values as `'G' if
   configuration.norm_chromo_gs else '1.'` (L3809) and `'(2*MW*sw)/vev' if
   configuration.mw_ew_input else '2*cmath.sqrt(aEW)*cmath.sqrt(cmath.pi)'`
   (L3866). The loader refuses the conditional definitions
   (`parameter(s) 'MW', 'aEWM1', 'aEW', 'MW' are defined only inside an if
   branch`), so `tZq_EFT_CPV_point_1_13p6` still does not get past model load
   and `ufo-coupling-order-expressions-refused` stays open on its stated
   oracle. Closing it takes the same `configuration.py` flags applied to `if`
   statements around `Parameter` definitions and to string-valued `value`
   conditionals; the coupling-order evaluator is the piece to share.
2. **A same-order coupling list is refused where MadGraph keeps the later
   member.** No model read here has one. If a store model does (NMSSMHET and
   NMSSMHET-NoY were not run), decide between reproducing the overwrite and
   keeping the refusal.
3. **`EffDM_UFO` is refused with `parameter 'mDM' is defined twice`**, ahead
   of the vertex-list form the item listed it under. MadGraph's import also
   refuses a repeated name, so this is probably the model, not the loader;
   `EffDM_222_restricted` may be the intended one.
4. **A governed function a library leaves undefined is still evaluated.**
   `FunctionLibrary` falls back to the SM definitions when `function_library.py`
   exists but does not define `sec` and the like, where MadGraph raises a
   `NameError`. Refusing it needs a decision on models that ship no library
   at all (the toy models use `complexconjugate` without one).
5. **`hierarchy` and `get_coupling_orders` after restriction**: MadGraph drops
   the orders of removed interactions, the loader does not (blind spot 3). It
   moves only `order_hierarchy`'s key set, which `resolve.rs` reads to decide
   whether `QAD=2` names an order.
6. **`heco_spinhalf_withZ0` is refused at `propagators.py` `F1`** once past
   load; the HSCP gridpacks have a second wall after item 6.

## Not run

`pixi run validate` (CI's `banked` job is the gate), the
`extended-validation` targets (`ufo`, `sm_interned_blob` and the others
feature-gated), and any MadGraph run. The interned SM's `Func` indices are
unchanged by construction (new variants are appended); `sm_interned_blob` is
the test that would show otherwise and is in CI's banked layer.
