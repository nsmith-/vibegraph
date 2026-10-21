---
type: Backlog Item
title: A hand-built reweight launch may move an internal parameter without its externals
description: "ReweightPlan::new accepts any name in base.param_values, internal parameters included, whereas the card reader's resolve admits externals only; a launch naming an internal parameter moves it alone."
area: validation
state: open
priority: low
closes_when: "ReweightPlan::new refuses a launch naming a parameter that is not an external (as resolve does), by a test, or the case is shown unreachable from every public constructor."
blocked_by: []
opened: 2026-10-20
tags: [reweight, parameters, api]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: hy1, resource: "../../sprints/ufo-conditional-params/sessions/HY1-report.md", title: "ufo-conditional-params HY1 report, Found 1 (suspected)"}
---
Suspected, not demonstrated. `ReweightPlan::new` is `pub` and takes
`Vec<Launch>` with `pub` fields; since the
[ufo-conditional-params sprint](../../sprints/ufo-conditional-params/sprint.md)
it refuses a name absent from `base.param_values`, but that map holds internal
parameters too. The reweight-card path goes through `resolve`, which admits
externals only, so only a hand-built launch reaches this. Moving an internal
parameter without the externals it is derived from gives an inconsistent model
point. Check `model.params.externals` at the same place.
