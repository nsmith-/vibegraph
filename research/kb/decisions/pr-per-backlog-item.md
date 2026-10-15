---
type: Design Decision
title: One PR per backlog item (or a small related bundle), seen by every session type
description: "After the knowledge-bundle migration, work runs in parallel as one PR per backlog item, or per small bundle of closely related items; a manager runs each session type on it in turn."
decided: 2026-10-09
decided_by: human:nsmith-
verified: [{by: "human:nsmith-", at: 2026-10-09}, {by: "human:nsmith-", at: 2026-10-14}]
status: stable
tags: [process, agents, hygiene]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: user, resource: "https://claude.ai/artifact/N5Rm9hTa3rpkQ3duboK5UB", title: "User review of the Phase B scope decisions, 2026-10-09"}
  - {id: pr18-bundle, resource: "https://github.com/nsmith-/vibegraph/pull/18#discussion_r4232820058", title: "User review on PR #18: bundling small, closely related items"}
  - {id: hygiene-agent, resource: "https://claude.ai/code/session_01Y5NsGfe1rRLuxB4VVe5pV9", title: "User review of the hygiene lessons, 2026-10-14: the hygiene agent's routing and model"}
---
This replaces the [feature → validation → performance sprint
cycle](sprint-rhythm.md) once the knowledge-bundle migration lands.

- **Parallel, one PR per backlog item.** Each item gets its own PR, opened
  as a draft that claims it (`Backlog: <slug>`, note 42 §7.1). Several items
  proceed at once.
- **Small, closely related items may share a PR.** One PR per item alone
  would produce many very small PRs, so a PR may bundle items that are small
  and closely related: they touch the same code or the same gate, and one
  review reads them as one change. [^pr18-bundle] The PR claims each item with
  its own `Backlog:` line and deletes the file of each item it closes. Items
  that are large, or related only by area, keep their own PRs.
- **Every change seen from each perspective.** The PR's manager runs the
  session types on it in turn: feature, validation and performance
  (`.agents/agents/*-dev.md`). A change is then reviewed for what it adds,
  how it is checked and what it costs, instead of waiting for the next sprint
  of that kind.
- **A fourth type, hygiene.** It covers code maintainability, test
  non-vacuity, visibility (`pub` only where something needs it) and reusable
  abstractions. The first work after the migration was one dedicated hygiene
  sprint over the existing codebase
  ([hygiene sprint](../sprints/hygiene/closeout.md)), and its lessons designed
  the agent, `hygiene-dev` ([hygiene review](../workflow/hygiene-review.md)).
- **The sequence is feature, validation, performance, hygiene.** Hygiene runs
  last and fixes what it finds, except that a physics finding goes back to
  `validation-dev` and a performance finding to `performance-dev`, which should
  be rare after the earlier sessions. The manager dispatches it on Opus or
  Sonnet by the change's scope and complexity.[^hygiene-agent]

[^pr18-bundle]: User review comment on PR #18, 2026-10-09.
[^hygiene-agent]: User review of the hygiene lessons, 2026-10-14.
