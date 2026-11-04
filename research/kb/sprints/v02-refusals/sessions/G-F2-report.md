---
type: Session Report
title: "G-F2 report: required UFO fields, DECAY Auto, MadGraph's zero rule"
description: "51 field refusals and the restrict-card Auto refusal, each shown not refused on the pre-change code; MadGraph's 1e-13 rule with the strict rerun, which prunes three MSSM_SLHA2 vertices on rounding-residue couplings and leaves every other banked (model, card) vertex set unchanged."
status: draft
tags: [ufo, refusal, restrict-card, madgraph-parity, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Branch `sprintg-refusals-f2`, worktree `/home/user/wt/G1-F2`, base 851ae38.
Implementation commit `56e78ba`; this report is committed on top of it.

## What landed

1. **Required UFO fields** (`ufo/ast_util.rs` `Fields` and `FieldError`, used by
   `particles.rs`, `parameters.rs`, `couplings.rs` and `vertices.rs`). Every
   `require_args` field of `Particle` (L69), `Parameter` (L148), `Coupling`
   (L187) and `Vertex` (L170) in `mg5amcnlo/models/sm/object_library.py` is
   refused by name when it is missing or in a form the loader does not read.
   An external parameter's `lhablock`/`lhacode` are required as well (L161).
   The parameter `nature` must be `external`/`internal` and its `type` must be
   `real`/`complex`. An external `value` is a number and an internal `value`
   a string. `lhacode` is a list of integers, `order` a dict of strings to
   non-negative integers, a vertex `color` a list of strings and `couplings`
   a dict of `(int, int)` to `C.<name>`. Particle `mass`/`width` are
   `Param.<name>`. A parameter name defined twice is refused, as MadGraph's
   `load_model` refuses it ("define multiple time"), and so is a vertex name
   defined twice. The defaults kept are MadGraph's own
   (`goldstoneboson = False`, L74) plus optional fields that are not in
   `require_args` (`GhostNumber`, `line`, `propagator`). A malformed
   `GhostNumber` is refused and an absent one is 0.
2. **DECAY Auto.** A restrict card with an `Auto` width is refused with
   `UfoError::RestrictAutoWidth { pdg }` (`ParsedModel::apply_restriction`,
   now `Result`). This covers `UFOModel::load*`, `into_model(Some(card))` and
   the interned SM. `EvaluatedModel::from_model_card` panics on an `Auto`
   card (documented under `# Panics`). The other paths were already closed:
   `integrate --param-card` → `bind_card` `CardError::AutoWidth` (#82), and a
   reweight card's `set … Auto` is not a number to `parse_set`. The
   remaining `ParamCard` readers (`helas/eval/run.rs`, `rooting_soundness.rs`,
   `triple_gluon.rs`, `hadronic/tail_probe.rs`) read MadGraph-written cards
   and reference tables, and the `from_model_card` panic now covers them.
   Policy stated in `research/kb/model/restriction-semantics.md`.
3. **Zero rule** (`ZeroRule`, `ufo/mod.rs`). This is
   `detect_identical_couplings` (`import_ufo.py` L2549-L2558): over *all*
   couplings of the model, which matches MadGraph's `coupling_dict`, built
   by `OrganizeModelExpression.analyze_couplings` from `all_couplings`. If
   any `|g|` lies in `[1e-13, 1e-10)`, only exact zeros are pruned
   (`Strict`); otherwise `|g| < 1e-13` is pruned (`Threshold`). A NaN is
   never zero, which matches Python's comparisons.
   **`limit_to_6_digit`:** it only feeds the identical-coupling merge
   (`merge_iden_couplings`), and `optimise_interaction` then combines Lorentz
   structures that share one coupling. That renames couplings inside
   surviving interactions and adds or removes none, so the vertex set is
   unaffected. Not ported.
4. **Docs**: `model/restriction-semantics.md` (the rule, stated as a
   MadGraph-compatibility choice, with the scale-dependence a cleaner rule
   would fix; the `limit_to_6_digit` finding; the Auto policy) and
   `model/ufo-parsing.md` (required-fields policy).

## Brief corrections

- `research/refs/mg5amcnlo/models/object_library.py` does not exist. Each
  model ships its own; the pinned SM's is `models/sm/object_library.py`,
  cited above.
- "No banked restrict card is known to sit in that window" (backlog item
  `restriction-zero-coupling-threshold-differs`) is wrong. Every SMEFTsim
  card, all 13, has couplings in `[1e-13, 1e-10)` (`GC_309`–`GC_311` at
  2.8–4.2e-11 on all of them; `restrict_massless` has 19). So SMEFTsim is
  always restricted under MadGraph's *strict* rule. That is exactly why the
  old 1e-20 threshold agreed there: no SMEFTsim coupling lies in
  `(0, 1e-20)`.
- "Restriction must stay MadGraph-identical, vertex sets unchanged" cannot
  both hold for `MSSM_SLHA2`. The old rule kept three vertices that
  MadGraph's prunes, so becoming MadGraph-identical changes that vertex set.
  Details under "Census".

## Census: vertex sets on every banked (model, card) pair

A scratch test (not committed, at
`/tmp/claude-0/-home-user-vibegraph/623f339a-f10f-5e5f-84ea-95e77916e82c/scratchpad/G-F2-scripts/g_f2_census.rs`)
loaded every pair with `UFOModel::load` and dumped each vertex
(name, particles, Lorentz ids, `(colour, lorentz) → coupling`). It also
dumped every coupling with `0 < |g| < 1e-10` at the restriction's values.
It ran on the pre-change tree and again on the post-change tree. The pairs:
`sm` × {none, its 9 cards, `ckm_no_b_mass`}; SMEFTsim × {2 shipped, 11
`vg_*`}; `loop_sm` × {none, 11}; `MSSM_SLHA2` × {none, 4}; `taudecay_UFO`
(fails to load identically before and after: `Unknown Lorentz operator
'FFCT2'`); and the six toy UFOs × {none, their cards}. That is 56 loaded
pairs, 8700 vertex lines before and 8685 after.

```
$ diff G-F2-census-before.txt G-F2-census-after.txt   # every hunk is MSSM_SLHA2
< MSSM_SLHA2|-|NVERT|815                 > ...|812
< MSSM_SLHA2|-|V|V_13|a,W-,H+|LorentzId(9)|(0, 0)=GC_888
< MSSM_SLHA2|-|V|V_21|a,W+,H-|LorentzId(9)|(0, 0)=GC_888
< MSSM_SLHA2|-|V|V_89|A0,sl6+,sl6-|LorentzId(0)|(0, 0)=GC_857
  (the same three on restrict_default, no_b_mass, no_masses; NVERT 815→812,
   810→807, 807→804; no_tau_mass likewise)
```

`GC_888` = 4.44e-16 and `GC_857` = 8.88e-16 are rounding residues. An
`a W H±` vertex vanishes at tree level, and both values sit at a few ulp of
an O(1) cancellation. No MSSM coupling lies in the window, so MadGraph
applies `|g| < 1e-13` and logs them as "treated as zero". There is no
MadGraph install here to confirm it on MadGraph's own output, and no banked
MSSM diagram reference exists. Every other pair's vertex set, coupling keys
included, is byte-identical.

## Gates (commands run in this worktree, with `CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`)

| Command | Tail |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets --jobs 2 -- -D warnings` | `Finished`, exit 0 |
| `cargo clippy --workspace --all-targets --features vibegraph/extended-validation --jobs 2 -- -D warnings` | `Finished`, exit 0 |
| `cargo test --workspace --jobs 2` | exit 0; summed `test result` lines: 1817 passed, 0 failed |
| `cargo test -p vibegraph-lib --features extended-validation --test sm_interned_blob` (check-sm-blob-fresh) | `ok. 2 passed; 0 failed` |
| `cargo test -p vibegraph-lib --test coupling_oracle -- --nocapture` (validate-couplings, final step) | `ok. 50 passed; 0 failed` |
| `cargo test -p vibegraph-lib --test amplitude_oracle -- --nocapture` | `ok. 51 passed; 0 failed` |
| `cargo test -p vibegraph-lib --test smeftsim` (banked SMEFTsim interaction counts vs MadGraph) | `ok. 13 passed; 0 failed` |
| `cargo test -p vibegraph-lib --features extended-validation --test ufo` (loop_sm, MSSM_SLHA2, taudecay, sm from source) | `ok. 4 passed; 0 failed; 1 ignored` |
| `cargo test -p vibegraph-lib --test toy_models` | `ok. 13 passed; 0 failed` |
| `cargo test -p vibegraph-lib --config profile.release-debug.debug=0 --profile release-debug --test validate_madgraph_diagrams -- --nocapture` (validate-diagrams, final step) | `ok. 63 passed; 0 failed` |
| `python3 scripts/kb.py lint` | `1054 concepts, 340 backlog items; 0 errors, 5 warnings` (all five are pre-existing `state: blocked` backlog warnings) |

The pixi tasks were run as the commands they wrap; pixi is not installed.
Their MadGraph dependency steps (`generate-couplings`, `extract-diagrams`)
need MadGraph and did not run; the gates read the banked references. Not
run: `pixi run validate` end to end. The diagram gate's 63 rows matching
MadGraph's counts is the diagram-count evidence. It covers SM, SMEFTsim and
toy rows, not MSSM.

## Mutation evidence

New file `vibegraph-lib/tests/ufo_input_refusals.rs`, 13 tests. Run on the
pre-change source (`git stash push -- vibegraph-lib/src`, same test file):

```
$ cargo test -p vibegraph-lib --test ufo_input_refusals
test result: FAILED. 1 passed; 12 failed
```

The one pass is the fixture control `the_fixture_loads_and_keeps_both_vertices`.
The table tests stop at their first row, so a non-panicking copy
(`g_f2_sweep.rs`, scratch) recorded every case on both trees:

| | pre-change | post-change |
|---|---|---|
| field-refusal cases (particle 18, parameter 16, coupling 6, vertex 10, restrict Auto 1) | 51 / 51 not refused (48 load silently; vertex `color` ×2 and `lorentz` fail on an index-range error naming neither the vertex nor the field) | 51 / 51 refused, naming object and field |
| zero-rule cases | 6 / 8 wrong, the 2 exact-zero controls right | 8 / 8 right |
| `from_model_card` on an `Auto` card | `test did not panic as expected` | panics |

Examples of pre-change behaviour: `nature = 'exernal'` loads, `c1` becomes an
internal; `survivors("1e-15","0.5","1.0") = (true,true)`, i.e. kept;
`survivors("1e-25","1e-11","1.0") = (false,true)`, pruned where MadGraph's
strict rerun keeps it. The window cases also defeat a half-port: a 1e-13
threshold without the rerun prunes `1e-25` in all three window rows.

**What the tests cannot detect.** They read the error message for the object
and field name, so a refusal that names the right field for the wrong reason
passes. The zero-rule tests use one fixture with real couplings, so they
cannot see complex-valued couplings, where `norm()` and MadGraph's `abs()` are
the same hypot but rounding at a threshold edge could differ. They also
cannot see a model whose couplings differ from MadGraph's evaluation at the
ulp level near 1e-13 or 1e-10. The census is the vertex-set check across real
models, and it is not a comparison with MadGraph's own interaction list except
where `smeftsim.rs` banks one.

## Found

1. **`Lorentz` fields still default.** `lorentz.rs` `parse_lorentz`:
   `name` falls back to the Python name, a missing `structure` becomes `""`
   (MadGraph's default is `'external'`), and missing or non-list `spins`
   becomes `[]`. Left alone because the sibling session's scope touches
   `lorentz.rs`. Closing it means `Fields` with `name`/`spins` required and
   `structure` defaulting to `external` (refused as unsupported).
2. **`coupling_orders.py` defaults** (`ufo/mod.rs` `parse_coupling_orders`):
   a missing `name` becomes the Python name and a missing `hierarchy` becomes
   1. `CouplingOrder(name, expansion_order, hierarchy)` has no defaults
   except `perturbative_expansion`. Same fix with `Fields`.
3. **Duplicate particle, coupling and Lorentz names** go into name-keyed
   `IndexMap`s, where a later entry replaces an earlier one. Vertex
   resolution looks things up by Python name, so a duplicate usually
   surfaces as a "nonexistent" error rather than silently, but not always.
   Refuse like parameters and vertices.
4. **A parameter's UFO `name` differing from its Python variable is not
   checked.** The loader keys parameters by the variable and expressions
   reference names. Refuse a mismatch, or define which one is canonical.
5. **Stale "Known divergence" in `model/restriction-semantics.md`.** It says
   the loader locks only zeros, but `ParameterSet::ones` and
   `CardError::MovesFixedOne` now fix ones as MadGraph does. Still listed as
   a gap (unit-values backlog item).
6. **MSSM_SLHA2 vertex change unconfirmed against MadGraph.** Bank an
   MSSM_SLHA2 interaction count (as `smeftsim.rs` does for SMEFTsim) on
   `restrict_default` to pin the 812.
7. **The zero rule is scale-dependent** (MadGraph-compatibility site). It
   should be added to `madgraph-compat-sites-unconditional` (decision G3);
   that file is for the manager to edit.

## Follow-up

The manager asked for four changes after the first hand-off. They are made on the
same branch, after merging `sprintg-refusals` (G-F1) in as `f18d651`, which
conflicted only in the generated sessions index.

1. **No new panic: `EvaluatedModel::from_model_card` now returns
   `Result<Self, CardError>`** and refuses an `Auto` width with the
   `CardError::AutoWidth { pdg }` that `integrate --param-card` already gives.
   I chose to make it fallible rather than add a fallible twin, because that is
   the smaller *correct* change. The only callers outside `#[cfg(test)]` are
   `bind_card`/`bind_recorded`, which already return `CardError` and now use
   `?`, and `from_model`, which reads an empty card. `from_model` now calls a
   private, infallible `evaluate_at`, so it carries no `expect`. A twin would
   have left the public infallible constructor reachable from any card. Every
   other caller (`helas/eval/*`, `onshell`, `reweight/engine`, `lhef/write`,
   `hadronic/tail_probe`, 26 integration-test files) is test code reading
   MadGraph-written cards, and gets `.unwrap()`. No `panic!` on card input
   remains on this path.
2. **Required fields of the remaining UFO objects.**
   - `Lorentz`: `name` and `spins` (a list of integers) are required. `structure`
     defaults to `'external'` as UFO's constructor does (`object_library.py`
     L204). That default, and an explicit `'external'`, are refused as
     `LorentzError::ExternalStructure`, because such a routine is code outside
     the model. No banked model has one.
   - `coupling_orders.py`: `name` and `hierarchy` (a non-negative integer) are
     required. A file that does not parse is refused; it used to fall back to
     the SM hierarchy silently. `perturbative_expansion` defaults to 0 in
     `CouplingOrder.__init__` (L236), but the class never stores it and nothing
     here reads it, so there is no default to match.
     **`expansion_order` stays optional**, deviating from the letter of
     `CouplingOrder`, whose positional `expansion_order` is also required.
     MadGraph's import (`import_ufo.py`, the `try`/`except AttributeError` around
     `for order in all_orders`) explicitly tolerates an order without one and
     leaves the model uncapped. The loader already matched that, and
     `an_order_without_an_expansion_order_leaves_the_model_uncapped` pins it. A
     present but malformed `expansion_order` is refused.
   - Duplicate names: one `UniqueNames` (`ast_util.rs`) refuses a repeated UFO
     `name` *or* Python variable in particles (antiparticles included, so
     `.anti()` of a self-conjugate particle is refused, as UFO's `Particle.anti`
     refuses it), couplings, Lorentz structures, coupling orders, parameters and
     vertices.
