---
type: Backlog Item
title: "Pairing splits and on-shell vetoes compile their sub-sets one at a time"
description: "PairingSplit::new and group_vetoes call compile_class per group, serially, inside the compile phase that otherwise runs across the pool."
area: performance
state: open
priority: low
closes_when: "Their share of the compile phase is measured on a card with pairing-weighted cuts and a $ veto, and either they compile across the pool or the share is recorded as too small to matter."
blocked_by: []
opened: 2026-11-23
tags: [performance, compile, pairing, onshell-veto]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: lt3, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-T3-report.md", title: "L-T3 report: Found 3"}
---
Sprint L moved the main set compile onto the pool. Two callers inside the
compile phase still loop over groups serially: `PairingSplit::new`
(`pairing.rs`) and `group_vetoes` (`onshell.rs`), each calling
`compile_class` on sub-sets. Their cost is unmeasured; measure it first.
