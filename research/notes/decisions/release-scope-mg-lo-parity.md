---
type: Design Decision
title: Release scope is MadGraph leading-order process parity
description: "The release goal is MadGraph LO process parity, without MLM matching and NLO; unsupported card features are refused by one check on the fully parsed proc card."
decided: 2026-09-25
decided_by: human:nsmith-
status: deprecated
superseded_by: release-scope-lo-mlm
verified: [{by: "human:nsmith-", at: 2026-10-09}]
tags: [scope, process-grammar]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L35-L48", title: "TODO.md scope decision (user, 2026-09-25)"}
---

The release goal is **MadGraph leading-order process parity**. Beyond the
[earlier fixed-order SM scope](release-scope-sm-fixed-order.md), it includes
decay-chain syntax, 1→n decay processes, the s-channel restrictions (`>`, `$`,
`$$`), or-multiparticles, and `add process` over processes with the same
final-state multiplicity. MLM matching and NLO follow in later sprints; the
data structures passed along the pipeline leave room for both
([note 38 §3.2](../38-process-grammar-sprint-plan.md)).

The hard-error rule is enforced in one place: the proc card is parsed in full
into MadGraph's `ProcessDefinition` shape, and a single check refuses every
unsupported feature before anything downstream reads the card
([note 38 §3.1](../38-process-grammar-sprint-plan.md)). Its `Unsupported` enum
is the feature backlog against MadGraph.

Superseded on 2026-10-09 by [release-scope-lo-mlm](release-scope-lo-mlm.md):
MLM matching landed (`mlm` sprint, PR 14) and joined the release scope.
