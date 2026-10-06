---
type: Backlog Item
title: Polynomial reweighting costs K amplitude evaluations per event
description: Class amplitudes are read off K node evaluations per event; a graded evaluator would compute coupling-free subtrees once.
area: performance
state: open
priority: low
closes_when: The polynomial reweighting path evaluates each event once with a graded evaluator carrying each current as its coupling-monomial components, with weights unchanged within the reweight tests' tolerances.
blocked_by: []
opened: 2026-10-05
tags: [reweight, helas-eval, graded-evaluator, squared-orders]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L668-L730", title: "TODO.md entry T059"}
  - {id: todo-sq, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L603-L609", title: "TODO.md entry T054"}
---
The polynomial path collects the amplitude by coupling monomial, reading the
class amplitudes off `K` node evaluations chosen by pivoted QR (`K = 1 + n`
for `n` couplings with one insertion per diagram). That is `K` passes per
event where one graded pass would do: carry each current as its monomial
components, so subtrees without the reweighted couplings are computed once.

The same graded `helas/eval` refactor is what squared-order constraints need
(complex amplitudes grouped by coupling order; see
[squared-order constraints](../feature/squared-order-constraints-refused.md)).
It was deferred until the evaluator performance PRs in flight landed; none
are open now (checked 2026-10-06). The user shelved squared-order constraints
on 2026-09-25, so this item is the one that motivates the refactor.
Reweighting is already 100–320× cheaper than MadGraph's reweight module per
event and hypothesis on this path
([reweight-vs-madgraph results](../../reweight-vs-madgraph-results.md)).
