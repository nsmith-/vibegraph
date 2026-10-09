---
type: Backlog Item
title: No hygiene agent type exists for the per-item PR flow
description: "The PR-per-item flow needs a fourth session type, hygiene (or quality), beside feature, validation and performance; it is designed from the hygiene sprint's lessons."
area: hygiene
state: blocked
priority: medium
closes_when: "An agent definition for the hygiene session type exists beside feature-dev, validation-dev and performance-dev, and the manager's per-item sequence includes it."
blocked_by: [hygiene-sprint]
opened: 2026-10-09
tags: [hygiene, process, agents]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: user, resource: "../../decisions/pr-per-backlog-item.md", title: "Decision: one PR per backlog item, four session types"}
---
The hygiene agent (alternative name: quality) reviews a change for
maintainability, test non-vacuity, visibility (`pub` only where needed) and
reusable abstractions. It is designed from what the
[hygiene-sprint](hygiene-sprint.md) learns, not before, and then becomes a
regular member of the per-item PR sequence
([pr-per-backlog-item](../../decisions/pr-per-backlog-item.md)).
