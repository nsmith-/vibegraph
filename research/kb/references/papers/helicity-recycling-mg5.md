---
type: Paper
title: Speeding up MadGraph5_aMC@NLO (helicity recycling)
description: "arXiv:2102.00773 (Ostrolenk, Mattelaer; EPJC 81:435): helicity filtering and CSE across the unrolled helicity loop, about 2x on gg -> ttgg; what vibegraph's helicity expansion mirrors."
resource: "https://arxiv.org/abs/2102.00773"
status: draft
tags: [madgraph, helicity, cse, performance, paper]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n15-mg, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/15-eval-optimization-plan.md#L31-L62", title: "Note 15 §1.1, what MadGraph does before emitting Fortran"}
  - {id: n15-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/15-eval-optimization-plan.md#L788-L799", title: "Note 15, references"}
  - {id: n41-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-completeness-trace-msq-feasibility.md#L788-L810", title: "Note 41, references"}
  - {id: mg-runcard, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Cards/run_card.dat#L80", title: "MadGraph LO run_card.dat template, sde_strategy line"}
  - {id: mg5-beyond, resource: "https://arxiv.org/abs/1106.0522", title: "Alwall et al., MadGraph 5: Going Beyond (wavefunction reuse within one helicity)"}
---

"Speeding up MadGraph5_aMC@NLO" (Ostrolenk and Mattelaer, EPJC 81 (2021) 435)
describes helicity recycling, MadGraph's second generation of pre-emission
optimisation. MadGraph's LO `run_card.dat` template cites it for `sde_strategy` as
`hep-ph/2021.00773`, a typo for this paper[^mg-runcard].

## The two generations

1. **Wavefunction reuse within one helicity configuration** (MadGraph 5,
   arXiv:1106.0522[^mg5-beyond]): identical wavefunction calls across diagrams
   are emitted once.
2. **Helicity recycling** (this paper): common-subexpression elimination
   across the unrolled helicity loop[^n15-mg].
   - Generate the matrix element normally, then evaluate a few events to find
     the contributing helicity configurations; the rest are filtered out
     permanently.
   - Unroll the helicity loop, build the DAG of wavefunction and amplitude
     calls, and deduplicate calls whose inputs coincide between
     configurations, at three levels: external spinors (spinor `1⁺` is the same
     in `(1⁻2⁺3⁻4⁺)` and `(1⁻2⁺3⁺4⁻)`), internal currents (a current that
     depends only on legs 3 and 4 is shared by every configuration of legs 1
     and 2), and partial amplitude factors (for `ψ̄₁γ_μψ₂ ε^μ`, store
     `ψ̄₁γ_μψ₂` and contract it with each polarization; which factorisation to
     recycle is chosen by reuse count).
   - Prune calls that feed only vanishing amplitudes; about 20% more from CSE
     on colour factors. ALOHA gained new output forms to support it.

Net gain: 2.27× for `g g → t t̄ g g`, about 1.3× for fermion-heavy processes.

## Relevance to vibegraph

vibegraph does the same thing structurally. Each node's helicity dependence is
static (the external legs in its subtree), so the helicity program evaluates a
node once per distinct helicity assignment of its own legs rather than once per
full configuration, and prunes zero operands
([helicity expansion](../../performance/helicity-expansion.md),
[helicity sum and pruning](../../amplitudes/helicity-sum-and-pruning.md)). The
gain applies to the helicity-summed path, and that is the only path vibegraph
has: integration evaluates `eval_m2`, and an accepted event draws its helicity
from the per-helicity diagonal `eval_hel_m2` of one more summed evaluation
(`helas/eval/run.rs`, `hadronic.rs`), as MadEvent's `SELECT_HEL` does. No hot
loop evaluates a single fixed helicity until helicity Monte Carlo (`nhel = 1`,
not built) exists, so a single-helicity timing against MadGraph would answer
no current question
([mg-single-helicity-bench-no-consumer](../../backlog/performance/mg-single-helicity-bench-no-consumer.md)).
Sharing across diagrams in the evaluator is extraction-limited in the e-graph
route ([DAG extraction](../../performance/egraph-dag-extraction.md)), and the
trace-form alternative to the helicity sum is
[the trace-form feasibility study](../../performance/trace-form-msq-feasibility.md).

[^n15-mg]: Note 15 §1.1; authors confirmed on arXiv (the notes cite "Frederix et al.").
[^mg-runcard]: `Template/LO/Cards/run_card.dat:80` at `b7687064`: `sde_strategy  ! default integration strategy (hep-ph/2021.00773)`.
[^mg5-beyond]: arXiv:1106.0522, cited in note 15's reference list.
