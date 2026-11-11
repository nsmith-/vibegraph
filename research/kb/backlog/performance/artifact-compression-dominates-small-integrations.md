---
type: Backlog Item
title: "zstd compression of the integrate artifact is about 3% of a small integration"
description: "integrate compresses its artifact at a high zstd level: about 0.47 G instructions per p p > e+ e- j run, flat with budget, negligible at 1M x 8 points but about 3% of a small run."
area: performance
state: open
priority: low
closes_when: "A decision on the compression level (or level by artifact size) is recorded with the size/time trade-off measured on a small and a large card."
blocked_by: []
opened: 2026-11-10
tags: [performance, artifact, zstd]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr124, resource: "https://github.com/nsmith-/vibegraph/pull/124", title: "Configuration shares, with the share-accounting performance commits (found there)"}
---
Seen in the share-accounting profile (nsmith-/vibegraph#124): compressing
`grid.bin.zst` costs about 0.47 G instructions on `p p > e+ e- j`, the same
at any budget. At `--neval 1000000 --niter 8` that is noise; at the small
budgets tests and quick checks use, it is about 3%. The artifact now embeds
the model and diagram sets, so its size, and this cost, grow with the card.
