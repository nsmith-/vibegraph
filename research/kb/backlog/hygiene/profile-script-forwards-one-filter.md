---
type: Backlog Item
title: scripts/profile.sh cannot pass test-harness flags
description: "profile.sh forwards one test-name filter to the test binary and everything after -- to samply, so --test-threads=1 or --ignored cannot reach the harness."
area: hygiene
state: open
priority: low
closes_when: "profile.sh forwards arbitrary harness arguments to the test binary, separately from samply's, and the pixi profile tasks still work."
blocked_by: []
opened: 2026-10-09
tags: [tooling, profiling]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: script, resource: "../../../../scripts/profile.sh", title: "scripts/profile.sh (~:14-40)"}
---
`scripts/profile.sh` takes `<test-name> [test-filter] [-- samply-args...]`.
It execs `samply record "$@" "$EXECUTABLE" "$TEST_FILTER"`, so the binary gets
one filter and nothing else. A profile that needs `--test-threads=1` (to keep
one hot thread) or `--ignored` (the long-tier gates) cannot go through it. The
`fill_arenas` profiling study had to bypass the script. Found by Phase 2
drafter D15.
