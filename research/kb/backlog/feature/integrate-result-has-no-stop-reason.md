---
type: Backlog Item
title: "integrate's ldjson result does not say why the run stopped"
description: "Whether integrate met its target, hit max iterations or evaluations, or was stopped is only in a log line; batch tooling needs it, and the relative error reached, in the result object."
area: feature
state: open
priority: high
closes_when: "IntegrateResult carries a stop_reason (target_met, max_iters, max_evals, fixed_budget, stopped) and the relative error reached, quoted and consistency-scaled; a stop before any iteration is kept emits a refusal; schema_version stays 1 (additive) and each reason is pinned by a CLI test."
blocked_by: []
opened: 2026-11-13
tags: [cli, batch, ldjson, integrate]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-13}
---
Reported by the batch-proxy design session. Today the only record of why
`integrate` ended is a log line such as "target: 0.2000% relative, met after 6
iterations". A batch wrapper deciding whether to resubmit has to parse log
text.

Add `stop_reason` and the relative error reached (both the quoted and the
consistency-scaled values) to `IntegrateResult` (`vibegraph-cli/src/output.rs`).
The decision lives in `budget.rs` and `integrate.rs`. The fields are additive,
so `schema_version` 1 holds.

A run stopped before any iteration was kept must still end its stream with a
final object: a `refusal` of kind `run`, or a new `interrupted` kind. This
pairs with [`integrate-ignores-sigterm-outside-the-tui`](integrate-ignores-sigterm-outside-the-tui.md)
and narrows [`ldjson-stream-gaps`](ldjson-stream-gaps.md), which covers the
panic case.
