---
type: Session Report
title: "F1 report: conditional parameter definitions refused by name"
description: "parameters.py definitions under 'if' branches are refused at load naming the parameter; undefined parameter references in parameters and couplings are refused at load, so expr::eval cannot panic on a loaded model."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-20}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-20}
sources:
  - {id: commit, resource: "git:HEAD of sprintd-ufo-conditional-params after F1", title: "F1 commit"}
---
## Choice: refuse

MadGraph does not supply `configuration`. `models/__init__.py:97-102` (pinned
b7687064) puts the model directory on `sys.path` and `__import__`s the UFO
package, so `configuration` is whatever module the UFO itself imports
(`import_ufo.py` has no `configuration` handling). No constant default exists in
a known place, so no branch can be chosen without running Python. The loader
refuses with `ParameterError::ConditionalDefinition`, naming every `Parameter`
assigned under an `if`/`elif`/`else` at any depth. I could not inspect
dim6top_LO_UFO (not in the repository).

## No panic

`expr::eval` still panics on a name missing from the map, and its doc now says
which load-time checks make that unreachable for a loaded model:
`parse_parameters` refuses an internal expression naming an undefined parameter
(`ParameterError::UnknownParameter`); model load refuses a coupling naming one
(`CouplingError::UnknownParameter`). I chose a load-time check over changing
`eval` to `Result` to keep the public `EvaluatedModel` API unchanged. Other
panic sites in `expr.rs`: the `parse::<f64>().unwrap()` is gated by the literal
grammar; function arity is fixed at one by the grammar, so `args[0]` is safe.

## Gate (worktree /home/user/wt/C)

- `cargo fmt --all --check` -> `fmt=0`
- `cargo clippy --workspace --all-targets -- -D warnings` -> `Finished ... clippy=0`
- `cargo test --workspace` -> `test=0`; 37 `test result: ok` lines, none with failures
- `python3 scripts/kb.py lint` -> `823 concepts, 289 backlog items; 0 errors, 4 warnings` (the four are pre-existing `blocked` items without `blocked_by`)

## Mutation evidence

- All three checks disabled (pre-fix behaviour): the three toy_models tests panic
  at `expr.rs:120` "UFO expression references unknown parameter 'gl_alt'"
  (likewise `gx_missing`, `gl_missing`).
- Conditional check off: `parameter_defined_only_under_a_conditional_is_refused_by_name` and unit `conditional_definitions_are_refused_by_name_at_any_depth` fail.
- Parameter-dependency check off: `internal_parameter_referencing_an_undefined_parameter_is_refused_by_name` and unit `internal_expression_naming_an_undefined_parameter_is_refused` fail.
- Coupling check off: `coupling_referencing_an_undefined_parameter_is_refused_by_name` fails.
- Constructor-name filter replaced by `true`: `conditional_non_parameter_statements_are_not_refused` fails.

## Found

- Other UFO files (`particles.py`, `couplings.py`, `vertices.py`) are read by
  `named_assignments`/`constructor_calls` at top level only; an `if` branch
  there is silently skipped, not refused. Close by applying
  `conditional_constructor_names` to each file's constructor.
- `EvaluatedModel::recompute` panics on an unknown parameter name passed by the
  caller (public API); an error return would close it.
