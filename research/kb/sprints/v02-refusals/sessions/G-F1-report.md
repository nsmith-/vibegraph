---
type: Session Report
title: "G-F1 report: spin 2 and Lorentz indices naming no leg are refused at compile"
description: "AmplitudeEvaluator::compile refuses any external or internal line of spin code outside 1-3, naming the particle; Lorentz rooting refuses an index at or above the vertex's leg count, naming the structure; build_child's todo! (reached only by C as a scalar factor) returns UnsupportedVertex; the three spin-code panic! arms are unreachable! behind the compile refusal. Five new tests, each shown panicking on the pre-change code; workspace tests, clippy (both feature sets), fmt and kb lint pass."
status: draft
tags: [refusal, spin, non-sm-ufo, panic, helas, lorentz, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-11-03/04, shared with one sibling session; cargo --jobs 2, CARGO_INCREMENTAL=0, debug info off"
  commit: "change 3f89868 on sprintg-refusals-f1; base 851ae38"
  command: "see Gates; logs under the session scratchpad, prefixed G-F1-"
sources:
  - {id: brief, resource: "G-F1.md", title: "G-F1 brief"}
  - {id: compile, resource: "../../../../../vibegraph-lib/src/helas/eval/compile.rs", title: "refuse_unsupported_spins"}
  - {id: root-lorentz, resource: "../../../../../vibegraph-lib/src/helas/eval/root_lorentz.rs", title: "reject_indices_naming_no_leg, scalar_root's C arm, RootLorentzError::in_structure"}
  - {id: toy, resource: "../../../../../vibegraph-lib/tests/toy_models.rs", title: "spin_two_toy_model and the refusal tests"}
---
## Verdict

Both items' close conditions are met under option (a) of decision G1.

- **Spin 2, internal and external**, is refused by `AmplitudeEvaluator::compile`
  with `EvalError::UnsupportedSpin { particle, spin, line }`:
  - `lt~ lt > vt vt NP<=2` → `particle 'gt' has spin code 5, which has no propagator here (spin 3/2 and spin 2 are not supported)`
  - `lt~ lt > gt NP<=1` → `particle 'gt' has spin code 5, which has no external wavefunction here (spin 3/2 and spin 2 are not supported)`
- **A Lorentz index naming no leg** is refused in `LorentzEvalTree::build_at_leg`
  before any walk, and the vertex names the structure:
  `invalid structure: Lorentz structure FFV1 (Gamma(1003,2,1)): index 1003 of Gamma { mu: 1002, i: 1, j: 0 } names no leg of a 3-leg vertex (spin-2 tensor indices are not supported)`.
- **The four hygiene sites.** (1) `scalar_root`'s `todo!` arm (the brief and item
  call it `build_child`'s; it is in `scalar_root`, see Brief corrections) is now
  an explicit `LorentzOp::C` arm returning `UnsupportedVertex` naming the
  operator. The match is exhaustive, so an operator added later fails to build
  until it is routed. (2) `build_external_core`, (3) `external_out_type` and (4)
  `spin_state_count` are `unreachable!` with the invariant ("compile refuses
  every spin code but 1, 2 and 3") in the message. The compile refusal tests
  pin it. `spin_state_count` loses its `5 => 5` arm, which can no longer be
  reached.

## Where the refusals sit, and why

- **The spin refusal is in `compile`**, as the first check after the external
  ids resolve. It runs on the process's external legs and on every propagator
  of every diagram. The UFO load is the wrong place: it would refuse a model
  for declaring a graviton. Enumeration is also wrong: it would need a spin
  policy of its own, and it serves `generate` paths that never evaluate. In
  `compile`, the check runs before colour, rooting and lowering, so no later
  stage sees spin 4 or 5. `a_spin_two_model_still_runs_processes_without_it`
  pins that the model still runs a graviton-free process, with |M|² equal to the
  plain toy model's (exact `assert_eq`). The predicate is `evaluable_spin`
  (`|spin| ∈ 1..=3`), so spin 3/2 on an internal line is now refused too.
  Before, it was not checked. External spin 3/2 was already refused.
- **`Particle::helicity_states` still lists five states for spin 5.** Returning
  `None` would leave the polarization parser unchanged (it errors either way).
  But it would make `summed_states` empty for a graviton subprocess, and the
  duplicate-subprocess check would then pass it silently. The doc comment now
  says what the list serves, and that compile refuses the spin.
- **The index refusal is in Lorentz rooting** (`build_at_leg`, through every
  `root_term`), not `ufo/lorentz.rs`'s `to_isize`. Only rooting knows the
  vertex's leg count (`spins.len()`). A load-time check would also refuse the
  whole model. Through `compile`, a spin-2 vertex is refused by the spin check
  first, so the index check catches structures that carry a tensor index on a
  vertex with no spin-2 particle.
  `VertexTerm::from_ufo` wraps every rooting error with
  `Lorentz structure NAME (STRUCTURE): …`. This changes existing messages only
  by that prefix. All callers match with `contains`, and the one `starts_with`
  test (`mixed_sign_vertex_is_refused_at_compile`) reads an error raised outside
  `root_term`.

## The `todo!` sweep

`scalar_root`'s fallback matched every `LorentzOp` but `C`, so `C` is the only
operator that reached it. The census test
`in_tree_models_never_reach_the_charge_conjugation_refusal` parses every UFO
under `validation/ufo/` and `research/refs/mg5amcnlo/models/`. It asserts that
no structure term carries `C`, and it roots every term at the amplitude and at
every non-fermion leg without a panic. Result: no in-tree model carries `C`. A
grep counts 364 `= Lorentz(` definitions across the parsed models (SMEFTsim
260, toys 18, sm 22, MSSM_SLHA2 17, loop_sm 40, hgg_plugin 7). That count is
from grep, not from the test, which only asserts that it saw more than zero.
`taudecay_UFO` is refused by the loader (`Unknown Lorentz operator 'FFCT2'`, a
form-factor function) and is excluded by name. The test fails if any other model
stops parsing. The #71 survey models are not in the tree and were not swept.

## Mutation evidence (each new test on the pre-change code)

The tests were written first and run against base 851ae38's library. Logs are
`G-F1-pre-integration.log` and `G-F1-pre-unit.log` in the scratchpad.

```
$ cargo test --jobs 2 -p vibegraph-lib --test toy_models -- spin_two lorentz_index
test a_spin_two_model_still_runs_processes_without_it ... ok
test a_lorentz_index_naming_no_leg_is_refused_at_compile ... FAILED
test external_spin_two_is_refused_at_compile ... FAILED
test internal_spin_two_is_refused_at_compile ... FAILED
thread 'a_lorentz_index_naming_no_leg_is_refused_at_compile' panicked at vibegraph-lib/src/helas/eval/lower.rs:646:22:
index out of bounds: the len is 2 but the index is 1001
thread 'external_spin_two_is_refused_at_compile' panicked at vibegraph-lib/src/helas/eval/lower.rs:646:22:
index out of bounds: the len is 3 but the index is 2002
thread 'internal_spin_two_is_refused_at_compile' panicked at vibegraph-lib/src/helas/eval/lower.rs:646:22:
index out of bounds: the len is 2 but the index is 1001
test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 13 filtered out

$ cargo test --jobs 2 -p vibegraph-lib --lib -- charge_conjugation an_index_naming in_tree_models_never
test helas::eval::root_lorentz::tests::charge_conjugation_as_a_scalar_factor_is_refused ... FAILED
test helas::eval::root_lorentz::tests::an_index_naming_no_leg_is_refused_at_every_root ... FAILED
test helas::eval::root_lorentz::tests::in_tree_models_never_reach_the_charge_conjugation_refusal ... ok
thread '...charge_conjugation_as_a_scalar_factor_is_refused' panicked at vibegraph-lib/src/helas/eval/root_lorentz.rs:730:17:
not yet implemented: Routing for remaining ops not yet implemented in tree builder: C { i: 1, j: 0 }
thread '...an_index_naming_no_leg_is_refused_at_every_root' panicked at vibegraph-lib/src/helas/eval/root_lorentz.rs:2364:18:
called `Result::unwrap_err()` on an `Ok` value: (LorentzEvalTree { nodes: [Leg(0), Leg(1002), Metric { mu: 0, nu: 1 }, Leg(2), Mul { children: [2, 3] }], root: Some(4) }, 1.0, 1)
test result: FAILED. 1 passed; 2 failed
```

The pre-change messages reproduce the item's reproducer exactly: len 2 /
index 1001 internal, len 3 / index 2002 external. The two tests that pass on
both sides are a positive control and a census. They are not refusal tests.

## What each test cannot detect

- `internal_spin_two_is_refused_at_compile`,
  `external_spin_two_is_refused_at_compile`: they pin that a spin-2 line is
  refused, by name. Neither says anything about a spin-2 amplitude's value.
  The external test does not see a spin-2 leg on a process whose diagrams also
  have a spin-2 propagator: the external check runs first.
- `a_spin_two_model_still_runs_processes_without_it`: blind to a graviton
  diagram that enumeration silently drops from a process that should carry it.
  It only compares a process that has none.
- `a_lorentz_index_naming_no_leg_is_refused_at_compile` and the unit
  `an_index_naming_no_leg_is_refused_at_every_root`: they test indices at or
  above the leg count. They cannot see a wrong in-range index, such as a
  structure that names the wrong leg.
- `charge_conjugation_as_a_scalar_factor_is_refused`: pins the arm on a
  hand-built term. No in-tree model reaches it, so no compile-path test does.
- The census: blind to models outside the tree (the #71 survey set). It does
  not root fermion legs, which need adjoints from a diagram.
- Spin 3/2 on an internal line is newly refused, but no test pins it. A
  gravitino toy would need spin-3/2 Lorentz structures, and that is out of scope.

## Gates

All runs used `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` and `--jobs 2`.

```
$ cargo fmt --all --check                                         → exit 0
$ cargo clippy --jobs 2 --workspace --all-targets -- -D warnings  → exit 0
    Finished `dev` profile [optimized] target(s) in 1m 23s
$ cargo clippy --jobs 2 --workspace --all-targets --features vibegraph/extended-validation -- -D warnings → exit 0
    Finished `dev` profile [optimized] target(s) in 35.84s
$ cargo test --jobs 2 --workspace                                 → exit 0
    50 test binaries / doc-test sets; summed: 1811 passed, 0 failed, 25 ignored
    amplitude_oracle  : test result: ok. 51 passed; 0 failed
    standalone_jamps  : test result: ok. 49 passed; 0 failed
    toy_models        : test result: ok. 17 passed; 0 failed
    validate_helas    : test result: ok. 1 passed; 0 failed (helas_matches_fortran_reference)
$ python3 scripts/kb.py lint                                      → exit 0
    1054 concepts, 340 backlog items; 0 errors, 5 warnings (all pre-existing `state: blocked` warnings)
```

These ran after `cargo fmt` and before the report commit. The report commit
changes only `research/kb/`. The kb lint was re-run after it, with `kb.py index`:
`1055 concepts, 340 backlog items; 0 errors, 5 warnings`.

Of the `extended-validation` skill's rows for `helas/` amplitude evaluation:

- **`validate-helas`** wraps `cargo test -p vibegraph-lib --test validate_helas`.
  It ran inside the workspace test against the committed Fortran reference
  (1 passed). Its `generate-helas` dependency, which rebuilds that reference
  with Fortran, was not run. This container has no pixi environment or Fortran
  toolchain.
- **`validate-amplitudes`** wraps `amplitude_oracle` (51 passed). It ran on the
  banked tables only. Its `generate-amplitude-tables` dependency needs
  MadGraph, so it could not run here. The skill calls the regenerated run the
  stronger check.
- **`standalone_jamps`** (the hermetic sign gate for `helas/eval/root_*`):
  49 passed.

The change touches only refusal arms and the text of rooting errors, so no
banked number should move. No per-row tolerance comparison against a base
build was made: the gate passing at each row's enforced tolerance is the
evidence. It is not bit-for-bit evidence.

## Brief corrections

- The `todo!` the brief and the hygiene item place in `build_child` is in
  `scalar_root` (`root_lorentz.rs` :730 at base), which `build_at_leg` calls for
  each unvisited operator. `build_child`'s own `C` arm already returned
  `UnsupportedVertex`.
- `cargo test -p vibegraph` names the CLI package. The library's tests are
  `-p vibegraph-lib`. The brief's `--features vibegraph/extended-validation` is
  right: the CLI feature forwards to the library.

## Files and commits

- `3f89868` fix(eval): refuse spin 2 and Lorentz indices naming no leg at compile.
  - `vibegraph-lib/src/helas/eval/compile.rs`: `evaluable_spin`,
    `refuse_unsupported_spins`.
  - `vibegraph-lib/src/helas/eval/error.rs`: `UnsupportedSpin { particle, spin, line }`
    and `SpinLine`.
  - `vibegraph-lib/src/helas/eval/root_lorentz.rs`: `leg_indices`,
    `reject_indices_naming_no_leg`, `RootLorentzError::in_structure`, the `C`
    arm and three unit tests.
  - `vibegraph-lib/src/helas/eval/diagram_eval.rs`: wraps errors with the
    structure name.
  - `vibegraph-lib/src/helas/eval/run.rs`, `.../analysis.rs`,
    `vibegraph-lib/src/hadronic.rs`: `unreachable!` arms.
  - `vibegraph-lib/src/ufo/particles.rs`: the `helicity_states` doc.
  - `vibegraph-lib/tests/toy_models.rs`: the spin-2 toy and four tests.
- This report, with the regenerated sessions index.

## Found

- **The duplicate-subprocess check uses `helicity_states().unwrap_or_default()`**
  (`diagrams/mod.rs` `summed_states`). For a spin-3/2 leg the state set is
  empty, so two duplicate gravitino subprocesses are disjoint and are not
  reported as duplicates. Evidence: the code path. Spin 3/2 returns `None`
  there. A refusal, or a spin-3/2 state list for enumeration bookkeeping, would
  close it. It is harmless today, because compile refuses those processes.
- **Internal spin 3/2 has no test.** The refusal is newly generic, but only
  spin 5 is pinned. A spin-3/2 toy vertex test (or the generic predicate tested
  on a spin-4 propagator) would close it.
- **The #71 survey models were not swept** for operators reaching the
  scalar-root `C` arm, because they are not in the tree. Running the census
  test's loop over that set would close it. A Majorana/MSSM-style UFO that
  writes `C` explicitly would now get `UnsupportedVertex` rather than a panic.
