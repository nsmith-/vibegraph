---
type: Backlog Item
title: Param, reweight and restrict cards have no field classification
description: "The run card classifies every field as consumed, benign or refused; the other input cards have no such table, so an entry MadGraph acts on could be dropped silently."
area: feature
state: open
priority: medium
closes_when: "The param, reweight and restrict cards each have a classification of every construct MadGraph's reader accepts (consumed, benign with an argument, or refused when it could bite), asserted by a test as the run card's FIELD_CLASSES is."
blocked_by: []
opened: 2026-10-09
tags: [cards, refusal, param-card, reweight, restrict, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: user, resource: "https://github.com/nsmith-/vibegraph/pull/18#discussion_r4233033772", title: "User review on PR #18: the same policy for the other input cards"}
---
The run card's policy ([field classification](../../run-card/field-classification.md))
gives each recognised name exactly one class, and the proc card's one check
([supported-card check](../../process/supported-card-check.md)) refuses every
unsupported construct. Both rest on one rule: a parameter MadGraph acts on and
this crate silently drops is a wrong answer.

The other input cards have no equivalent table:

- **param card** (`ufo/slha.rs`, `ufo/parameters.rs`): blocks and entries the
  model does not name, `DECAY` tables with branching ratios, `QNUMBERS` blocks;
- **reweight card** (`reweight/card.rs`): commands and options beyond the
  `set`/`launch` subset it reads (some are refused today, see
  [reweight-forbidden-schannel-and-as-refused](reweight-forbidden-schannel-and-as-refused.md));
- **restrict card** (`ufo/`): the values restriction reads as zero or one, and
  the entries it cannot act on
  ([restrict-card-unit-values-not-fixed](restrict-card-unit-values-not-fixed.md)).

For each card: list what MadGraph's reader accepts, classify each construct,
and refuse those that could change σ, a weight or the record.
