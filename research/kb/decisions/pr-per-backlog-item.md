---
type: Design Decision
title: One PR per backlog item, seen by every session type
description: "After the knowledge-bundle migration, work runs in parallel as one PR per backlog item; a manager runs feature, validation, performance and later hygiene sessions on it in turn."
decided: 2026-10-09
decided_by: human:nsmith-
verified: [{by: "human:nsmith-", at: 2026-10-09}]
status: stable
tags: [process, agents, hygiene]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: user, resource: "https://claude.ai/artifact/N5Rm9hTa3rpkQ3duboK5UB", title: "User review of the Phase B scope decisions, 2026-10-09"}
---
This replaces the [feature → validation → performance sprint
cycle](sprint-rhythm.md) once the knowledge-bundle migration lands.

- **Parallel, one PR per backlog item.** Each item gets its own PR, opened
  as a draft that claims it (`Backlog: <slug>`, note 42 §7.1). Several items
  proceed at once.
- **Every change seen from each perspective.** The PR's manager runs the
  session types on it in turn: feature, validation and performance
  (`.agents/agents/*-dev.md`). A change is then reviewed for what it adds,
  how it is checked and what it costs, instead of waiting for the next sprint
  of that kind.
- **A fourth type, hygiene (alternative name: quality).** It covers code
  maintainability, test non-vacuity, visibility (`pub` only where something
  needs it) and reusable abstractions. It does not exist yet. The first work
  after the migration is one dedicated hygiene sprint over the existing
  codebase ([hygiene-sprint](../backlog/hygiene/hygiene-sprint.md)), and its
  lessons design the hygiene agent
  ([hygiene-agent-type](../backlog/hygiene/hygiene-agent-type.md)). From then
  on the PR sequence has four session types.
