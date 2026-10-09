---
type: Backlog Item
title: The codebase has had no dedicated hygiene pass
description: "One sprint to clean up the existing codebase for maintainability, test non-vacuity, minimal visibility and reusable abstractions, whose lessons design the hygiene agent."
area: hygiene
state: open
priority: high
closes_when: "The hygiene sprint has run over the codebase, its findings are fixed or filed as items, and its lessons are written up as the input to the hygiene agent's design."
blocked_by: []
opened: 2026-10-09
tags: [hygiene, process, sprint]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: user, resource: "../../decisions/pr-per-backlog-item.md", title: "Decision: one PR per backlog item, four session types"}
---
Decided by the user on 2026-10-09
([pr-per-backlog-item](../../decisions/pr-per-backlog-item.md)). This is the
first sprint after the knowledge migration. Its scope:

- **Maintainability:** duplicated logic, long functions, unclear module
  boundaries.
- **Test non-vacuity:** tests that pass without comparing anything, gates
  that soft-skip, assertions too loose to fail.
- **Visibility:** items `pub` beyond what needs them. This subsumes
  [lib-pub-api-surface-unaudited](../feature/lib-pub-api-surface-unaudited.md).
- **Reusable abstractions:** patterns repeated across modules that want one
  home.

The sprint records what it learns about running such a review, because that
record is how the hygiene agent type gets designed
([hygiene-agent-type](hygiene-agent-type.md)).
