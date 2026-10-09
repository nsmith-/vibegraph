---
type: Design Decision
title: Diagram rooting at the vertex with fewest external legs
description: "canonical_root picks the vertex with fewest attached external legs (~20% fewer post-CSE nodes); greedy rooting buys ~1% more and was rejected. Soundness rests on anchor-read convention signs."
status: draft
tags: [performance, rooting, cse, evaluator, diagrams]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n15-rooting, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/15-eval-optimization-plan.md#L80-L111", title: "Note 15 §1.3 (rooting symmetry)"}
  - {id: n15-results, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/15-eval-optimization-plan.md#L611-L667", title: "Note 15 §3.1 (rooting study results)"}
  - {id: rs-defs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/rooting-study-results.md#L9-L51", title: "Rooting study: harness, definitions, variants"}
  - {id: rs-tables, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/rooting-study-results.md#L54-L221", title: "Rooting study: per-process tables"}
  - {id: rs-totals, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/rooting-study-results.md#L222-L266", title: "Rooting study: cross-process totals and findings"}
  - {id: n20-outcome, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/20-eval-perf-2-plan.md#L23-L58", title: "Note 20 sprint outcome"}
  - {id: n20-s4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/20-eval-perf-2-plan.md#L219-L267", title: "Note 20 S4 rooting-cse"}
  - {id: root-diagram, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/root_diagram.rs#L850-L920", title: "canonical_root and choose_root"}
  - {id: rooting-soundness, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/rooting_soundness.rs#L1-L60", title: "rooting_soundness.rs (all-rootings gate, REL_TOL)"}
measured:
  - {commit: 9bb8e14, command: "RUST_MIN_STACK=134217728 cargo test -p vibegraph-lib --profile profiling --features extended-validation rooting_study::rooting_headroom_study -- --ignored --nocapture --test-threads=1"}
  - {commit: 6ab25f1, host: "Apple M3 Max", command: "cargo bench -p vibegraph-lib --bench eval_strategies"}
---

# Diagram rooting at the vertex with fewest external legs

Every Feynman diagram is a tree. Choosing the vertex to root it at orients its
internal edges, and so decides which off-shell currents the lowering emits. The
amplitude does not depend on the choice; how much cross-diagram CSE finds does.

**Decision.** Production roots each diagram at `canonical_root`: the vertex with
the **fewest directly-attached external legs**, ties broken toward the lowest
vertex index.[^root-diagram] A greedy per-diagram search buys only ~1% more and
is not built.

```rust
fn canonical_root(diagram: &Diagram) -> VtxIdx {
    let mut best_vi = 0usize;
    let mut best_c = ext_leg_count(&diagram.vertices[0]);
    for (vi, v) in diagram.vertices.iter().enumerate().skip(1) {
        let c = ext_leg_count(v);
        if c < best_c { best_c = c; best_vi = vi; }
    }
    VtxIdx(best_vi)
}
```

Rooting at a low-external vertex keeps sub-currents closer to the
`(edge, direction)` signatures CSE deduplicates across diagrams; rooting at a
high-external hub duplicates them. In test builds `choose_root` consults a
per-thread override (`set_root_override`) so a harness can re-root diagrams; in
release builds it is `canonical_root` alone.

## Why it is sound

The rooting-dependent convention signs are read off one fixed rooting, at the
diagram's anchor, and lifted into the diagram's `fermi_sign`; the honest
currents are then root-invariant. The convention itself is described in
[rooting invariance and the anchor](../amplitudes/rooting-invariance-and-anchor.md).
The `rooting_soundness` module pins it: `all_rootings_preserve_amplitude`
re-roots every diagram at every vertex across the MG-validated processes and
requires |M|² to match the baseline (unoverridden) evaluation within
`REL_TOL = 1e-10`.[^rooting-soundness]

- The oracle is the baseline itself, not MadGraph: production is already pinned
  against MadGraph, so any rooting that reproduces it is correct.
- `1e-10` is looser than the amplitude gate because a correct re-rooting
  reassociates the momentum sums that route each propagator; the observed worst
  case is 2.2e-11 (`e+e-→τ+τ-H`, an 8-momentum sum), and any sign or structure
  error is O(1).
- The test is `#[ignore]` (a slow full recompile sweep). Run it after touching
  the rooting, Lorentz-output or fermion-sign machinery:

```
RUST_MIN_STACK=134217728 cargo test -p vibegraph-lib \
    --lib helas::eval::rooting_soundness::all_rootings_preserve_amplitude \
    -- --ignored --nocapture --test-threads=1
```

The MadGraph amplitude gate (`tests/amplitude_oracle.rs`) stayed at its
**1e-12** tolerance under canonical rooting; only the all-rootings check uses
1e-10. Agreement was unchanged or better after the switch (e.g. `ee_to_mumua`
3.92e-13 → 1.62e-14): shorter, more-shared current chains reduce floating-point
drift.[^n20-s4][^n20-outcome]

## The headroom study

A per-diagram root override, six variants, 14 processes; nodes are the reachable
nodes of the colour-aware post-CSE arena (pre-helicity-expansion), "weighted" is
Σ output-slot bytes (scalars 16 B, currents 96 B).[^rs-defs] Cross-process
totals:[^rs-totals]

| variant | Σ nodes | Σ weighted (B) | vs `VtxIdx(0)` |
|---|--:|--:|--:|
| `VtxIdx(0)` (feyngraph's first vertex) | 9 111 | 534 976 | — |
| lowest-leg anchor | 9 111 | 534 976 | 0% |
| most external legs | 10 564 | 673 824 | +15.9% |
| **fewest external legs** | **7 287** | **361 072** | **−20.0%** |
| greedy (as-generated or largest-first) | 7 200 | 354 080 | −21.0% |

- Greedy (lower diagrams one at a time, each choosing the root adding the fewest
  new nodes to the cumulative arena) cuts nodes 21% and slot-weighted traffic
  34%. The one-line fewest-legs rule captures nearly all of it; diagram order
  makes no difference.
- "Most external legs" moves the wrong way: rooting at a central, high-degree
  vertex duplicates currents.
- Feyngraph's `VtxIdx(0)` is already the vertex adjacent to the lowest-index
  external leg, so "lowest-leg anchor" reproduces it on every process.
- 2→2 processes do not move: their vertices tie on external-leg count, so
  `canonical_root == VtxIdx(0)` there.
- The extremes are the 8-point QCD=0 processes: `u u~ > c c~ e+ e- mu+ mu-`
  goes 3 876 → 3 040 nodes and realises 642 → 251 distinct `Propagate` currents
  (against 2 895 internal edges with no sharing); `b b~ > c c~ e+ e- mu+ mu-`
  4 248 → 3 304. `e+ e- > mu+ mu- a` is the only 2→3 that moves (95 → 87).[^rs-tables]
- The deduplicated `(edge, direction)` "floor" (2 090 on the `uux` 2→6) counts
  both directions of every edge, about 3× what any single rooting realises; it is
  not a reachable target. The informative comparison is realised `Propagate`
  count against the no-share bound `sum_edges`.

In the original study every node-reducing rooting failed the MadGraph gate,
silently and grossly (max_rel up to 1.7e3, 50/50 points) on `e+e-→W+W-`,
`e+e-→τ+τ-H` and every ≥6-point QCD=0 process. That was the
orientation-dependence the anchor-read signs removed; it is why the gate above
exists, and the study's table records which rootings had exercised
it.[^rs-totals][^n15-results]

## What it bought

Measured on the helicity-expanded, pruned program through
`eval_strategies`, M3 Max: `forward` **−19% (2→3), −18% (2→4), −26% (2→6)**,
neutral on 2→2. The bar any future e-graph re-rooting or DAG-cost extractor must
beat is this −19…−26% ns/eval, not the node counts.[^n20-s4] Why sharing
rewrites in an e-graph cannot yet reach it is in
[e-graph DAG extraction](../performance/egraph-dag-extraction.md); the evaluator
program the rooted diagrams lower into is in
[the helicity program layout](../performance/evaluator-program-layout.md).

## Caveats

- Node and traffic counts were taken on the pre-expansion per-diagram arena; the
  delivered gain is the ns/eval figure above.
- Re-rooting is never bit-for-bit against another rooting. Any change to the
  root rule moves amplitude values at the ~1e-14 level and must be gated at
  tolerance, not byte equality.
- New rootings exercise kernel paths the old rooting never hit; the
  all-rootings sweep is what covers them.[^n15-rooting]

[^n15-rooting]: Note 15 §1.3: rooting symmetry, canonical and greedy alternatives, the validation caveat.
[^n15-results]: Note 15 §3.1: study results summary.
[^rs-defs]: Rooting study: harness, metric definitions and variants.
[^rs-tables]: Rooting study: per-process tables (full tables stay in the archived results note).
[^rs-totals]: Rooting study: cross-process totals and findings.
[^n20-outcome]: Note 20 sprint outcome.
[^n20-s4]: Note 20 S4 `rooting-cse` outcome.
[^root-diagram]: `canonical_root`, `choose_root`, `set_root_override` in `root_diagram.rs`.
[^rooting-soundness]: `rooting_soundness.rs` module doc, `REL_TOL`, `all_rootings_preserve_amplitude`.
