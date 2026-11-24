---
type: Backlog Item
title: "The weight-scan warning counts a vanishing group's channels as lost cross section"
description: "On a card with a flavour group whose matrix element vanishes, the w_max scan's warning reports that group's channels as lost σ, though integrate set them aside as carrying none."
area: hygiene
state: open
priority: low
closes_when: "The scan's warning leaves out channels of groups integrate found vanishing, or names them separately as vanishing, and a test on a card with a vanishing group pins it."
blocked_by: []
opened: 2026-11-23
tags: [generate, unweighting, warnings, flavour-groups]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: lg2, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-G2-report.md", title: "L-G2 report: Found F2"}
---
A user reading the warning on g2HDM card 894 would think some σ was dropped.
None was: the group's matrix element is zero at the card.
