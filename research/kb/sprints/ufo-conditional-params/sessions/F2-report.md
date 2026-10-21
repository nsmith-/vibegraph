---
type: Session Report
title: "F2 report: no panic on an unknown parameter from a public entry; conditional objects refused"
description: "recompute and set_alpha_s return an error naming an unknown parameter; conditional Particle, Lorentz, Coupling, Vertex, CouplingOrder and Propagator definitions are refused at model load."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-20}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-20}
sources:
  - {id: commit, resource: "git:92152df", title: "F2 commit 92152df"}
---
## 1. No panic from a public entry

**Chosen:** keep the `expr::eval` panic as an internal invariant; make every
public path that takes a name from a caller fallible.

- New `ufo::UnknownParameter(String)` error (`the model has no parameter '...'`).
- `EvaluatedModel::recompute` and `EvaluatedModel::set_alpha_s` return
  `Result<(), UnknownParameter>`; nothing changes on error.
- Callers changed: `helas/eval/rescale.rs` (`set_alpha_s` on the amplitude keeps
  its infallible signature; a model with no `aS` has no strong coupling to move, so
  it records the value and leaves the pools alone; the sqrt-law probe returns
  `NoReferenceCoupling`), `reweight/engine.rs` (four sites, `expect` with the
  invariant that launch parameters were checked against the model when the card
  was read: `reweight/mod.rs` raises `ReweightError::UnknownParameter`
  first, and `generic_point` already indexes `param_values[name]`), tests in
  `ufo/mod.rs`, `reweight/engine.rs`, `helas/eval/rescale.rs`, `tests/ufo.rs`,
  `tests/validate_scale_couplings.rs` (feature-gated; checked with
  `--features extended-validation,bench-internals`).

**Cost of the alternative** (`eval -> Result`): `eval` is `pub(crate)`, but the
`Result` would thread through `ParameterSet::evaluate`, `recompute`,
`EvaluatedModel::from_model_card`/`from_model` and the restriction pruning, i.e.
the infallible public constructors used by hundreds of call sites in the CLI,
benches and tests, for a case the load-time checks already exclude.

**Every public path to `eval`, and its guard** (also in the `eval` doc):

| Path | Guard |
|---|---|
| `UFOModel::load` / `ParsedModel::parse` (restriction pruning calls `evaluate`) | undefined parameter in an internal expression refused (`ParameterError::UnknownParameter`); in a coupling refused (`CouplingError::UnknownParameter`); conditional definitions refused |
| `EvaluatedModel::from_model_card`, `from_model` | `ParameterSet::evaluate` inserts every external and internal parameter (restriction-zeroed as 0.0); couplings checked at load |
| `EvaluatedModel::recompute`, `set_alpha_s` | name checked, returns `UnknownParameter` |
| `ParameterSet::recompute` (crate-private) | reached only through `EvaluatedModel::recompute` |

Not covered: a `UFOModel` deserialized from bytes (the interned SM blob) is
trusted to come from the checked loader; and `param_values` is a `pub` map, so a
caller who removes an entry from it can still reach the panic.

## 2. Conditional objects

`ufo::mod::refuse_conditional_definitions` (one parse of the file) applies
`conditional_constructor_names` to `particles.py` (`Particle`), `lorentz.py`
(`Lorentz`), `couplings.py` (`Coupling`), `vertices.py` (`Vertex`),
`coupling_orders.py` (`CouplingOrder`) and `propagators.py` (`Propagator`),
returning `UfoError::ConditionalDefinition { file, names }`. Parameters keep
their own refusal in `parse_parameters`. Names are the Python variable names.

## Tests

- `tests/toy_models.rs`: `recompute_of_an_unknown_parameter_is_an_error`,
  `conditional_particle_and_vertex_definitions_are_refused_by_name`,
  `conditional_coupling_and_lorentz_definitions_are_refused_by_name`;
  `try_edited_toy_model` now takes `(file, text)` appends.
- `ufo/mod.rs`: `recompute_refuses_an_unknown_parameter_and_changes_nothing`.

## Gate (/home/user/wt/C)

- `cargo fmt --all --check`: `fmt=0` (after `cargo fmt --all` reflowed the new code).
- `cargo clippy --workspace --all-targets -- -D warnings`: `clippy=0`.
- `cargo test --workspace` (unfiltered): `test=0`; 37 `test result: ok`, none failing;
  the new tests listed `ok`, including the `toy_models` binary.
- `cargo check --workspace --all-targets --features extended-validation,bench-internals`: clean.
- `python3 scripts/kb.py lint`: 0 errors after `kb.py index` (F1's report had left
  `sessions/index.md` stale); 4 pre-existing warnings.
- Not run: the `validate` MG net.

## Mutation evidence

- `recompute` back to `unwrap_or_else(panic!)`: `recompute_of_an_unknown_parameter_is_an_error`
  and `recompute_refuses_an_unknown_parameter_and_changes_nothing` panic
  ("attempted to recompute unknown parameter").
- `refuse_conditional_definitions` made to find nothing: both new conditional toy
  tests fail (the Parameter test still passes: separate path).

## Found

- `EvaluatedModel::param_values` is a `pub` field; mutating it can reach the
  `eval` panic. Making it private would close it.
- A conditional `x.anti()` or other non-constructor statement under an `if` in
  `particles.py` is still not seen; only constructor calls are refused.
