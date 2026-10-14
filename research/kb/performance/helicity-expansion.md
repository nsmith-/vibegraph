---
type: Design
title: Helicity expansion and zero-operand pruning
description: "Every helicity combination baked into one hash-consed program so each distinct current is computed once per point; liveness slots, lazy expansion, and probe-verified removal of zero operands."
status: draft
tags: [performance, helicity, evaluator, cse, pruning]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
sources:
  - {id: n15-mg, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/15-eval-optimization-plan.md#L31-L62", title: "Note 15 §1.1 (what MadGraph does before emitting Fortran)"}
  - {id: n15-expansion, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/15-eval-optimization-plan.md#L365-L465", title: "Note 15 §2.2 (helicity-expansion session, CF-factoring analysis)"}
  - {id: n15-filter, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/15-eval-optimization-plan.md#L466-L545", title: "Note 15 §2.3 (helicity filtering)"}
  - {id: n20-zeroamp, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/20-eval-perf-2-plan.md#L186-L218", title: "Note 20 S3 zeroamp-skip"}
  - {id: n31-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L560-L575", title: "Note 31 §E1 (working-set correction)"}
  - {id: compile-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/compile.rs#L60-L370", title: "compile.rs (folded_hel OnceLock, prune_zero_helicities)"}
  - {id: fa-report, resource: "../sprints/hygiene/sessions/F-A-report.md", title: "Hygiene sprint F-A report (the 162 188 live-slot reading)"}
  - {id: fold-prune, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/fold.rs#L289-L400", title: "fold.rs expand_helicities and prune_zero_scalar_operands"}
measured:
  - {commit: c9f826d, host: "Apple M3 Max", command: "cargo bench -p vibegraph-lib --bench eval_strategies"}
---

# Helicity expansion and zero-operand pruning

The helicity-summed |M|² is evaluated by **one** program that contains every
contributing helicity combination, hash-consed across combinations so that each
distinct current is computed exactly once per phase-space point, in one linear
pass with no skip predicate. Which combinations are kept, the probe that decides
it, and the partonic-CM frame contract that follows are owned by
[helicity sum and pruning](../amplitudes/helicity-sum-and-pruning.md); this
concept covers the evaluator side.

## What MadGraph does

MadGraph applies two generations of optimisation before emitting
Fortran:[^n15-mg]
- **wavefunction reuse across diagrams** within one helicity configuration
  ([MadGraph 5: Going Beyond](https://arxiv.org/abs/1106.0522)),
  the analogue of our hash-cons CSE;
- **helicity recycling**
  ([the helicity-recycling paper](../references/papers/helicity-recycling-mg5.md)):
  the helicity loop is unrolled, a dependency DAG built, and calls whose inputs
  coincide between configurations deduplicated, at three levels (external
  spinors, internal currents depending on a subset of legs, partial amplitude
  factors), plus numeric pruning of calls feeding only vanishing amplitudes.
  Net 2.27× on gg→ttgg, ~1.3× on fermion-heavy processes.

## Expansion

`Folded::expand_helicities` takes the list of helicity combinations and builds
one arena under an `Op::Hels` root, with `External` leaves specialised to
`(leg, helicity)` entries and nodes interned across combinations. `PMom` /
`PMomOut` read only routed, helicity-independent momenta, so they are memoised
per pre-expansion node and shared outright.[^n15-expansion][^fold-prune]

Sharing achieved (expanded nodes against combinations × base nodes): 2.8×
(`ee_to_mumu`, 16 combinations), 2.0× (`gg_to_gg`), 2.3× (2→4, 64), 1.8× (2→6,
256: 543k nodes against 990k). `gg_to_gg` shares least: 16 combinations over an
NCOLOR = 6 flow basis.

**Why not recycle per combination.** The first design walked the unexpanded
program once per combination in odometer order and skipped nodes whose
helicity-support mask did not intersect the changed legs. It underdelivered
(1.29× falling to 1.08× with multiplicity) for two structural reasons: the scan
still visited every instruction per combination, and odometer recycling reuses
only the *previous* combination's values, so a node whose support contains the
fastest-varying leg recomputes every time although it takes only
2^|support| distinct values. Expansion removed both: against recycling it measured
2.4–3.0× faster on six of the seven benchmarked processes and 1.68× on `gg_to_gg`,
in a bench that also carried same-day kernel merges. The support-mask machinery
was deleted.[^n15-expansion]

Companion pieces:
- **Liveness slot allocation** in `Program::build` (a slot is recycled once its
  last reader has run; roots pinned live to the end; an instruction never writes
  over its own operands) sizes the arenas by peak live width, not node count. The
  pruned 2→6's arenas are ~288 KiB at `f64` in production order today (see
  [constant collection and fused sums](../performance/constant-collection-and-fused-sums.md)
  and [execution order](../performance/execution-order.md)). The unpruned
  program peaks at 162 188 live slots for 194 371 nodes, measured in the
  hygiene sprint (F-A) and reproduced at `190c13e` by
  `hel_expand_stats::expansion_bounds_arenas`, which
  sums `arena_sizes` over `folded_hel()` of `u u~ > c c~ e+ e- mu+ mu- QCD=0`
  compiled without helicity pruning[^fa-report]; note 31 §E1 recorded 149k.[^n31-e1]
- **Lazy expansion.** `AmplitudeEvaluator::folded_hel` is a `OnceLock`, forced by
  the first helicity-summed read-out (one-time cost ~150 ms on the 2→6, µs–ms
  elsewhere). `eval_amplitude`, Ward checks and probes keep the unexpanded
  program.[^compile-rs]
- **Multi-flow contraction** scales each JAMP by the real CF entry (matching
  MadGraph's real × complex product) and reads JAMPs straight from the arena.

**Exactness.** The expansion copies each node's arithmetic verbatim, so
`eval_m2` is **bit-for-bit** the per-helicity sum through the unexpanded
program, pinned by `expanded_eval_m2_matches_per_helicity_sum` (exact
`assert_eq!` over colourless, massive-external, NCOLOR = 2 and NCOLOR = 6
processes).

## Combination filter

`prune_zero_helicities` probes the full expansion at deterministic generic
partonic-CM points, keeps the combinations above `HEL_PRUNE_REL = 1e-24` of the
sum (MadGraph's madevent criterion, with a threshold far below its
`LIMHEL = 1e-8`, so dropped terms sit below half an ulp and the pruned sum is
bit-for-bit the unpruned one), and **re-expands** over the survivors: the pruned
program is the expansion of the surviving subset.[^n15-filter][^compile-rs]
Survivor counts match MadGraph's generated `NHEL` tables (e.g. 2→6 16/256 for
`uux`, 32/256 for `bbx`). Removing 4–16× more combinations than MadGraph
evaluated was most of the remaining gap to MadGraph's filtered `MATRIX1` at the
time; the 2→6 alone went 10.1× faster.

## Zero-operand pruning inside kept combinations

MadGraph's second filter skips individual diagrams that are zero within a
surviving combination. Here it is `Folded::prune_zero_scalar_operands`, run by
`prune_zero_helicities` after the combination filter:[^n20-zeroamp][^fold-prune]

- It marks scalar nodes that are **exactly `0.0`** at every probe point (the
  same generic partonic-CM points). Detection at the scalar `Add`-operand level
  is sufficient: dropping a zero diagram amplitude dead-code-eliminates its whole
  private vector and spinor subtree with no access to representation internals.
  MHV-type residues are non-zero at this level, so only exact zeros qualify.
- Each modified `Add` (and each `AddScaled`, whose term goes with its weight; see
  [constant collection and fused sums](../performance/constant-collection-and-fused-sums.md))
  is re-folded over its survivors at every probe point and **reverted unless
  byte-identical** to the full sum. That closes the one floating-point hazard, a
  `-0.0` seed whose sign a dropped `+0.0` would flip.
- `AmplitudeEvaluator::zeroamp_node_reduction()` reports before/after node counts.

Effect: `forward` −17% to −32% on the colour-heavy 2→2s that were then the
widest gaps to MadGraph (`ee_to_ee` −32%, `uux_to_uux` −27%, `gg_to_gg` −17%),
neutral elsewhere. Node reductions: `gg_to_ttx` 447 → 277, `ee_to_ee` 165 → 109,
`uux_to_uux` 138 → 98, `gg_to_gg` 775 → 625, the `bbx` 2→6 −3%. Bit-exact
against the unpruned evaluation on all MG-validated processes.

The zero-operand prune inherits the combination filter's frame contract: a
contribution zeroed only in the partonic CM frame is dropped, so a pruned
evaluator takes partonic-CM momenta with beams along ±z.

## Colour-flow factoring: no headroom

Accumulating the Hermitian flow matrix `M_ij = Σ_hel JAMP_i · JAMP_j*` across
combinations and contracting with CF once at the end rebalances rather than
shrinks the arithmetic: the CF multiply leaves the helicity loop, but an
NCOLOR² outer-product accumulation enters it, the same O(N_hel · NCOLOR²) work.
It would also reorder the |M|² sum and drop the bit-for-bit gate to a tolerance
on every multi-flow process. Not built.[^n15-expansion]

The diagonal `M_ii = Σ_hel |JAMP_i|²` is MadGraph's `JAMP2`, the weight its
`SELECT_COLOR` uses to draw the event's leading-colour flow; that read-out is an
O(N_hel · NCOLOR) accumulator beside the existing loop, described in
[colour and helicity selection](../events/colour-and-helicity-selection.md).

## What stays open

The scalar arena holds every configuration amplitude live to the end of the pass
(the `Configs` bundle pins them for AMP2), which bounds how far the arena can
shrink: [scalar-arena-holds-all-amplitudes-live](../backlog/performance/scalar-arena-holds-all-amplitudes-live.md).
Single-helicity evaluation (`eval_amplitude`) runs the unexpanded program and
serves oracles and probes; a MadEvent-style mode that accepts or rejects an event
at one sampled helicity (`nhel = 1`) is in scope but not built
([nhel1-run-cards-refused](../backlog/feature/nhel1-run-cards-refused.md)). How an
accepted event's helicity is chosen from the expanded per-helicity diagonals is in
[colour and helicity selection](../events/colour-and-helicity-selection.md).

[^n15-mg]: Note 15 §1.1.
[^n15-expansion]: Note 15 §2.2, the expansion session and the CF-factoring wrap-up.
[^n15-filter]: Note 15 §2.3, helicity filtering.
[^n20-zeroamp]: Note 20 S3 `zeroamp-skip` outcome.
[^n31-e1]: Note 31 §E1, the corrected 2→6 working set.
[^fa-report]: Hygiene sprint F-A report, Stopped and Found 2: the asserted bound of half the nodes fails because the read-out scalars are pinned live ([expansion-arena-bound-false](../backlog/performance/expansion-arena-bound-false.md)).
[^compile-rs]: `AmplitudeEvaluator::folded_hel`, `prune_zero_helicities`, `HEL_PRUNE_REL`, `zeroamp_node_reduction` in `compile.rs`.
[^fold-prune]: `Folded::expand_helicities` and `prune_zero_scalar_operands` in `fold.rs`.
