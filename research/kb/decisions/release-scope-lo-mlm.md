---
type: Design Decision
title: Release scope is arbitrary-multiplicity LO with MLM merging
description: "The release goal is leading-order generation at arbitrary leg multiplicity with MLM merging; beam polarization and other beam configurations stay in the backlog."
decided: 2026-10-09
decided_by: human:nsmith-
verified: [{by: "human:nsmith-", at: 2026-10-09}]
status: stable
tags: [scope, mlm]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: user, resource: "https://claude.ai/artifact/N5Rm9hTa3rpkQ3duboK5UB", title: "User review of the Phase B scope decisions, 2026-10-09"}
---
The release goal is **leading-order event generation at arbitrary leg
multiplicity, with MLM merging**. MLM matching joined the scope when it
landed (the `mlm` sprint, PR 14,
[note 41](../history/notes/41-mlm-feature-sprint-plan.md)). Everything the
[MadGraph LO parity scope](release-scope-mg-lo-parity.md) covered stays in
scope: the proc-card grammar, decay chains, 1→n decays, the s-channel
restrictions, polarized legs and `add process`.

Outside the release goal, kept as backlog items: beam polarization
(`polbeam1`/`polbeam2`), beam configurations other than unpolarized
proton–proton or fixed-energy partonic beams, CKKW-L merging and NLO.

The hard-error rule carries over unchanged: every unsupported surface a card
can reach is refused by the one check on the fully parsed card, never
silently accepted.
