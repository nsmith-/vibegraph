---
type: Backlog Item
title: Lorentz tree building and spin-code matches panic instead of refusing
description: "build_child ends in a todo! for operators it does not route, and three spin-code matches panic on an unexpected code, so a model reaching them crashes instead of getting a named error."
area: hygiene
state: open
priority: medium
closes_when: "Each of the four sites either returns a typed error (RootLorentzError / EvalError / HadronicError) naming the operator or spin code, with a test that reaches it through the public compile path, or is shown unreachable by an earlier refusal that a test pins; no todo!, and no panic! on model-supplied input, remains on these paths."
blocked_by: []
opened: 2026-10-26
tags: [refusal, panic, non-sm-ufo, helas, lorentz]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: root-lorentz, resource: "../../../../vibegraph-lib/src/helas/eval/root_lorentz.rs", title: "helas/eval/root_lorentz.rs LorentzEvalTree::build_child, the todo! fallback arm (~:730)"}
  - {id: run, resource: "../../../../vibegraph-lib/src/helas/eval/run.rs", title: "helas/eval/run.rs build_external_core, 'unsupported external spin code' (~:1948)"}
  - {id: analysis, resource: "../../../../vibegraph-lib/src/helas/eval/analysis.rs", title: "helas/eval/analysis.rs external node type, 'unsupported external spin code' (~:251)"}
  - {id: hadronic, resource: "../../../../vibegraph-lib/src/hadronic.rs", title: "hadronic.rs spin_state_count, 'unsupported spin code' (~:625)"}
  - {id: pr71, resource: "https://github.com/nsmith-/vibegraph/pull/71", title: "PR #71: CMS-gridpack survey (a panic outcome is counted per run)"}
---
Four places panic on input that comes from a UFO model, where the release scope
asks for a hard error. The #71 survey records "panic" as a run outcome of its own.
These were found while reproducing the spin-2 panic:

1. **`LorentzEvalTree::build_child`** (`helas/eval/root_lorentz.rs` ~:730)
   routes Gamma, Sigma, the projectors, Identity, Metric, P and Epsilon. Any
   other operator it meets on a routing path ends in
   `todo!("Routing for remaining ops not yet implemented in tree builder: ...")`.
   The function already returns `Result<_, RootLorentzError>`, so this arm can
   return an error naming the operator and the structure. Which UFO
   structures reach the arm is unmeasured; a sweep of the in-tree and #71
   survey models would say.
2. **`build_external_core`** (`helas/eval/run.rs` ~:1948): `other =>
   panic!("unsupported external spin code: {other}")`.
3. **The external node type** in `helas/eval/analysis.rs` (~:251): the same
   panic message for the same match.
4. **`spin_state_count`** (`hadronic.rs` ~:625): `other => panic!("unsupported
   spin code {other} for the spin average")`.

Sites 2–4 match on a spin code that `AmplitudeEvaluator::compile` is supposed
to have vetted already. Today compile lets spin code 5 through, so these arms
are reachable once the `lower_lorentz` index panic is gone. That case is
[spin-2-panics-instead-of-refusing](../feature/spin-2-panics-instead-of-refusing.md).
Either one refusal upstream makes them unreachable (pin that with a test and
keep `unreachable!` with the invariant stated), or each returns an error.

`EvalError` already has an "unsupported external spin code" variant
(`helas/eval/error.rs` ~:50), so sites 2 and 3 have an error to return.
