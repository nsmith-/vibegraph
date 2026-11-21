---
type: Backlog Item
title: "No reusable driver replays one phase-space point through MadEvent's clustering"
description: "K-M1 hand-built a driver replaying one point through MadEvent's clustering; a maintained one against the pinned template would make per-point merge-sequence oracles routine."
area: validation
state: open
priority: low
closes_when: "A committed script builds the replay driver for a given MadGraph process output against the pinned template and prints MadEvent's merge sequence and pt² for a supplied point; one MLM banked card's point is pinned against vibegraph's history."
blocked_by: []
opened: 2026-11-19
tags: [madgraph, mlm, clustering, oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
3.5.7's `checkbw` does not call `cut_bw`, so a replay against it must.

Found by K-M1 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-M1-report.md), Found 6).
