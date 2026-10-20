---
type: Session Report
title: "F1 report: Majorana and fermion-number-violating diagrams are refused at enumeration"
description: "Each kept diagram is checked before it reaches the evaluator: a Majorana fermion on a leg or propagator, or a vertex pairing two particles or two antiparticles, is refused with a ConvertError naming it. Before the fix the FNV toy panicked at the survey's run.rs:1354 site, and both Majorana toys returned a σ without any error. The compile-time bra/ket resolution now asserts its operand pair."
status: draft
tags: [majorana, fermion-flow, refusal, panic, toy-ufo, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: fix, resource: "https://github.com/nsmith-/vibegraph/commit/a04fc80", title: "fix(diagrams): refuse Majorana and fermion-number-violating diagrams"}
---
## What was implemented

- **`Diagram::check_fermion_flow`** (`vibegraph-lib/src/diagrams/diagram.rs`).
  It is called in `diagrams/mod.rs` on every diagram the s-channel filters
  keep, so a `$$` that removes the offending diagrams also removes the
  refusal. It refuses with two new `ConvertError` variants, which reach the
  CLI as `DiagramError::Convert`:
  - `MajoranaFermion`: a self-conjugate spin-½ particle on an external leg or
    an internal propagator, naming the particle and its role;
  - `FermionNumberViolation`: a vertex whose flow-group pairing joins two
    particles or two antiparticles (all-incoming slot particles, by PDG
    sign), naming the vertex and both particles.
- **A debug assertion** in the same function, run in every test build. Every
  enumerated diagram that passes the check must have one particle and one
  antiparticle end (`Leg::charge`, all-incoming) on each fermion line. This
  pins the claim that the check implies the evaluator's one-ket-one-bra
  precondition. The first version of the assertion was wrong, and the suite
  caught it (see Convention surprises).
- **Panic sites.**
  - `kernel.rs` `resolve_bra_ket`: the panic message now states the invariant
    and points to the check.
  - `run.rs` `Bilinear` and `GammaVout`: these are plain arena indexings, with
    no `expect` to reword. Their real precondition lives in the compile-time
    lowering. `layout.rs`'s `bra_ket` closure assumed that if operand *a* was
    not a bra, then *b* was. It now matches `(FermionOut, FermionIn)` /
    `(FermionIn, FermionOut)` and panics on any other pair with an
    internal-invariant message. This is a three-line change in the compiler,
    not a rework. It is the only change in `helas/eval` beyond the message,
    and it is what turns the silent case below into a loud one.
- **Two refusal-fixture UFOs.** `validation/ufo/vibegraph_toy_majorana_UFO`
  has Dirac `lt`, Majorana `nt`, vector `vt` and charged scalar `sc`;
  `vibegraph_toy_fnv_UFO` has Dirac `at` and `bt`, vector `vt` and a
  diquark-like scalar `dq` with `V_atbtdq` = [at, bt, dq~] and its conjugate,
  no `C`. Both are documented in `validation/ufo/README.md`.
- **CLI tests.** `vibegraph-cli/tests/cli_fermion_flow.rs` runs
  `vibegraph integrate --ufo-dir` (the CLI path):
  - `lt~ lt > nt nt` (Majorana external);
  - `lt~ lt > sc sc~` (Majorana t-channel propagator only);
  - `at bt > at bt` (FNV vertex);
  - a control: `lt~ lt > lt~ lt` and `at~ at > at~ at` must integrate, so
    the refusal is per process, not per model.

  Each refused test also asserts no `panicked` in stderr and no artifact.

Refusal text from the CLI:

```
ERROR failed to enumerate process: diagram conversion error: particle 'nt' is a Majorana fermion (its own antiparticle) and is an external leg here; Majorana fermion flow needs charge conjugation, which this evaluator does not implement
ERROR failed to enumerate process: diagram conversion error: particle 'nt' is a Majorana fermion (its own antiparticle) and is an internal propagator here; Majorana fermion flow needs charge conjugation, which this evaluator does not implement
ERROR failed to enumerate process: diagram conversion error: vertex 'V_atbtdq' joins 'at' and 'bt' on one fermion line as two incoming particles; a fermion-number-violating vertex needs charge conjugation, which this evaluator does not implement
```

## The structural condition, per site

The evaluator types each external fermion as a ket or a bra and carries that
adjoint unchanged along the line: currents follow their input's adjoint. A
bilinear therefore closes a line correctly only if the line's two ends have
opposite adjoints. The external typing is in `root_diagram::external_leg`:
`incoming == is_particle` on the all-incoming charge, except that a
final-state leg on a line to the initial state is uncrossed. Under that typing,
the ends are opposite exactly when one end is an all-incoming particle and the
other an antiparticle. Fermion number conserved through every vertex of the
line guarantees this. It breaks in two ways:

