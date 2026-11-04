---
type: Backlog Item
title: "Optional UFO files hide read errors, and a model shipping running.py is not refused"
description: "coupling_orders.py and propagators.py are treated as absent on any read error, not only NotFound; and running.py, the one place MadGraph reads a block's Q= scale, is ignored silently."
area: hygiene
state: open
priority: medium
closes_when: "A read error other than NotFound on an optional UFO file is refused naming the file; a model shipping running.py is refused (or its running applied), each pinned by a test."
blocked_by: []
opened: 2026-11-04
tags: [ufo, refusal, sprint-g]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
G-F3 Found 3 and 5. No banked model ships `running.py`. Report: [G-F3](../../sprints/v02-refusals/sessions/G-F3-report.md).
