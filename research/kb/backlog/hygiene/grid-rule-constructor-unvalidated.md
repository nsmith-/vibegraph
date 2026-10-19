---
type: Backlog Item
title: set_grid_rule accepts a bin-width bound ≤ 1 or non-finite and the library panics later
description: "The CLI refuses K ≤ 1, but the public set_grid_rule does not; integrate_channels then panics inside with_rule's assert. The predicate is duplicated between CLI and library."
area: hygiene
state: open
priority: low
closes_when: "A GridRule constructor (or set_grid_rule) returns a Result both the CLI and the library use, with a test for the refusal, and the duplicated predicate is gone."
blocked_by: []
opened: 2026-10-18
tags: [vegas, api, validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-18}
sources:
  - {id: hy1, resource: "../../sprints/vegas-low-acceptance-bins/sessions/HY1-report.md", title: "vegas-low-acceptance-bins HY1 report, F4"}
---
Filed at the close-out of [vegas-low-acceptance-bins](../../sprints/vegas-low-acceptance-bins/sprint.md).
