---
type: Session Report
title: "K-O1 report: negative UFO coupling orders and the gluon-free WEIGHTED trial"
description: "Coupling orders load signed and are spent as MadGraph spends them, vertex by vertex in its reduction order; the automatic WEIGHTED search trials each subprocess with its final-state gluons removed. SMEFT_alpha loads and matches MadGraph per diagram and per helicity."
status: draft
tags: [ufo, coupling-orders, weighted, smeft, smeftsim, madgraph-parity, report]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-17}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Worktree `/home/user/wt/sprintk-negative-orders`, branch `sprintk-negative-orders`,
based on `main` at `703f57f`. MadGraph is the pinned submodule
(`research/refs/mg5amcnlo`, `VERSION` 3.7.1), run through
`validation/madgraph/mg5_pinned.sh` from the main checkout's `madgraph` pixi
environment. Scratch work sits in `/home/user/wt/sprintk-negative-orders-mg/`.
The brief named MadGraph 3.5.7; the pinned checkout every generator here runs is
3.7.1.

| commit | what |
|---|---|
| `ee1eb01` | signed UFO coupling orders and the gluon-free WEIGHTED trial (checkpoint) |
| `de997b7` | spend coupling-order bounds in MadGraph's reduction order; toy model counts |

## How MadGraph treats a negative order

1. **Loading.** `models/import_ufo.py:1854` copies `coupling.order` into the
   interaction as written; `Interaction.filter` (`base_objects.py:725-734`) asks
   only that each power is an `int`. No clamp.
2. **A diagram's orders.** `Diagram.calculate_orders` (`base_objects.py:2638-2657`)
   sums each order's signed powers over the vertices, and WEIGHTED as Σ
   hierarchy × power; both can fall below a positive-only sum.
3. **Interaction WEIGHTED and the search's model inputs.**
   `Interaction.get_WEIGHTED_order` (`:893-899`) is the same signed sum over
   `max(n − 2, 1)`; `get_max_WEIGHTED` (`:1481`) takes the largest and
   `find_optimal_process_orders` truncates it with `int()`
   (`diagram_generation.py:1999`). `get_particles_hierarchy` (`:1432-1479`)
   reads only the order *names*, so SMEFT_alpha's `{NP: 1, QED: -1}` triple-Higgs
   vertex joins the QED class.
4. **The bounds: a running budget, not a total.** `reduce_orders`
   (`diagram_generation.py:1048-1083`) subtracts each new vertex's signed orders,
   and its WEIGHTED, from what the process allows as `reduce_leglist`
   (`:958-1046`) builds the diagram, and drops it the moment a remaining budget is
   below zero; a negative bound is ignored. Nothing is clamped, but the check is
   per prefix: a diagram whose total is within the bound is dropped when its
   negative vertex is added after a prefix that is not. With non-negative orders
   prefix and total agree, which is why no banked row can see it.
5. **The reduction order is a property of the tree.** A combination needs at
   least one leg made in the step before (`LegList.can_combine_to_1`,
   `base_objects.py:2250-2256`; uncombined legs lose the mark, `merge_comb_legs`,
   `diagram_generation.py:1224-1231`), so each vertex is made at its height above
   the external legs. The closing vertex needs two fresh legs (`can_combine_to_0`,
   `base_objects.py:2258-2279`) or is the identity joining the last two
   (`:579-589`, merged into `ref_dict_to0` at `:1176-1179`): the root is the tree's
   centre. A decay closes on its initial leg (`:2267-2274`). Within a step the
   vertices are checked in leg order, which stays sorted by the smallest external
   leg number below each (`combine_legs`, `diagram_generation.py:1114-1153`).

## What changed

- `ufo/couplings.rs`: `Coupling.orders` is `BTreeMap<String, i64>`; a negative
  power loads. bincode writes a non-negative `i64` with the same bytes as a
  `usize`, so model digests and every artifact written before are unchanged.
- `ufo/topo.rs`: `NegativeOrders`. feyngraph counts powers as `usize`; each
  negative power goes to it as 0 and is recorded per feyngraph vertex, and a
  diagram's signed power is feyngraph's count plus the recorded part.
- `ufo/weighted.rs`: interaction WEIGHTED, the cap and `maximum_weighted` are
  signed, truncated towards zero as Python's `int` truncates.
- `diagrams/selector.rs`: an order some interaction carries negatively is bounded
  on the signed sum through a custom selector function; every other order keeps
  feyngraph's own (pruning) selection, so Standard-Model paths are untouched.
- `diagrams/reduction.rs` (new): MadGraph's running budget in its reduction order
  (height, then smallest leg number, the centre vertex last; a decay's initial-leg
  vertex last). `generate_sets_inner` applies it to every `<=`/`=` bound and the
  WEIGHTED bound when the model carries any negative power.
- `diagrams/mod.rs`: the WEIGHTED filter is the signed sum; the automatic search
  runs MadGraph's gluon-free trials (below); a new refusal
  `GluonTrialWithoutQcd` names the case MadGraph answers with `KeyError`.
- Signed orders through `Diagram::coupling_orders`, the decay-chain overall-order
  check, `qcd_order`, and the CLI's `enumerate`/`inspect` JSON.
- `validation/ufo/vibegraph_toy_negorder_UFO` (authored, README entry added): a
  scalar cubic coupling at `QED=1` and again at `{NP: 1, QED: -1}`.
- `tests/negative_orders.rs`: the signed load, and 22 diagram counts against
  MadGraph. `tests/weighted_search.rs`: the gluon test runs.

## The gluon-free trial

`find_optimal_process_orders` (`diagram_generation.py:2015-2108`), reproduced as
`GluonFreeTrials` when PDG 21 is in the lowest particle class and a final-state
leg can be a gluon: per initial-state assignment, each final-state assignment
(one per sorted content) is trialled with its gluons removed, at the bound
lowered by the QCD hierarchy per gluon (a negative trial bound bounds nothing);
legs from particle ids alone; a crossing that failed is skipped unless
s-channels are forbidden; a two-leg trial passes on a particle-antiparticle pair
unless an s-channel is required. When the gluon condition does not hold the
search trials the whole process, as before: MadGraph's trials then differ only in
ignoring polarizations and skipping failed crossings, which can move only the
bound a refusal quotes.

## Oracles

ORACLES-PLACEHOLDER

## Found

FOUND-PLACEHOLDER
