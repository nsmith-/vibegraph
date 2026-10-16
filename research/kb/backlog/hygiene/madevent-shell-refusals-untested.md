---
type: Backlog Item
title: "The MadEvent driver's shell refusals have no committed test"
description: "mes_check_run, mes_check_card and script_driver's no-launch refusal were exercised only by scratch harnesses and manual forced failures."
area: hygiene
state: open
priority: low
closes_when: "A committed test (a validation/madgraph/tests script in the madgraph environment, or the checks moved into a Rust tool in the hermetic layer) fails when any of the three refusals is disabled."
blocked_by: []
opened: 2026-10-16
tags: [hygiene, madgraph, testing, shell]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
Each refusal was shown to catch its case by scratch mutation in
[HY1](../../sprints/sigma-reference-scatter/sessions/HY1-report.md) (a short
event file, a NaN or zero σ, a truncated gzip, an all-skipped card check, a
`launch -f` script), but nothing committed would fail if one were removed. The
hermetic layer has no Python and the repository has no shell-test harness.
