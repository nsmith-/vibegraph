---
type: Design
title: The egglog rewrite stage as built
description: "How the lowered AST maps onto an egglog Node datatype, the identity round trip in helas/eval/egraph.rs, the unapplied typed-sorts schema, and which rewrites equality saturation suits."
status: draft
tags: [performance, egglog, e-graph, rewriting, evaluator]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n14-summary, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/14-egglog-notes.md#L21-L38", title: "Note 14 §0 (egglog in one paragraph)"}
  - {id: n14-rewrite, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/14-egglog-notes.md#L152-L192", title: "Note 14 §3.4 (datatype + rewrite is equality saturation)"}
  - {id: n14-relevance, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/14-egglog-notes.md#L280-L312", title: "Note 14 §8 (relevance to vibegraph)"}
  - {id: n15-schema, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/15-eval-optimization-plan.md#L163-L182", title: "Note 15 §1.6 (egglog schema decisions)"}
  - {id: n15-consequences, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/15-eval-optimization-plan.md#L769-L787", title: "Note 15 §5 (consequences for egraph-rewrite)"}
  - {id: n41-egglog, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-completeness-trace-msq-feasibility.md#L251-L286", title: "Note 41 §6 (where egglog fits)"}
  - {id: egraph-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/egraph.rs#L1-L131", title: "helas/eval/egraph.rs (module doc, NODE_SCHEMA, roundtrip)"}
  - {id: lower-fuse, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/lower.rs#L599-L640", title: "lower.rs FuseCtx (chiral-pair FFV fusion at lowering)"}
---

# The egglog rewrite stage as built

`vibegraph-lib` depends on the `egglog` crate (v2.0.0). `helas/eval/egraph.rs`
holds the skeleton of an algebraic rewrite stage: it round-trips the lowered
`Ast<Sym>` through an egglog e-graph and back, unchanged. **No production code
consumes it**: extraction is not wired into the compile pipeline, because a
greedy extractor over the slot-traffic cost model was measured unable to realise
a sharing payoff (see [e-graph DAG extraction](../performance/egraph-dag-extraction.md)).
The enumeration and cost scaffolding are kept for a future global extractor.[^egraph-rs]

## egglog in brief

egglog unifies Datalog and equality saturation. Relations become *functions*
(partial maps with a `:merge` policy), and uninterpreted sorts hold e-class ids
under a union-find. A `datatype` declaration is sugar for a sort plus one
constructor per variant, whose implicit merge is `union`; with database
canonicalisation this is congruence closure, so structurally identical
subexpressions are shared for free.[^n14-summary] `(rewrite p1 p2)` desugars to
`(rule ((= v p1)) ((union v p2)))`: rewriting is non-destructive and matching is
modulo equality, the two properties that define EqSat. Uninterpreted rewrites
and rules that compute with built-in values use the same mechanism.[^n14-rewrite]
`extract` returns the cheapest term in an e-class. The language and the paper
are summarised in [the egglog paper](../references/papers/egglog.md); crate v2.0.0
departs from the 2023 paper's syntax (`constructor`, `:cost`, `:unextractable`,
`ruleset`/`run-schedule`, `subsume`), so check the crate before writing a rule
schedule.[^n14-relevance]

## The schema as built

`NODE_SCHEMA` declares one `Node` datatype with one constructor per fixed-arity
`Op`, named with the same head token the s-expression I/O uses
(`Op::name`):[^egraph-rs]

```lisp
(datatype Node
  (External i64 i64 i64 i64 Node)
  (Propagate Node Node Node)
  (Mul Node Node) (Add Node Node)
  (GammaVout Node Node) … (FfvVout Node Node Node Node) …
  (Coupling i64) (Mass i64) (Width i64) (Coeff f64) (CoeffRat i64 i64 i64))
(sort NodeVec (Vec Node))
(constructor PMomOut (NodeVec) Node)
(constructor Flows (NodeVec) Node)
(constructor Hels (NodeVec) Node)
(constructor Configs (NodeVec) Node)
(constructor AddScaled (NodeVec) Node)
```

- Leaf payloads become leading base-sort fields: `Coupling`/`Mass`/`Width` an
  `i64` id, `Coeff` an `f64`, `CoeffRat` three `i64` (`num den imag`),
  `External` the `leg spin sign incoming` quadruple. Arena children are `Node`
  arguments.
- The variadic ops (a vertex's whole input list for `PMomOut`, the per-flow JAMP
  list, the helicity and configuration bundles, `AddScaled`'s terms) take a
  `(Vec Node)` and are declared as separate constructors after the vector sort.
- `schema_covers_every_op` guards that every `Op` has a constructor.

`roundtrip` declares the schema, encodes the whole AST as one `let` binding with
every child nested inline, then `extract`s it. One `let` inserts the tree in a
single traversal and rebuilds once; egglog rebuilds after each command, so a
node-at-a-time encoding starves that step. Commands are built directly, never
rendered to text. With no rules registered, extraction returns exactly the
inserted term. The gap between the `let` and the `extract` is where a rule
schedule would run.

## The typed schema: decided, not applied

A typed schema was decided for when rewrite rules are written, and the current
`NODE_SCHEMA` does not have it:[^n15-schema][^n15-consequences]

- every leaf kind its own sort (`CouplingId`, `ParticleId`, `Real`,
  `ExtLegInfo`), so a rule bug like `(Mass (CouplingId 5))` fails to typecheck;
- a `ScalarConst` / `ScalarWf` split, so scalar-motion rules are well-typed by
  construction. The constraint is physical: `mul_apply` routes momentum when a
  scalar multiplies a current, so moving a scalar through a propagator is sound
  only for momentum-free scalars (couplings, coefficients), never scalar
  wavefunctions;
- typed constructor slots where the op fixes them (`(Propagate Node Mass Width)`,
  `Mul` split by operand class).

The compile pipeline's `NodeAnalysis` (output type and constness per node)
would be the encoder's source of truth. Apply the schema when rules are next
written; keep `schema_covers_every_op` and the round-trip suite as the guard.

## What is not an egglog job here

Several things the early plan expected the e-graph to do are done by ordinary
passes:

- **Constant folding** strictly removes nodes, is visible to tree-cost
  extraction, and needs no rules; it lives in `fold.rs`, together with the
  constant-collection and fused-sum passes of
  [constant collection and fused sums](../performance/constant-collection-and-fused-sums.md).
- **Chiral-pair FFV fusion** is done at lowering: `lower.rs`'s `FuseCtx` makes
  the walk emit one fused `FfvVout` / `FfvIout` / `FfvOout` node with operands
  `[a, f, g_L, g_R]` at the chiral site instead of the generic `Gamma` and
  projector nodes.[^lower-fuse] `benches/kernel_fusion.rs` measures the fused
  kernels against the generic chain.
- **Hash-cons CSE** is in `lower.rs`.

## Which rewrites equality saturation suits

The rule families still on the table all pay off only through *sharing*
(chiral decomposition, coupling factoring, propagator linearity, re-rooting), and
tree-cost extraction cannot see a sharing payoff. That blocker and its three
prerequisites are recorded in
[e-graph DAG extraction](../performance/egraph-dag-extraction.md).

For a helicity-summed |M|² built from completeness relations and traces
([trace-form feasibility](../performance/trace-form-msq-feasibility.md)), the
pipeline has three stages and the e-graph suits only the middle one:[^n41-egglog]

| stage | suited tool |
|---|---|
| Dirac algebra and trace evaluation (terminating, confluent rules) | a normaliser; equality saturation meets AC blow-up on sums and products. egglog 2.0's `MultiSet`/`BigRat` sorts could store one AC-normal form, but some other code would compute it |
| choosing a representation (momentum elimination, invariant basis, Schouten/Gram relations, common denominators) | e-graph — real choice problems, but every one pays off only through sharing, so it needs the global extractor too |
| evaluating the polynomial | multivariate Horner plus CSE ([FORM code optimisation](https://arxiv.org/abs/1310.7007)); the existing CSE covers half |

Even there, an e-graph fits best as a post-pass on an expression already
reconstructed against an ansatz, not by saturating a huge expanded input.

[^n14-summary]: Note 14 §0.
[^n14-rewrite]: Note 14 §3.4.
[^n14-relevance]: Note 14 §8, including the paper-versus-crate caveat.
[^n15-schema]: Note 15 §1.6, schema decisions.
[^n15-consequences]: Note 15 §5, consequences for the rewrite stage.
[^n41-egglog]: Note 41 §6.
[^egraph-rs]: `vibegraph-lib/src/helas/eval/egraph.rs`: module doc, `NODE_SCHEMA`, `roundtrip`, tests.
[^lower-fuse]: `FuseCtx` and `lower_lorentz` in `vibegraph-lib/src/helas/eval/lower.rs`.
