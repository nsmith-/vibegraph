---
type: Backlog Item
title: A UFO parameters.py with 'if configuration' branches panics
description: "Only top-level assignments in parameters.py are read, so a parameter defined only inside 'if configuration.…:' is missing and expr::eval panics on the reference."
area: feature
state: open
priority: medium
closes_when: "A UFO whose parameters.py defines a parameter only inside 'if configuration.…:' branches loads with the branch MadGraph takes by default, or is refused with an error naming the parameter, by a test; expr::eval no longer panics on a missing parameter."
blocked_by: []
opened: 2026-10-20
tags: [ufo, parser, panic, non-sm-ufo, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
`named_assignments` (`vibegraph-lib/src/ufo/ast_util.rs:15`) yields
top-level `Assign` statements only. Some UFOs (dim6top_LO_UFO among them)
define a parameter such as `MW` only inside `if configuration.<flag>:` /
`else:` branches, so neither branch is read. The first expression that
references it reaches `expr::eval` (`vibegraph-lib/src/ufo/expr.rs:117`),
which panics on an unknown parameter. A panic is not a refusal.

Measured in the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md)
on cards 034 and 035. Settling which branch MadGraph takes needs its
`configuration` defaults for UFO loading.
