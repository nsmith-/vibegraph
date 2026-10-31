---
type: Backlog Item
title: Where bbx_to_ccx_emmm_qcd0's single-point tail sits, and what removed it on main, is unknown
description: "Channels 551, 564, 369, 393, 604, 608 (b-exchange spines) produced weights 450–2000× the iteration mean on 784582d; on main (#48 + #51) the largest point fell from 4.3e-8 to 2.1e-10 pb, unattributed."
area: validation
state: open
priority: low
closes_when: "The largest-weight point per channel and iteration is recorded (momenta and channel densities, test-only) and its invariants read, and the tail's removal is attributed to #48 or #51 by rerunning seeds 88 and 110 with each change alone."
blocked_by: []
opened: 2026-10-23
tags: [two-to-six, heavy-tail, multichannel, attribution]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-23}
sources:
  - {id: b1, resource: "../../sprints/two-to-six-gate/sessions/B1-report.md", title: "two-to-six-gate B1 report, Found 2 and 5"}
---
Seed 88's channel-551 point reproduces on `784582d` and is the sharpest single
test. Also open: why `uux` (the same topologies with massless u exchange)
shows no such spread. One rarer tail remains on main: seed 55 at 1.2M, one
point in channel 613, which is not floor-bound.

The per-point half now exists on main's tree: `probe_2to6_tail_points`
(`vibegraph-lib/src/hadronic/tail_probe.rs`) records and decomposes any
channel's largest points, and seed 55's 613 point was an uncovered H → ZZ*
half, fixed by the symmetric pair draw ([alpha-share-tails T1](../../sprints/alpha-share-tails/sessions/T1-report.md)).
Channel 551's pooled tail index on main is 1.0–1.5 at the 2% threshold. The
`784582d` attribution (#48 or #51) is still open.

Filed at the close-out of [two-to-six-gate](../../sprints/two-to-six-gate/sprint.md).
