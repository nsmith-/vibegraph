---
type: Backlog Item
title: "The run-card parser drops any line whose value contains '='"
description: "split_line splits at the first '=', where MadGraph's banner.py splits at the last, so every banked card's systematics_arguments line is silently skipped."
area: hygiene
state: open
priority: medium
closes_when: "split_line uses rsplit_once('='), with a test on the banked systematics_arguments line."
blocked_by: []
opened: 2026-10-10
tags: [runcard, parser, latent-bug]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: f-f-report, resource: "../../sprints/hygiene/sessions/F-F-report.md", title: "F-F report (hygiene sprint)"}
---
F-F Found 3 (`banner.py:2902`).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [F-F](../../sprints/hygiene/sessions/F-F-report.md).
