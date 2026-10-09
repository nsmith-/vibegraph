---
type: Backlog Item
title: validate_scales.rs's module doc says scalefact is unpinned
description: "The module doc says every banked run has scalefact = 1 and that MadGraph applies it twice in one place; pp_to_ll_scalefact2 now pins it, and on 3.7.1 it applies once."
area: hygiene
state: open
priority: low
closes_when: "The module doc of vibegraph-lib/tests/validate_scales.rs describes the scalefact coverage the file actually has."
blocked_by: []
opened: 2026-10-06
tags: [comments, scales, validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: phase1, resource: "../../42-okf-knowledge-bundle-plan.md", title: "Found during note 42 Phase 1 (chunk classification of note 28)"}
---
`vibegraph-lib/tests/validate_scales.rs:46-48` says every banked run has
`scalefact = 1`, so where MadGraph applies it, "and the one place it applies
it twice", is pinned only by unit tests. Both halves are stale. The
`pp_to_ll_scalefact2` run pins the placement (`SCALEFACT_RUNS`, around
line 322), and the double application was read off MadGraph 3.5.7: the
pinned 3.7.1 applies it once per beam
([note 22](../../22-dynamical-scales-plan.md) against
[note 28 §K1](../../28-kt-spine-feature-sprint-plan.md)).
