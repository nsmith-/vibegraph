---
type: Backlog Item
title: "An informational amplitude row's bookkeeping lists are not checked two-way"
description: "ee_to_wpwm_cw has no CONFIG_AMP_SIGNS entry and its report row says two linear-level checks failed, but the row's info mode hides it; nothing makes an info row's lists two-way."
area: validation
state: open
priority: low
closes_when: "Every amplitude row, info or gated, either has its CONFIG_AMP_SIGNS (and pairing) entry or a named exemption, checked two-way by a test."
blocked_by: []
opened: 2026-10-16
tags: [amplitudes, oracle, bookkeeping]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: r2, resource: "../../sprints/amplitude-signs/sessions/R2-report.md", title: "R2 report, Found"}
---
R2 Found 2 and 8. The gated rows' lists are two-way (a stale entry fails); the info rows' are not, so a missing entry on an info row reads as a pass.
