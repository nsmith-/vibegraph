---
type: Backlog Item
title: "The WEIGHTED search's accepted bound is tested through its debug! log"
description: "weighted_search_record.rs reads the bound from debug! text with a private subscriber, safe only while it is the sole test in its binary, because tracing caches callsite interest process-wide."
area: hygiene
state: open
priority: low
closes_when: "The search loop returns its trial sequence (or accepted bound) to the caller, the_search_accepts_the_bound_madgraph_does asserts that value, and the private subscriber in weighted_search_record.rs is deleted."
blocked_by: []
opened: 2026-10-26
tags: [tests, tracing, diagrams, weighted]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: v2, resource: "../../sprints/weighted-cap/sessions/V2-report.md", title: "weighted-cap V2 report, Found 2 and 3"}
---
`vibegraph-lib/tests/weighted_search_record.rs` installs its own `Subscriber`
and collects the search loop's `debug!` messages (`vibegraph-lib/src/diagrams/mod.rs`
near :715-721). Its doc states the constraint: the file must stay the only test
binary with a scoped subscriber, because `tracing` caches each callsite's
interest process-wide on first hit, and a callsite first reached by a concurrent
test with no subscriber is cached as uninterested. Adding a second test to that
binary, or moving the test, reopens the race. The crate's own `test_capture`
module handles the cache (`rebuild_interest_cache`, `capture_primed`), but it is
crate-private and `cfg(test)`, so an integration test cannot use it.

The search also runs on the calling thread only because the test passes
`EnumerationPool::Ambient`; under `EnumerationPool::default()` it runs on a rayon
worker and a scoped subscriber sees nothing
([V2](../../sprints/weighted-cap/sessions/V2-report.md), Found 3).

Returning the trial sequence removes both traps and lets a refusal quote the
bound (V1 Found 3). Related:
[budget-test-keeps-private-capture-subscriber](budget-test-keeps-private-capture-subscriber.md),
the other private subscriber.
