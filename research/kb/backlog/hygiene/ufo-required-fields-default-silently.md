---
type: Backlog Item
title: "Required UFO fields default silently, and SLHA DECAY Auto is ignored"
description: "A typo'd parameter nature becomes internal, a missing value becomes 0, a missing pdg becomes 0 and unnamed vertices overwrite each other; an SLHA DECAY Auto row is silently skipped."
area: hygiene
state: open
priority: medium
closes_when: "Each required UFO field missing or malformed is refused with the field named (as MadGraph's object_library requires it), and DECAY Auto has a stated policy."
blocked_by: []
opened: 2026-10-10
tags: [ufo, refusal, input-validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: r-c-report, resource: "../../sprints/hygiene/sessions/R-C-report.md", title: "R-C report (hygiene sprint)"}
---
Sites: `ufo/parameters.rs`, `couplings.rs`, `particles.rs`, `vertices.rs` (R-C.19); `ufo/slha.rs` (R-C.18 policy half; its parse-error test landed in the hygiene sprint).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [R-C](../../sprints/hygiene/sessions/R-C-report.md).
