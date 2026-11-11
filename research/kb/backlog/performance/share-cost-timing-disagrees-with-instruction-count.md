---
type: Backlog Item
title: "On e+ e- > u u~ g the share-accounting time does not follow its instruction count"
description: "Optimised share accounting costs +0.3% of instructions but measured +3.7% CPU time on e+ e- > u u~ g on a shared 4-vCPU host; cache or IPC effects and host noise are not told apart."
area: performance
state: open
priority: low
closes_when: "A quiet-host (or pinned single-thread) timing of e+ e- > u u~ g with and without shares settles whether the +3.7% is real, and configuration-share-cost.md records it."
blocked_by: []
opened: 2026-11-10
tags: [performance, timing, configuration-shares]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr124, resource: "https://github.com/nsmith-/vibegraph/pull/124", title: "Configuration shares, with the share-accounting performance commits (found there)"}
---
The share-accounting cost measurement (nsmith-/vibegraph#124) found that
instruction counts and CPU time agree on `p p > e+ e- j` but not on
`e+ e- > u u~ g`: +0.33% instructions, +3.7% (+2.2 to +4.5) user time,
measured at load 3.6–4.0 on a shared 4-vCPU host, with the gain inside the
noise. Needs a quiet host (see [benchmark hosts](../../performance/benchmark-hosts.md)).
