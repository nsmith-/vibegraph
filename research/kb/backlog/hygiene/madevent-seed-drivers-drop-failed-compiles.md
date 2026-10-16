---
type: Backlog Item
title: MadEvent seed drivers drop a seed on a parallel-make race
description: A make race in MadEvent (libgammaUPC.a has no index) failed 2 of 204 diagnostic runs before integrating, and a failed generate_events leaves an ME5_debug symlink in the caller's directory.
area: hygiene
state: open
priority: low
closes_when: The banking drivers that source madevent_seeds.sh retry a failed compile once before giving up on a seed, and run generate_events from a scratch working directory.
blocked_by: []
opened: 2026-10-15
tags: [madgraph, banking, tooling]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: v1, resource: "../../sprints/sigma-2to4-offset/sessions/V1-report.md", title: "sigma-2to4-offset V1, Found 4 and 5"}
---
Two of 204 pinned-3.7.1 runs failed to link (`libgammaUPC.a: archive has no
index`), which is a parallel-make race, and recovered on retry. A banking
driver that drops the seed instead silently shrinks the seed set. A failed
`generate_events` also leaves a `ME5_debug` symlink in the caller's working
directory. One appeared in a worktree root.