3. **Parameter `name` vs variable: matched to MadGraph, `name` is canonical.**
   `model_reader.py` evaluates every value string with `param.name` bound
   (L203, L213). A `Param.<variable>` reference in `particles.py` is the
   object, and `UFOBaseClass.__str__` (L38) writes it as its `name`. So MadGraph
   accepts a mismatched model, and expressions name the UFO `name`. The loader
   keyed parameters by variable, so an expression naming the UFO name was
   refused as an unknown parameter.
   Now `parse_parameters_with_variables` keys the set by `name` and returns the
   variable → name map. `ParsedModel::parse` resolves each particle's mass and
   width through that map. A reference to no parameter is refused as
   `UfoError::UnknownParticleParameter`; it used to evaluate as a mass or width
   of zero (`unwrap_or(0.0)` in `EvaluatedModel::mass`). On every banked model
   each parameter's `name` equals its variable, so nothing banked moves. The
   tests are `a_parameter_is_known_by_its_ufo_name_not_its_variable` and
   `a_particle_parameter_reference_to_nothing_is_refused`.
4. **`model/restriction-semantics.md` "Known divergence"** is replaced by what
   the code does: ones are recorded in `ParameterSet::ones` and held at binding
   (`CardError::MovesFixedOne`), and identical values in `ParameterSet::merged`.
   Every toy-UFO restrict card leaves `WO8` at one, and `bind.rs` pins both
   rules. The Auto sentence there now says `from_model_card` refuses rather than
   panics. `model/ufo-parsing.md` gains the new coverage and the canonical-name
   rule.

