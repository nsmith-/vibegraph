---
type: Backlog Item
title: A UFO antiparticle declared only by antiname cannot be named in a process
description: "vibegraph resolves names against particle entries only; MadGraph's find_name also matches each particle's antiname, so a UFO whose antiparticle has no .anti() entry is usable in MadGraph and not here."
area: feature
state: open
priority: medium
closes_when: "A process naming an antiparticle that the UFO declares only through its particle's antiname resolves to that antiparticle (by creating the missing entry at parse or indexing antinames), by a test on a toy UFO without the .anti() line."
blocked_by: []
opened: 2026-10-20
tags: [ufo, particle-names, madgraph-parity, non-sm-ufo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: f1, resource: "../../sprints/default-particle-names/sessions/F1-report.md", title: "default-particle-names F1 report, Found 3"}
---
MadGraph's `find_name` (`base_objects.py:563-577` at b7687064) matches a
particle's `name` or its `antiname`. vibegraph's particle map holds only the
entries the UFO constructs, so an antiparticle the UFO never builds with
`.anti()` has no entry. Unchecked whether any CMS-survey model omits an
`.anti()` line. Found in the
[default-particle-names sprint](../../sprints/default-particle-names/sprint.md).
