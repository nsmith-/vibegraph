---
type: Backlog Item
title: "A charged vector current with no unique conjugate silently keeps its own coupling"
description: "conjugate_vector_current returns None when no unique conjugate matches, and bound_coupling then binds the vertex's own coupling, the convention the CKM fix replaced, without a word."
area: validation
state: open
priority: low
closes_when: "A charged vector-current vertex with no unique conjugate is refused or reported, or a census test over the vendored models asserts every such vertex finds one."
blocked_by: []
opened: 2026-10-26
tags: [coupling, convention, refusal, census]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: v3, resource: "../../sprints/ckm-no-b-mass/sessions/V3-report.md", title: "ckm-no-b-mass V3 report, Found 4"}
---
`UFOModel::bound_coupling` (`vibegraph-lib/src/ufo/mod.rs`) is
`self.conjugate_vector_current(vertex).unwrap_or(vertex)`. `None` covers three
cases at once: a self-conjugate vertex, a structure that is not a vector
current, and a charged vector current whose conjugate is not unique. Only the
last is a failure, and it falls back to the pre-fix convention silently.

SMEFTsim splits each charged current into several vertices with the same
particles (`[u~, d, W+]` is `V_331`–`V_334`), which is why the match also
compares orders and structures. A Python replica of the predicate found a unique
conjugate for all 36 SMEFTsim and 24 SM charged vector-current vertices, but it
compared raw colour strings, not this crate's parsed ones
([V3 report](../../sprints/ckm-no-b-mass/sessions/V3-report.md)).
