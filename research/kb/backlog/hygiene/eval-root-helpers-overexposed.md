---
type: Backlog Item
title: spine_sign_from_flow and root_tree_at are pub(super) with no outside caller
description: "Two helpers in helas/eval are visible to their parent module though nothing outside their own file calls them."
area: hygiene
state: open
priority: low
closes_when: "Both are private, or a caller outside their file is named in their docs."
blocked_by: []
opened: 2026-10-16
tags: [visibility, evaluator]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: bh, resource: "../../sprints/amplitude-signs/sessions/B-H-report.md", title: "B-H report, Found"}
---
Found by the hygiene review of the root-free convention sign (PR #30).