### Mutation evidence

The new tests run against the pre-follow-up source, `f18d651` (`git stash push --
vibegraph-lib/src`). The Auto test was dropped from the copy because it cannot
compile against the old signature. The copy shadows `assert!`/`assert_eq!` with
recording macros, so every row of every table is recorded:

| | pre-follow-up (`f18d651`) | post |
|---|---|---|
| new refusal rows: Lorentz 5, external structure 2, coupling order 5, duplicates 5, dangling `Param` 2 | 19 / 19 not refused | 19 / 19 refused, naming object and field |
| how they were not refused | 11 load; 4 duplicates fail later on an unrelated "nonexistent particle/coupling/Lorentz" lookup (the overwrite's symptom); 4 fail on a structure-grammar or spin error that names neither the field nor, for spins, the structure | — |
| `a_parameter_is_known_by_its_ufo_name_not_its_variable` | panics: `Parameters(UnknownParameter { name: "MS", user: "MS2" })` | passes |
| `an_order_without_an_expansion_order_leaves_the_model_uncapped` (control) | passes | passes |
| all checks in the copy | 137 pass, 19 fail; 18 tests pass, 1 fails | 160 / 160 pass; 19 / 19 tests pass |

The Auto refusal was tested by source mutation, deleting the `auto_widths()` check
from `from_model_card`:

```
$ cargo test -p vibegraph-lib --test ufo_input_refusals evaluating_a_model
assertion `left == right` failed
  left: None
 right: Some(AutoWidth { pdg: 9000001 })
test result: FAILED. 0 passed; 1 failed
```

The original file was restored afterwards (`AutoWidth { pdg }` occurs twice in
`mod.rs`: the restrict refusal and `from_model_card`).

**What these tests cannot detect.** They are the same message-reading oracles as
before. The canonical-name test pins one crossing, an expression naming the UFO
name and a particle naming the variable. It does not pin a model where one
parameter's variable equals another's name.

### Gates (follow-up)

| Command | Tail |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets --jobs 2 -- -D warnings` | first run exit 101 (`type_complexity` on `parse_coupling_orders`); after the `CouplingOrders` alias: exit 0, `Finished` |
| `cargo clippy --workspace --all-targets --features vibegraph/extended-validation --jobs 2 -- -D warnings` | after the alias: exit 0, `Finished` |
| `cargo test --workspace --jobs 2` | on the final tree: exit 0; 51 result lines summed: 1831 passed, 0 failed (an earlier run, with the scratch census target present, also summed 1831 passed) |
| `--features extended-validation --test sm_interned_blob` | `ok. 2 passed; 0 failed` |
| `--test coupling_oracle` | `ok. 50 passed; 0 failed` |
| `--test smeftsim` | `ok. 13 passed; 0 failed` |
| `--features extended-validation --test ufo` | `ok. 4 passed; 0 failed; 1 ignored` |
| `--profile release-debug --test validate_madgraph_diagrams` | `ok. 63 passed; 0 failed` |
| vertex-set census, all 56 pairs (before the alias, which is type-only) | diff against the pre-change census: only the MSSM_SLHA2 hunks above (3 vertices × 5 configurations, and the 5 counts); diff against the first commit's census: empty |
| `python3 scripts/kb.py lint` | `0 errors, 5 warnings` (pre-existing) |
