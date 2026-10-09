---
type: Feasibility Study
title: E-graph DAG-cost extraction
description: "egglog extracts by tree cost; the DAG extractor in egraph.rs; why sharing rewrites are NO-GO without ILP extraction, a work cost model and a ≥3-consumer demo; lowering's ±1-node hash-seed variance."
status: draft
tags: [performance, egglog, e-graph, extraction, cse]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n15-tree-cost, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/15-eval-optimization-plan.md#L63-L79", title: "Note 15 §1.2 (egglog 2.0 extraction is tree-cost)"}
  - {id: n15-sharing, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/15-eval-optimization-plan.md#L112-L133", title: "Note 15 §1.4 (sharing vertices across propagating particles)"}
  - {id: n15-track3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/15-eval-optimization-plan.md#L668-L701", title: "Note 15 §4 (dag-extraction investigation)"}
  - {id: n15-nogo, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/15-eval-optimization-plan.md#L702-L746", title: "Note 15 §4.1 (go/no-go: NO-GO)"}
  - {id: n15-variance, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/15-eval-optimization-plan.md#L747-L768", title: "Note 15 §4.2 (run-to-run AST variance upstream of egraph.rs)"}
  - {id: n15-consequences, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/15-eval-optimization-plan.md#L769-L787", title: "Note 15 §5 (consequences for egraph-rewrite)"}
  - {id: n15-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/15-eval-optimization-plan.md#L788-L799", title: "Note 15 references"}
  - {id: egraph-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/egraph.rs#L180-L830", title: "egraph.rs enumerate / extract / decode_extraction"}
  - {id: extraction-gym, resource: "https://github.com/egraphs-good/extraction-gym", title: "egg community extraction gym (greedy DAG and ILP extractors)"}
---

# E-graph DAG-cost extraction

The rewrites that would make an e-graph worth having for the evaluator all pay
off through **sharing**: two diagrams committing to the same sub-current. egglog
2.0 extracts by tree cost, which cannot see sharing; a greedy DAG extractor was
built and cannot realise it either. Sharing rewrites are **NO-GO** until three
prerequisites exist. The rewrite stage itself is described in
[the egglog rewrite stage](../performance/egglog-rewrite-stage.md); the open item
is [egraph-sharing-rewrites-need-global-extractor](../backlog/performance/egraph-sharing-rewrites-need-global-extractor.md).

## Tree cost is the blocker

In egglog 2.0's `src/extract.rs`, the default `TreeAdditiveCostModel` sums
children per occurrence, and the pluggable `CostModel` trait
(`fold(head, children_cost, head_cost)`) is tree-shaped. There is no
sharing-aware extraction. A shared form has equal or worse tree cost than the
unshared one, so tree-cost extraction never picks it. This blocks, identically:[^n15-tree-cost]

- **re-rooting rules**: all rootings of a diagram have equal tree cost, so
  extraction tie-breaks arbitrarily and will not align rootings across diagrams
  (production rooting is a fixed rule instead; see
  [diagram rooting](../performance/diagram-rooting.md));
- **chiral decomposition and coupling factoring**: the split form costs more as a
  tree and wins only when the pure current is shared at least twice.

### The motivating sharing rewrites

γ and Z exchanged between the same fermion pair could share structure, but the
fused `FfvVout(f1, f2, gL, gR)` carries couplings as operands (γ: gL = gR = e;
Z: gL′ ≠ gR′), so structural CSE can never merge them. Two rewrite families
would expose it:[^n15-sharing]

- **chiral decomposition**, `FfvVout(a,b,gl,gr) → gl·J_L(a,b) + gr·J_R(a,b)`:
  γ and Z then share `J_L`/`J_R` and differ only in scalar recombination. It is
  the inverse of the fusion: fusion wins for a used-once current, splitting wins
  for one shared ≥2×, exactly what a cost-based extractor should arbitrate;
- **propagator linearity**, `Propagate(Mul(s, x), m, w) ↔ Mul(s, Propagate(x, m, w))`,
  to float scalar couplings out so propagated currents unify. Sound only for
  momentum-free scalars: `mul_apply` routes momentum when a scalar multiplies a
  current (ket subtracts, bra adds, the FFS convention pinned by e⁺e⁻→τ⁺τ⁻H).

## What was built

All in `helas/eval/egraph.rs`, unconsumed by production:[^n15-track3][^egraph-rs]

- **`enumerate(&Ast<Sym>) -> DagEGraph`** goes through egglog's supported
  `EGraph::serialize` export (the canonicalised path its GraphViz/JSON tooling
  uses) and translates it into owned `DagEGraph`/`EClass`/`ENode`/`Payload`
  structures: e-classes with their e-nodes, each child edge resolved to its
  e-class, primitive leaf payloads recovered. The function-table route cannot
  recover raw child edges, so `serialize` is the seam.
- **`extract(&DagEGraph, &dyn CostModel, CostKind)`**, a worklist fixpoint in
  the style of the extraction gym's `faster-greedy-dag`: each e-class takes its
  min-cost node whose children are all costed; a candidate's DAG cost is its op
  cost plus the cost of the *union* of its children's chosen descendant sets
  (shared classes counted once); a class's parents are re-examined when its cost
  improves. `CostKind::{Dag, Tree}` selects the accounting. Cost models:
  `SlotTrafficCost` (per-op ≈ output-slot bytes) and `UnitCost`.[^extraction-gym]
- **`decode_extraction`** walks the chosen e-nodes back into an `Ast<Sym>`,
  memoised on e-class id so shared classes become shared arena nodes.
- **Identity gate.** On the rule-free round trip the DAG extractor reproduces the
  input DAG byte-for-byte with the CSE node count intact, over the dev processes
  (`extract_dev_processes_identity`, ungated) and all `MG_VALIDATED_PROCESSES`
  (`extract_validated_processes_identity`, behind `extended-validation`).

## Why sharing is NO-GO

The demo: chiral decomposition on `e+ e- > mu+ mu-`, expecting the DAG extractor
to pick shared `J_L`/`J_R` where tree cost picks the fused current. The rule was
built as throwaway instrumentation, measured and reverted. Two structural
findings:[^n15-nogo]

1. **Greedy cannot realise cross-diagram sharing under any cost model.** It
   decides each e-class independently. Sharing is cheaper only globally, when
   both diagrams co-commit; at any single current class the split form is
   strictly more expensive. Greedy picked the fused form at **0 of 4**
   decomposable classes, DAG cost unchanged. A global (ILP) extractor is
   required for any sharing-payoff rewrite.
2. **`SlotTrafficCost` makes the rewrite a loss even at the global optimum.**
   Forcing the split everywhere gave root cost **2 816 against 2 048–2 144
   fused**: a pure-chiral half-current is charged the same ~96 B as a full one,
   so splitting only adds recombination scaffolding. Under a prototype
   compute-aware `WorkCost`, the optimum is real: **935 split against 1 080
   fused (~13%)**, and greedy still cannot reach it.

`e+ e- > mu+ mu-` is also the marginal case: exactly two consumers (γ, Z) per
spinor pair, so even the `WorkCost` optimum is only ~13%.

**Path to yes, all three required before re-attempting:**
- (a) a global / ILP extractor (the extraction gym has implementations to crib
  from);
- (b) a compute-aware cost model that charges a pure-chiral half-current less
  than a full current, fed by the per-node output-type analysis;
- (c) a demo process with **≥3 consumers** of the same pure current, so the
  payoff is not marginal.

Re-rooting rules additionally need the rooting-soundness property, which now
holds (see [diagram rooting](../performance/diagram-rooting.md)); their bar is
the −19…−26% ns/eval the fixed fewest-legs rule already delivers.[^n15-consequences]

## Lowering's ±1-node variance

`enumerate()` → `extract()` is deterministic relative to its input: on the
rule-free graph every e-class holds one e-node, so the greedy choice is forced
and the identity gate never flakes. What varied between process invocations was
the **lowered AST itself**: `compile_diagram_ast` + `lower` emitted 37 or 38
subterms for `e+ e- > mu+ mu-` depending on the process's hash seed, and the
extractor reproduced it faithfully (37 ↔ DAG cost 2 048, 38 ↔ 2 144 under
`SlotTrafficCost`).[^n15-variance]

- The origin is `HashSet`/`HashMap` iteration in the lowering path
  (`root_diagram`/`lower`) selecting between structurally equivalent emissions,
  which changes what CSE can merge by ±1 node.
- It is correctness-neutral: both ASTs evaluate to the same |M|² and pass the
  amplitude gate. It is a missed-CSE reproducibility wart, not a wrong value, and
  not in the extractor.
- Consequence for any cost comparison used as an extraction oracle: compile the
  AST **once** and reuse it (the extract tests do), or pin the lowering's
  iteration order (a `BTreeSet` or sorted iteration in `root_diagram`/`lower`).
- The lowering still iterates `HashMap`/`HashSet` (`lower.rs`, `root_diagram.rs`);
  whether the ±1-node variance persists on the current tree has not been
  re-measured. Timing rigs that compile per process should assume it does.

## References

[MadGraph 5: Going Beyond](../references/papers/madgraph5-going-beyond.md) is the
diagram-level wavefunction-reuse precedent; helicity recycling is in
[the helicity-recycling paper](../references/papers/helicity-recycling-mg5.md).[^n15-refs]

[^n15-tree-cost]: Note 15 §1.2.
[^n15-sharing]: Note 15 §1.4, including the momentum-routing soundness constraint.
[^n15-track3]: Note 15 §4, milestones M1–M4.
[^n15-nogo]: Note 15 §4.1, the go/no-go record.
[^n15-variance]: Note 15 §4.2.
[^n15-consequences]: Note 15 §5.
[^n15-refs]: Note 15 references.
[^egraph-rs]: `enumerate`, `extract`, `decode_extraction` and their tests in `vibegraph-lib/src/helas/eval/egraph.rs`.
[^extraction-gym]: The egg community extraction gym, cited by note 15 as the source of greedy-DAG and ILP extractor designs.
