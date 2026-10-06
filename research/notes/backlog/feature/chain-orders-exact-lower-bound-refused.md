---
type: Backlog Item
title: A decay-chain part's == or > bound under an overall order is refused
description: ChainOrders refuses a chain part's ==/> bound on an order the line also caps overall; what MadGraph does with these two forms is unmeasured.
area: feature
state: open
priority: low
closes_when: A MadGraph census card for each of the == and > forms records what MadGraph does, and the refusal either matches that behaviour or is replaced by it.
blocked_by: []
opened: 2026-09-26
tags: [process-grammar, decay-chain, coupling-orders, madgraph-parity, refusal]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L313-L316", title: "TODO.md entry T018"}
---
`Unsupported::ChainOrders` (`vibegraph-lib/src/diagrams/check.rs:202`,
raised at `:590`) refuses an overall order on a decay-chain line
(`@1 QED=2`) that one of the chain's parts also constrains with `==` or
`>`. The `<=` and `=` forms are supported: each part's upper bound becomes
the lesser of its own and the overall one (`diagram_generation.py:570`), and a
stitched diagram exceeding the overall order is removed (`helas_objects.py:3986`).

What MadGraph does with `==` and `>` there is not measured. The check's
backlog table marks the variant "not planned", on the reading that a lower or
exact bound has no counterpart in MadGraph's fold. That reading is a hypothesis
until a census card for each form confirms it. Add them to the decay-chain census
(`validation/madgraph/dump_decay_chain_census.py`) first. Detail:
[note 38 §4 D3](../../38-process-grammar-sprint-plan.md).