1. **A Majorana fermion.** Its pdg is positive whatever its direction, so its
   charge says nothing about the line's direction.
2. **An FNV pairing.** The arrow reverses at the vertex.

The check refuses both. The diagram is the earliest point where this is
decidable *per process*. Refusing at model load would also refuse SM-like
processes of a model that merely contains such vertices.

| Site | What it needs | Broken by | Now |
|---|---|---|---|
| `kernel.rs` `resolve_bra_ket` (interpretive path) | one ket, one bra slot | a line with two same-adjoint ends | unreachable from enumerated diagrams; message states the invariant |
| `run.rs` `Bilinear` arena read (~:1354) | `bra` indexes the fout arena | `layout.rs` `bra_ket` taking a fin operand as the bra | lowering asserts the pair; unreachable |
| `run.rs` `GammaVout` arena read (~:1281) | same, for the vector sink | same | same |

Is one continuity check enough? Yes for every input that comes through
enumeration. The debug assertion holds all 1542 tests' enumerated diagrams to
"each line has one particle end and one antiparticle end". Under the evaluator's
external typing, that is the precondition for all three sites. The sites stay
reachable only by input that does not come from `diagrams::enumerate`:

- hand-built `Diagram`s or ASTs in unit tests;
- a corrupted or foreign artifact, if one bypasses enumeration.

The `unchecked-study` feature's `get_unchecked` arena reads are now protected
by the lowering assertion, not by luck.

**The survey's cards could not be rerun here** (they are on a CERN host). The
claim that this catches *those* cards rests on the toy reproductions. The FNV
toy panics at the identical `run.rs:1354` site before the fix.

## Mutation evidence

**A: the whole fix reverted.** `git stash push -- vibegraph-lib/src`, so the
new tests and toys run against the original library, then
`cargo test -p vibegraph --test cli_fermion_flow`:

```
test a_fermion_number_violating_vertex_is_refused ... FAILED
test a_majorana_fermion_on_a_propagator_only_is_refused ... FAILED
test a_majorana_fermion_on_an_external_leg_is_refused ... FAILED
test dirac_only_processes_of_the_same_models_integrate ... ok
thread 'main' (18525) panicked at vibegraph-lib/src/helas/eval/run.rs:1354:21:
index out of bounds: the len is 4 but the index is 5
thread 'a_majorana_fermion_on_a_propagator_only_is_refused' (18520) panicked at vibegraph-cli/tests/cli_fermion_flow.rs:59:5:
thread 'a_majorana_fermion_on_an_external_leg_is_refused' (18521) panicked at vibegraph-cli/tests/cli_fermion_flow.rs:59:5:
test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.86s
```

- **FNV** fails by panicking at the survey's own site.
- **Both Majorana tests fail differently:** line 59 is the "must be refused"
  assertion. The original binary *integrated them and printed a σ*:

```
== lt~ lt > nt nt
 INFO enumerate: 3 diagrams for lt~ lt > nt nt
σ = 9341.146709 ± 9067.794158 pb
exit 0
== lt~ lt > sc sc~
 INFO enumerate: 1 diagrams for lt~ lt > sc sc~
σ = 0.515771 ± 0.002286 pb
exit 0
```

The external case is two bras on one line. The fin index happened to lie
inside the fout arena, so the read returned garbage and nothing panicked.
That is a silent wrong σ.

**B: only the check call disabled.** The loop body in `diagrams/mod.rs` was
replaced by `let _ = diagram;`, with the `layout.rs` hardening kept:

```
test a_fermion_number_violating_vertex_is_refused ... FAILED
test a_majorana_fermion_on_an_external_leg_is_refused ... FAILED
test a_majorana_fermion_on_a_propagator_only_is_refused ... FAILED
test dirac_only_processes_of_the_same_models_integrate ... ok
thread 'main' (20889) panicked at vibegraph-lib/src/helas/eval/layout.rs:767:22:
internal invariant: a fermion bilinear needs one bra and one ket operand, got (FermionIn, FermionIn); ...
thread 'main' (20885) panicked at vibegraph-lib/src/helas/eval/layout.rs:767:22:
internal invariant: a fermion bilinear needs one bra and one ket operand, got (FermionOut, FermionOut); ...
thread 'a_majorana_fermion_on_a_propagator_only_is_refused' (20882) panicked at vibegraph-cli/tests/cli_fermion_flow.rs:59:5:
test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
```

The hardening turns the silent Majorana-external case into a loud one. The
propagator-only case still evaluates with the check off. Its line has
consistent Dirac-like ends (`lt` in, `lt~` in), so only the check refuses it
(see Found).

