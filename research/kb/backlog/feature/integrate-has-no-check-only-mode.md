---
type: Backlog Item
title: "Run-card and cut refusals surface only inside a full integrate"
description: "enumerate sees only the proc card, so run-card and cut refusals (xqcut vs ptj, uncut collinear limits, $ vetoes with cut_decays = F) appear only after a batch integrate starts."
area: feature
state: open
priority: medium
closes_when: "integrate --check (or equivalent) parses every card, builds flavour groups, cuts and the limit guards, and exits with ok or the same refusal object integrate would emit, without sampling; a test pins refusal parity with integrate on at least three refusal kinds."
blocked_by: []
opened: 2026-11-13
tags: [cli, batch, refusal, cuts, integrate]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-13}
---
Reported by the batch-proxy design session, from the CERN host's survey of 199
CMS gridpacks. `enumerate` reads only the proc card. A card refused for its run
card or cuts is found out only when the batch `integrate` starts. Examples are
`ptj < xqcut` under MLM, an uncut photon or collinear limit, a `$` veto with
`cut_decays = F` and a varying exemption, and an unimplemented cut.

A check-only mode would run everything `integrate` does before its first
sampling point, then report. The web UI would run it interactively before
submitting a job. The refusal must be the same object `integrate` emits, so
the two paths cannot drift.
