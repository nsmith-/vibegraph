---
type: Design Decision
title: "Hygiene sprint shape: one PR, sessions by module cluster on all four points"
description: "One sprint folder and one draft PR; each review session covers one module cluster for maintainability, non-vacuity, visibility and abstractions together."
decided: 2026-10-09
decided_by: human:nsmith-
status: draft
tags: [hygiene, process, sprint]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: user, resource: "../log.md", title: "Hygiene sprint log, 2026-10-09: the user's answers in the planning session"}
---
Chosen over "a PR per cluster" and "sessions by point".

- **One PR.** The sprint branch integrates every session, and the draft PR
  carries the plan and the claims. This follows the lifecycle's integration
  advice ([agent dispatch](../../../workflow/agent-dispatch-and-worktrees.md),
  "Integration").
- **By cluster, all four points at once.** The hygiene agent will review one
  PR's change on all four points, so this is the shape whose lessons transfer.
- **Cross-module patterns** that a by-point split would catch more easily are
  covered in two ways. Each reviewer lists patterns it suspects recur outside
  its cluster, and triage merges the lists, so a pattern seen by two or more
  reviewers is filed as one abstraction item.
- **The clusters** are in [sprint.md](../sprint.md), "Clusters".
