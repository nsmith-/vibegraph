---
type: Backlog Item
title: UFOModel::particles is a public field beside the name table derived from it
description: "The particle-name table is built from the pub particles map; a debug check catches an added or removed particle at lookup, but a particle edited in place (name or PDG changed, key kept) is not seen."
area: hygiene
state: open
priority: low
closes_when: "UFOModel::particles is private behind accessors that keep the name table in step, or the table is checked on (name, pdg) pairs, with a test of an in-place edit."
blocked_by: []
opened: 2026-10-20
tags: [ufo, api, visibility]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: hy1, resource: "../../sprints/default-particle-names/sessions/HY1-report.md", title: "default-particle-names HY1 report, Found"}
---
About 40 readers across the library and tests use `.particles`; one test
writes it after load. The [default-particle-names sprint](../../sprints/default-particle-names/sprint.md)
added `debug_assert!(names.is_built_from(..))` at `UFOModel::particle_names()`.
