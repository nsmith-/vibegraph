---
type: Session Report
title: "L-U2 report: a configuration if with no Parameter under it loads again, and a refused if names its own parameters"
description: "An out-of-subset if of parameters.py that defines no Parameter is skipped as in v0.2.2; a refused if names only the Parameters in its own branches; a bare-name value last bound under a skipped if is a named refusal."
status: draft
tags: [ufo, model-loading, refusal, regression]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-config-if` off main 033044e4. Fix commit c6335a01. No backlog
item is closed (the regression was never released, so none was filed).

## What changed

`vibegraph-lib/src/ufo/parameters.rs`. `flatten_branches` (which turned any
out-of-subset `if` into a refusal listing every conditional `Parameter` of the
file) is replaced by `select_statements`:

- an `if` whose test the configuration subset evaluates is replaced by the
  branch it selects, at any depth (unchanged: L-U1's feature);
- an `if` whose test is outside the subset and that defines a `Parameter` in
  any branch is refused, `ConditionOutside` naming the `Parameter`s of *that*
  `if` only (`conditional_definitions` on the one statement), so the list is
  never empty and never another `if`'s;
- an `if` whose test is outside the subset and that defines no `Parameter` is
  kept as a statement the parameter loop does not read, which is v0.2.2's skip.

The names an assignment under a skipped `if` binds (`bound_names`: plain, tuple,
starred, augmented and annotated targets, nested `if`s) become *unknown* from
that point until a module-level assignment rebinds them. A `Parameter` whose
`value` is a bare name that is unknown is refused with the new
`ParameterError::ConditionalValueName` ("parameter 'P': `value` is the name
`plain`, which parameters.py last binds under an `if` whose condition is
outside the subset …"). `flatten_branches` and its unit test are removed from
`config_eval.rs`; the test moved to `parameters.rs` as
`statements_are_the_branches_that_run_and_the_skipped_ifs`.

**v0.2.2's code** (`git show v0.2.2:vibegraph-lib/src/ufo/parameters.rs`,
`parse_parameters_with_variables`):

```rust
    let stmts = parse_stmts(content).map_err(|e| ParameterError::Parse(e.to_string()))?;

    if let Some(names) = conditional_definitions(&stmts, "Parameter") {
        return Err(ParameterError::ConditionalDefinition { names });
    }
    ...
    for (python_name, assigned) in named_assignments(&stmts) {
```

`named_assignments` reads module-level `Assign` statements only, so every `if`
with no `Parameter` under it was skipped whatever its test, and a name bound
only inside one was not in `numbers`.

**Deviation from the brief (scope 1), with reason.** The brief says such an
`if` is skipped "whatever its condition". The fix skips only the ones whose
test is outside the subset; a `Parameter`-free `if` the subset evaluates keeps
main's behaviour (the selected branch's assignments bind). v0.2.2 skipped
those too, and that skip is a silent wrong answer, not a refusal:
`plain = 1.0` / `if configuration.flag: plain = 2.0` /
`P = Parameter(value = plain)` gives P = 1.0 under v0.2.2 with `flag = True`,
where MadGraph's Python gives 2.0. Main and this branch give 2.0, pinned by
the last assertions of `a_value_naming_a_name_bound_under_a_skipped_if_is_refused`.
Every model v0.2.2 loads with no conditional `Parameter` still loads; the
only difference from v0.2.2 is that such a name read later gets the Python
value instead of the earlier binding or a refusal.

**What v0.2.2 does with a name bound under a skipped `if` and read later**
(scope 1's second point), measured with the v0.2.2 binary on three variants of
`sm_if` with `plain = 2.0` under the `if` and `P` appended (models in the
session scratchpad, `models/sm_r{a,b,c}`):

```
$ vibegraph-v022 --no-network enumerate ra.proc --ufo-dir models --out out22-ra --svg none
  (sm_ra: P = Parameter(..., value = plain, ...))
ERROR failed to load model: Parameter parse error: parameter 'P': `value` is the name `plain`, which parameters.py has not defined as a number or a parameter by that point
$ ... rb.proc   (sm_rb: `plain = 1.0` before the if, then the same P)
total:    2 diagrams in 1 subprocess          <- loads, P = 1.0 silently
$ ... rc.proc   (sm_rc: P = Parameter(..., value = "plain", ...))
ERROR failed to load model: Parameter parse error: internal parameter 'P' references 'plain', which parameters.py does not define
```

This branch refuses all three by name (sm_rb included, where v0.2.2 was
silently wrong):

```
sm_ra: ERROR ... parameter 'P': `value` is the name `plain`, which parameters.py last binds under an `if` whose condition is outside the subset of Python the loader evaluates against configuration.py: ...
sm_rb: ERROR ... (the same message)
sm_rc: ERROR ... internal parameter 'P' references 'plain', which parameters.py does not define
```

**Release note:** none. `docs/release-notes/v0.2.3.md` does not mention
configuration support (`grep -i configuration` finds nothing), so no line in
it is wrong.

## Tests (hermetic, `vibegraph-lib/tests/ufo_value_forms.rs`)

- (a) `an_if_outside_the_subset_that_defines_no_parameter_loads`: the `sm_if`
  shape (`if configuration.x > 1: plain = 1.0`, `x = 3`) loads, and every
  parameter value equals the model's without the `if`.
- (b) `a_refused_if_names_only_the_parameters_under_it`: an in-subset `if`
  defining `EWK` and an out-of-subset `if`/`else` defining `extra` and
  `other`; the message contains exactly
  `parameter(s) 'extra', 'other' are defined under an `if`` and not `'EWK'`.
  The exact-substring assertion also fails on an empty list.
- (c) `a_value_naming_a_name_bound_under_a_skipped_if_is_refused`: external
  and internal `value = plain`, with and without an earlier `plain = 1.0`, each
  refused naming `'P'` and `` `plain` ``; a module-level rebinding after the
  `if` reads that number (3.0); under an in-subset `if` the branch that runs
  binds (2.0 / 1.0).

**Fail on main, pass with the fix** (`git stash push --` the two `src` files,
keep the test file):

```
$ cargo test -p vibegraph-lib --test ufo_value_forms      # main's src + new tests
test a_refused_if_names_only_the_parameters_under_it ... FAILED
test a_value_naming_a_name_bound_under_a_skipped_if_is_refused ... FAILED
test an_if_outside_the_subset_that_defines_no_parameter_loads ... FAILED
thread 'a_refused_if_names_only_the_parameters_under_it' panicked at vibegraph-lib/tests/ufo_value_forms.rs:531:5:
Parameter parse error: parameter(s) 'EWK', 'extra', 'other' are defined under an `if` of parameters.py whose condition is outside the subset ...
thread 'a_value_naming_a_name_bound_under_a_skipped_if_is_refused' panicked at vibegraph-lib/tests/ufo_value_forms.rs:555:9:
"" external: Parameter parse error: parameter(s)  are defined under an `if` of parameters.py whose condition is outside the subset ...
thread 'an_if_outside_the_subset_that_defines_no_parameter_loads' panicked at vibegraph-lib/tests/ufo_value_forms.rs:510:56:
called `Result::unwrap()` on an `Err` value: "Parameter parse error: parameter(s)  are defined under an `if` ...
test result: FAILED. 16 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo test -p vibegraph-lib --test ufo_value_forms      # with the fix
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
$ cargo test -p vibegraph-lib --lib ufo::
test result: ok. 210 passed; 0 failed; 0 ignored; 0 measured; 1291 filtered out
```

The unit test `statements_are_the_branches_that_run_and_the_skipped_ifs` calls
the new functions, so it does not compile on main; it pins that nested
in-subset branches select one assignment (`q`), that a nested out-of-subset
`Parameter`-free `if` survives as one statement, and that `bound_names` reads
`s, (t, *u) = …` and `v += …` as `s t u v`.

**What the tests cannot detect.** (a) compares against the same fixture without
the `if`, so it cannot see a change that moves both alike (the binary identity
oracle below covers that for real models). (b) checks one two-`if` layout; a
`Parameter` defined under `for`/`with`/`try` is invisible to
`conditional_definitions` (as in main and v0.2.2) and none of these tests
exercises it. (c) covers bare-name values only: a string expression reads UFO
`name`s, not Python variables, so `value = 'plain'` falls to the existing
`UnknownParameter` refusal, which these tests do not re-pin. A name bound under
a skipped `if` through a statement `bound_names` does not walk (`for` targets,
`import`, walrus) is not marked unknown; a later bare-name read then gives
whatever main gives (the earlier binding or `UnresolvedValueName`).

## Binary oracles

Binaries: this branch's `target/debug/vibegraph` (`v0.2.2-29-g8154b298-dirty`,
c6335a01's tree); main's at `/home/user/wt/z512-main/target/debug/vibegraph`
(`v0.2.2-27-g8b3af5ab`, clean checkout); v0.2.2's
`/tmp/claude-0/lp/z512/bin/vibegraph-v022`. Driver: `lu2/oracle.sh <binary> <tag>`
in the session scratchpad, running `parameters` and `enumerate` as below.

**sm_if against v0.2.2:**

```
$ vibegraph --no-network enumerate sm_if.proc --ufo-dir z512/models --out out-fix/e-sm_if --svg none
enumerate sm_if rc=0                                   (main: rc=1, "parameter(s)  are defined ...")
$ cmp z512/out22-sm_if/diagrams.json out-fix/e-sm_if/diagrams.json && echo raw-identical
raw-identical
$ cmp out-fix/e-sm_if/diagrams.bin.zst z512/out22-sm_if/diagrams.bin.zst && echo "bin identical"
bin identical
```

`diagrams.json` carries no version field (only `"schema_version": 1`), so no
mask was needed.

**sm_ifp still refused, naming `'extra'` alone:**

```
enumerate sm_ifp rc=1
ERROR failed to load model: Parameter parse error: parameter(s) 'extra' are defined under an `if` of parameters.py whose condition is outside the subset ...
```

**sm_ctl byte-identical to main:**

```
$ cmp out-main/e-sm_ctl/diagrams.json out-fix/e-sm_ctl/diagrams.json && cmp out-main/e-sm_ctl/diagrams.bin.zst out-fix/e-sm_ctl/diagrams.bin.zst && echo identical
identical sm_ctl diagrams.json+bin
```

**`vibegraph parameters` identity with main.** Cards: `import model sm`;
`import model sm-no_b_mass` / `generate p p > e+ e- j`; `import model sm_ctl`
(the SM read from files, so the parser runs) / `generate e+ e- > mu+ mu-`;
`import model dim6top_LO_UFO` / `generate g g > t t~`;
`import model SMEFTsim_topU3l_MwScheme_UFO_ctGpatched-massless` /
`generate e+ e- > t t~` (the last two are the copies L-U1 unpacked in the
scratchpad, `lu1-models/`). All five exit 0 on both binaries.

```
identical p-sm/param_card.dat            (and p-sm_nob, p-sm_ctl, p-dim6top, p-ctg: all byte-identical)
masked-identical p-sm/parameters.json    (and the other four)
masked-identical p-sm.stdout             (and the other four)
masked-identical p-sm.stderr
$ diff out-main/p-sm/parameters.json out-fix/p-sm/parameters.json
3c3
<   "version": "v0.2.2-27-g8b3af5ab",
---
>   "version": "v0.2.2-29-g8154b298-dirty",
```

`parameters.json` differs only in its `version` line (the git description),
stdout only in the `--out` path; the other four stderr files differ only in
the logged enumeration wall time (`(0.010 s)` vs `(0.015 s)`).

## Gates

All on head c6335a01, cargo under `flock /home/user/wt/heavy.lock` with the
brief's `CARGO_TARGET_DIR` exports.

```
$ flock /home/user/wt/heavy.lock /tmp/claude-0/lp/verify-hidden.sh /home/user/wt/sprintl-config-if lu2
head c6335a01
FMT_OK                                         # cargo fmt --all --check
    Finished `dev` profile [optimized] target(s) in 43.80s
CLIPPY_RC=0                                    # clippy --workspace --all-targets -D warnings
    Finished `dev` profile [optimized] target(s) in 24.52s
CLIPPY_EV_RC=0                                 # the same with vibegraph{,-lib}/extended-validation
TEST_RC=0                                      # cargo test --workspace --no-fail-fast, three data paths hidden
lines=79 passed=2355 failed=0

$ cargo test -p vibegraph-lib --features extended-validation --test sm_interned_blob   # check-sm-blob-fresh
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
$ cargo test -p vibegraph-lib --test coupling_oracle -- --nocapture                    # validate-couplings, existing generated data
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
$ cargo test -p vibegraph-lib --features extended-validation --test ufo --test toy_models --test smeftsim --test ufo_value_forms --test ufo_input_refusals
smeftsim            test result: ok. 14 passed; 0 failed; 0 ignored
toy_models          test result: ok. 19 passed; 0 failed; 0 ignored
ufo                 test result: ok. 5 passed; 0 failed; 1 ignored
ufo_input_refusals  test result: ok. 20 passed; 0 failed; 0 ignored
ufo_value_forms     test result: ok. 19 passed; 0 failed; 0 ignored

$ pixi run --skip-deps kb-lint
1394 concepts, 480 backlog items; 0 errors, 7 warnings   # all 7 pre-existing backlog warnings, none in this report
```

The `extended-validation` layer was run as the cargo commands the pixi tasks
wrap (`check-sm-blob-fresh`, `validate-couplings`), not through pixi, so no
dependency task regenerated reference data; the `ufo` test's one ignored test
is ignored on main too.

## Long-run requests

None.

## Corrections to the brief

- "Also check is byte-identical to main on sm, sm-no_b_mass and the dim6top
  and ctGpatched models" is garbled: it means `vibegraph parameters` output is
  byte-identical to main on those four models (sm being the required one).
- `vibegraph parameters` on `import model sm` uses the interned SM blob, not
  `parameters.py`, so it cannot see a parser change at all. The identity check
  that exercises the parser on the SM is `sm_ctl` (the SM loaded from files),
  added above.
- "Byte-identical" cannot hold for `parameters.json` between two builds: it
  records the binary's git description. The check is byte-identical with the
  `version` value masked; `param_card.dat` is byte-identical unmasked.
- `vibegraph-v022` has no `parameters` subcommand, so the v0.2.2 comparisons
  use `enumerate`.
- Scope 1's "skipped, whatever its condition" would regress main's correct
  reading of an in-subset `Parameter`-free `if` into v0.2.2's silent wrong
  value; see the deviation above.

## Found

- **A `Parameter` defined under `for`, `while`, `with` or `try` in
  parameters.py is silently ignored.** `conditional_definitions` and
  `named_assignments` walk module-level `Assign` and `if` only, so such a
  definition is neither loaded nor refused (in main, v0.2.2 and this branch).
  Evidence: `ast_util.rs` `conditional_constructor_names` matches only
  `Stmt::If`. Closing: refuse any `Parameter` constructor call outside a
  module-level assignment or an evaluated `if` branch, by name.
- **The other UFO files keep v0.2.2's blanket rule.** `refuse_conditional_definitions`
  (`ufo/mod.rs`) still refuses any `Coupling`/`Particle`/... under any `if`,
  configuration-evaluable or not; only `parameters.py` evaluates the subset.
  No model in hand needs more, so this is a note for when one does, not a bug.
