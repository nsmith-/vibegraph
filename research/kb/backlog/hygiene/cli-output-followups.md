---
type: Backlog Item
title: "Loose ends from the CLI-output fixes: a stale format floor, a 0.0 χ² default, survey tags"
description: "artifact-format-versioning.md still names floor 15; the report collator defaults χ²/dof to 0.0; the survey keys a multi-tag part by its first tag and does not parse μF."
area: hygiene
state: open
priority: low
closes_when: "The versioning concept states the current floor; IntegralsRow's χ²/dof is optional and prints n/a when absent, with a test; the survey records every tag of a part (PARSER_VERSION bumped) and parses the μF line, with tests."
blocked_by: []
opened: 2026-11-04
tags: [cli, output, cms-survey, sprint-g]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
G-HY3 Found 1, 3, 4 and 5 ([report](../../sprints/v02-cli-output/sessions/G-HY3-report.md)).
`report.rs:281`/`:312` and `cells.rs:150` hold the χ²/dof default; every
current writer sets a value, so it is latent.