## Gate

All commands were run in `/home/user/wt/D` with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

```
$ cargo fmt --all --check; echo "fmt exit $?"
fmt exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 59.82s
clippy exit 0
$ cargo test --workspace      # final tree, log fnvD-F1-test2.log
test exit 0
41 suites: passed 1542 failed 0 ignored 16
tests/cli_fermion_flow.rs test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.55s
tests/amplitude_oracle.rs test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.03s
tests/smeftsim.rs test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.44s
tests/standalone_jamps.rs test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.87s
tests/toy_models.rs test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
tests/validate_madgraph_diagrams.rs test result: ok. 57 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.48s
$ python3 scripts/kb.py index && python3 scripts/kb.py lint | tail -1
824 concepts, 289 backlog items; 0 errors, 4 warnings
```

The four kb-lint warnings are pre-existing `state: blocked` items, unrelated
to this change. The amplitude and diagram oracles above ran on the committed
banked data (`validation/madgraph/amplitudes`, `diagrams.json`). This worktree
has no `validation/madgraph/output` and no `research/refs/mg5amcnlo` content,
so any suite keyed on those did not exercise them here. The MG net
(`pixi run --skip-deps validate`) was not run: pixi is absent, and the change
alters no amplitude for a Dirac process. It only refuses diagrams, and adds
assertions that held across the hermetic suite.

## Files

- `vibegraph-lib/src/diagrams/diagram.rs`: `check_fermion_flow`, the two
  `ConvertError` variants, the debug assertion.
- `vibegraph-lib/src/diagrams/mod.rs`: the call after the s-channel retain.
- `vibegraph-lib/src/helas/eval/layout.rs`: asserting `bra_ket`.
- `vibegraph-lib/src/helas/eval/kernel.rs`: the invariant message.
- `vibegraph-cli/tests/cli_fermion_flow.rs`: new.
- `validation/ufo/vibegraph_toy_majorana_UFO/`, `validation/ufo/vibegraph_toy_fnv_UFO/`: new.
- `validation/ufo/README.md`: documents both.
- `research/kb/backlog/feature/fermion-number-violation-panics-instead-of-refusing.md`:
  deleted.
- `research/kb/validation/cms-gridpack-survey-2026-10.md`: its link to the
  deleted item now points to this sprint's `sprint.md`, marked closed.

Commit on `sprintd-fnv-refusal`: `a04fc80` (code, toys, tests, item deletion),
plus the commit adding this report.

## Convention surprises

`Leg::charge` is the all-incoming charge, as its doc says. `ExtLegInfo::adjoint`'s
`incoming == is_particle` is applied to it *except* for a final-state leg on a
line to the initial state, which `root_diagram::external_leg` first uncrosses
to its physical particle. The first version of the debug assertion applied
`incoming == is_particle` directly to `Leg::charge`. It fired on the Dirac
control `lt~ lt > lt~ lt` (t-channel line from `lt~` in to `lt~` out). The
correct invariant is simply that the two end charges differ. The assertion now
pins that, across the whole suite.

## Found

- **A Majorana propagator with Dirac-consistent ends is now refused, though it
  may have been right.** Before the fix, `lt~ lt > sc sc~` gave
  σ = 0.5158 ± 0.0023 pb with no error. Its only line has consistent ends, so
  the Dirac evaluation may be correct. Nothing validates it, and the refusal
  now blocks it along with every other Majorana diagram. Closing it is part of
  `majorana-fermions-unsupported`: lift the refusal for this case only after a
  per-diagram MadGraph comparison.
- **The silent-wrong-σ class.** Before this change, any fermion-flow defect
  whose fin index fell inside the fout arena produced a σ instead of a panic.
  The Majorana external toy gave 9341 ± 9068 pb. Released builds before
  `a04fc80` may have done this on other Majorana models. The CMS survey
  recorded only panics, so any survey card that "ran" on a Majorana model
  should be rechecked. Evidence: mutation A above. Closed by rerunning the
  survey's BSM cards on this branch.
- **`research/kb/validation/toy-ufo-models.md` does not list the two new
  fixture models.** That bundle file was out of this session's scope. Closed
  by adding them there.
- **Unpaired fermion slots are still unchecked.** A vertex whose Lorentz
  structure leaves a spin-½ slot with no spinor partner (`spin_map[i] == i`)
  passes `check_fermion_flow`, which skips such slots, and its downstream
  behaviour is untested. No model in reach writes one. Closed by a refusal plus
  a toy, if a real model ever needs it.
- **The refusal comes after enumeration, so it costs a full enumeration.** It
  also aborts the whole card on the first refused subprocess, which is
  consistent with how other `DiagramError`s behave.
