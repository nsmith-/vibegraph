---
type: Backlog Item
title: budget.rs's progress test keeps its own tracing capture subscriber
description: "The shared test_capture helper replaced the private subscribers in progress.rs and runcard/classes.rs; budget.rs's progress-total test still carries its own copy."
area: hygiene
state: open
priority: low
closes_when: "budget.rs's the_progress_total_projects_the_convergence_target uses crate::test_capture::capture_primed with its primer, and no private Subscriber copy remains in the crate's tests."
blocked_by: []
opened: 2026-10-20
tags: [tests, tracing, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: hy1, resource: "../../sprints/runcard-hidden-lines/sessions/HY1-report.md", title: "runcard-hidden-lines HY1 report, H1"}
---
The runcard-hidden-lines hygiene review moved the test-capture subscribers
into `vibegraph-lib/src/test_capture.rs`. Its `budget.rs` part was reverted
because a separate design session owns `budget.rs` for now. The migration is
mechanical: `capture_primed(prime, run)` keeps the existing primer, which the
full-suite run needs (without it the test failed under load: "one event per
iteration", left 0, right 6). Do it once that design session has landed.
