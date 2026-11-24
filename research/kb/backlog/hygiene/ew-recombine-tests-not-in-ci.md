---
type: Backlog Item
title: "No CI workflow runs the test-ew-recombine pixi task"
description: "ew_recombine.py gained unit tests and a pixi task in #196, but no CI job runs them."
area: hygiene
state: open
priority: low
closes_when: "A CI job runs pixi run test-ew-recombine and fails on a broken test."
blocked_by: []
opened: 2026-11-24
tags: [ci, madgraph, tests]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
---
Found by sprint L's hygiene pass L-Z4 on PR 5 (#184) and `ew_recombine.py`
(#170, #196).
