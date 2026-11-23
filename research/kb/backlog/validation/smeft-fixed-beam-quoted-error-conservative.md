---
type: Backlog Item
title: "Quoted σ errors on the fixed-beam SMEFT rows are about 3× the seed scatter"
description: "Over 5 seeds the fixed-beam SMEFT ee→tt̄ rows show χ²/dof about 0.1 against their quoted errors, so the quoted error is about 3× conservative."
area: validation
state: open
priority: low
closes_when: "20 seeds on one fixed-beam SMEFT row measure the scatter against the quoted error, and the over-quote is either traced to its cause in the error combination and fixed, or recorded with its cause."
blocked_by: []
opened: 2026-11-22
tags: [sigma, seeds, error-estimate, smeft, fixed-beam]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
sources:
  - {id: pr1, resource: "https://github.com/nsmith-/vibegraph/pull/168", title: "Sprint L PR 1 (squared orders), Found (e)"}
---
Seen in sprint L's PR 1 on the `ee_to_ttx_smeft` family, including the new
NP^2==1 and NP^2==2 splits: five seeds agree far better than each quotes. A
χ²/dof well below 1 is as much a warning as one above it. Compare
[quoted-error-overestimated-on-long-ttx-decay-chain](quoted-error-overestimated-on-long-ttx-decay-chain.md),
which may share the cause.
