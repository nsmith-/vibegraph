---
type: Backlog Item
title: "Param-card refusals and ignored-entry warnings carry no structured block, code or value"
description: "Under --output-ldjson a card refusal is error_kind card with only a message, and an ignored entry is a free-text WARN log; a consumer must parse text to find the entry."
area: feature
state: open
priority: medium
closes_when: "A card refusal's ldjson record carries a structured reason (kind, parameter name, block, code, value, restriction; one per entry for Several), and each ignored entry is a structured record with block, code, value and cause, on integrate, enumerate and parameters, with a CLI test for each."
blocked_by: []
opened: 2026-11-28
tags: [param-card, ldjson, cli, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-28}
---
Found while answering simshop, whose Parameters wizard step validates an edited
card by running `vibegraph parameters --param-card`.

`CardError` (`vibegraph-lib/src/ufo/bind.rs`) holds the parameter name, block,
code, value and restriction, but `bind_param_card`
(`vibegraph-cli/src/integrate.rs`) turns it into a string, so the `refusal`
record is `{kind: refusal, error_kind: card, message}`. Only `artifact`
refusals populate `reason` (`ArtifactReason`); a `CardReason` alongside it is
the natural shape.

Ignored entries (a computed value such as the SM's MW, a block or code with no
parameter, a decay for a particle the model lacks) are `warn!` calls with a
format string and no fields, so under `--output-ldjson` they become `log`
records whose only payload is `fields.message`. `structured-log-output-missing`
covers log structure in general; this item is the card-binding records a
consumer has to act on.
