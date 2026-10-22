---
type: Backlog Item
title: The colour sum is a dense CF contraction in MadGraph's flow basis
description: "eval_m2 contracts every helicity's JAMPs with the full n×n CF matrix in MadGraph's basis; pyAmpliCol reports 6.5× on gg → 5g from a smaller adjoint (DDM) basis plus a symmetric-group FFT."
area: performance
state: open
priority: low
closes_when: "The colour-sum share of eval_m2 is measured on the banked all-gluon and multi-gluon rows. Either it is too small to matter, recorded with the measurement, or a reduced basis (DDM where the model's colour tensors allow it) and/or an FFT contraction lands with |M|² agreeing with the dense CF path to rounding on every banked row, and per-flow complex values still checkable against MadGraph."
blocked_by: []
opened: 2026-10-21
tags: [colour, performance, amplitudes, ddm, fft, multi-gluon]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-21}
sources:
  - {id: pyamplicol, resource: "https://arxiv.org/abs/2610.10729", title: "pyAmpliCol: fast tree-level matrix elements (§3.5, §5.3, Table 6)"}
  - {id: run, resource: "vibegraph-lib/src/helas/eval/run.rs", title: "eval_m2: Σ_i (Σ_j CF_ji J_j) J_i* per helicity combination"}
  - {id: flow, resource: "../../amplitudes/colour-flow-evaluator.md", title: "Colour-flow evaluator"}
---
**Today.** `eval_m2` (`vibegraph-lib/src/helas/eval/run.rs`) sums
`Σ_i (Σ_j CF_ji J_j) J_i*` for every surviving helicity combination. That is
an O(n_flows²) dense contraction in MadGraph's flow basis, with the exact
rational CF matrix ([colour-flow evaluator](../../amplitudes/colour-flow-evaluator.md)).
It reproduces MATRIX1's ZTEMP order bit for bit, which the per-flow MadGraph
oracle relies on.

**What the paper reports.** pyAmpliCol (arXiv:2610.10729) contracts colour-ordered
partial amplitudes with the colour-overlap matrix through a non-Abelian FFT on
the symmetric group (§3.5). It uses the Del Duca–Dixon–Maltoni (DDM) adjoint
basis where the model's colour tensors allow it, which needs (n−2)! orderings
instead of (n−1)!. On gg → 5g, full colour with helicities summed, the adjoint
basis takes 1.43 ms against 9.30 ms in the trace basis, both already using
the FFT: the gain comes from the basis (§5.3). gg → H + 4g in HEFT gains 4.3×.

**Open questions.**
- How large is the colour sum's share of `eval_m2` on our rows? `gg_to_gg`
  is among the slowest rows against MATRIX1
  ([MATRIX1 comparison](../../performance/matrix-element-vs-madgraph.md)),
  but nobody has profiled whether the contraction or the currents dominate.
- MadGraph's flow basis is not the trace or DDM basis. A change of basis
  must keep a per-flow complex oracle against MadGraph (AGENTS.md "Physics
  Validation"), for example by keeping the MadGraph-basis JAMPs available
  under test.
- The win grows with gluon count. At the gluon multiplicities banked
  today the dense sum runs over few flows and may be negligible, so
  measure first.
