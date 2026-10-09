---
type: Design
title: "Evaluator architecture: compile, bind, evaluate"
description: "A process compiles once into a card-independent program, binds a param card at a chosen float type, then evaluates |M|² per point; each tree diagram is rooted into off-shell currents."
status: draft
tags: [evaluator, architecture, rooting, compile, amplitudes]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n10-goal, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/10-lorentz-runtime-eval-plan.md#L27-L46", title: "Note 10 §1: compile phase and evaluation phase"}
  - {id: n10-topo, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/10-lorentz-runtime-eval-plan.md#L355-L382", title: "Note 10 §5.4: rooting a tree diagram into currents"}
  - {id: code-compile, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/compile.rs#L1-L130", title: "compile.rs: AmplitudeEvaluator and its passes"}
  - {id: code-root, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/root_diagram.rs#L855-L990", title: "root_diagram.rs: canonical_root, root_tree, root_tree_at"}
  - {id: code-bind, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/run.rs#L230-L400", title: "run.rs: BoundAmplitude::bind and eval_m2"}
---

# Evaluator architecture: compile, bind, evaluate

A general tree-level amplitude evaluator, with no code generation: every HELAS-type
primitive is a statically compiled kernel dispatched through a flat `Op` enum
([flat-op-ir](flat-op-ir.md)).[^n10-goal] Work is split by how often it runs.

| stage | entry point | runs | depends on |
|---|---|---|---|
| compile | `AmplitudeEvaluator::compile(&DiagramSet, &UFOModel)` | once per subprocess | model and diagrams; not the param card, not the float type |
| bind | `BoundAmplitude::bind(&AmplitudeEvaluator, &EvaluatedModel)` | once per param card (and per float type `F`) | card values |
| evaluate | `BoundAmplitude::eval_m2(momenta, scratch)` | once per phase-space point | momenta |

## Compile

`helas/eval/compile.rs` orchestrates the passes:[^code-compile]

1. **Root each diagram (pass 1+2).** `compile_single_diagram` walks the diagram
   from a chosen root vertex into a `DiagramEval`: external legs become leaves,
   each internal vertex emits an off-shell current toward the root followed by a
   `Propagate` through its propagator, and the root vertex emits the amplitude
   contraction. Each vertex's UFO Lorentz structure is rooted at the leg the walk
   leaves through (`root_lorentz.rs::build_at_leg`). The per-diagram convention
   sign `fermi_sign` is attached here ([convention-sign-inventory](convention-sign-inventory.md)).
2. **Colourize.** Each diagram's colour strings are simplified into the colour
   basis and exact CF matrix ([colour-flow-evaluator](colour-flow-evaluator.md)).
3. **Lower (3a).** `lower::lower_flows` inlines every diagram into one whole-amplitude
   `Ast<Sym>`: per flow, `JAMP_f = Σ (colour coeff) · (symmetry · fermi_sign) ·
   amp_{d,chain}`, under a single `Flows` root (bundled with per-configuration
   amplitudes under `Configs`). Hash-consing shares every sub-current used by more
   than one diagram, flow or colour chain.
4. **Fold (3b).** `Folded::build` interns constants into a card-independent
   skeleton with constant-pool specs.

The helicity-expanded program (every helicity combination baked into one arena
under an `Op::Hels` root) is built lazily on first use of `eval_m2`; see
[helicity-sum-and-pruning](helicity-sum-and-pruning.md) and
[performance/helicity-expansion](../performance/helicity-expansion.md).

## Rooting a diagram

A tree diagram becomes a tree of currents once a root vertex is chosen. For the
s-channel diagram of `e+ e- → mu+ mu-` rooted at the muon vertex:[^n10-topo]

```
External(e-), External(e+) → OffShellCurrent(FFV, [e-, e+])   // γ/Z current
                           → Propagate(γ/Z)
External(mu-), External(mu+), propagated → ContractAmplitude(FFV)
```

Production roots each diagram at `canonical_root`: the vertex with the fewest
directly attached external legs, ties to the lowest vertex index.[^code-root] A
deep, few-external root keeps more sub-currents in the canonical
`(edge, direction)` form that cross-diagram sharing deduplicates; rooting at a
high-external hub duplicates them. The physics does not depend on the choice: the
currents are root-invariant, and every root-dependent convention sign is read
from a second tree rooted at the diagram's *anchor* (`Diagram::anchor`, built only
when the two roots differ). See
[rooting-invariance-and-anchor](rooting-invariance-and-anchor.md) and
[performance/diagram-rooting](../performance/diagram-rooting.md).

## Bind and evaluate

`BoundAmplitude::bind` resolves the folded constant pools against an
`EvaluatedModel` at a scalar type `F`, and turns the exact rational CF matrix into
`Box<[F]>`. Floats enter here, nowhere earlier.[^code-bind] One compiled evaluator
serves any card and any precision (including SIMD lanes).

`eval_m2` returns `Σ_hel Σ_ij CF_ij J_i J_j*`: summed, not averaged, over
helicities and colours, MadGraph's `MATRIX1` convention. Spin and colour averages
and identical-particle factors belong to the cross-section layer. Diagrams are
summed coherently inside each flow before squaring. Other read-outs off the same
program: `eval_jamp2` (per-flow `Σ_hel |J_f|²`) and `eval_amp2` (per-configuration
`Σ_hel |A_c|²`), which drive per-event colour selection
([per-diagram-amp2](per-diagram-amp2.md)).

`prune_zero_helicities` needs an `EvaluatedModel` because which combinations
vanish can depend on the card (massive vs massless fermions). Execution order and
layout of the helicity program are performance concerns:
[performance/execution-order](../performance/execution-order.md).

[^n10-goal]: Note 10 §1.
[^n10-topo]: Note 10 §5.4; it describes a bottom-up sweep that discovers the root, where the code chooses the root explicitly as described here.
[^code-compile]: `vibegraph-lib/src/helas/eval/compile.rs`, module doc and `AmplitudeEvaluator`.
[^code-root]: `vibegraph-lib/src/helas/eval/root_diagram.rs`, `canonical_root`.
[^code-bind]: `vibegraph-lib/src/helas/eval/run.rs`, `BoundAmplitude::bind`, `eval_m2`.
