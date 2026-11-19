---
type: Backlog Item
title: "The run-card parser skips $-led lines without saying so"
description: "Lines such as '$DEFAULT_PDF_SETS = lhaid' are skipped silently; a card relying on one runs at the default instead, with no warning."
area: feature
state: open
priority: medium
closes_when: "integrate (and the run-card report) warns naming each skipped $-line parameter, with a test."
blocked_by: []
opened: 2026-11-19
tags: [run-card, warnings, silent-wrong-answer]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Found by K-C1 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-C1-report.md), Found).
