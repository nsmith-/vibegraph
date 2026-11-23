---
type: Backlog Item
title: "The diagram artifact's serial zstd-19 encode is most of what is left of a fast enumerate"
description: "On DY+0–3j enumerate at -j 4, about 3.4 s of the 4.3 s wall is the single-threaded zstd-19 encode of the artifact, after enumeration itself fell to 0.9 s."
area: performance
state: open
priority: medium
closes_when: "Whether the artifact's file bytes are a contract is decided and recorded; the encode is measured across levels and with the multithreaded encoder on DY+0–3j and a 4j card; and the chosen setting lands with its measured wall time."
blocked_by: []
opened: 2026-11-23
tags: [performance, artifact, zstd, enumerate]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: lt3, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-T3-report.md", title: "L-T3 report: job 1 and Found 1"}
---
Sprint L's flavour prune cut DY+0–3j `enumerate` to 4.30 s wall at `-j 4`
(release), of which enumeration is 0.86 s. Most of the rest is the
single-threaded zstd-19 encode of the artifact, which embeds the model.
`integrate`'s artifact uses the same level.

A multithreaded zstd encoder changes the compressed bytes but not the
digests, which are taken over the decoded content. So the first question is
whether anything treats the artifact's file bytes as a contract (release
checksums, a gridpack cache, a test pinning a file hash). Then measure the
level against size and time, and the multithreaded encoder.

Related: [artifact-compression-dominates-small-integrations](artifact-compression-dominates-small-integrations.md),
the same level on `integrate`'s artifact.
