---
type: Backlog Item
title: "A PDF reading costs 1.8× what the precomputed tables did"
description: "Building the x cubic on use made one all-flavour reading 315–328 ns, against 174–177 ns with the precomputed tables; the four Q²-knot rows could be built for all flavours at once, exactly."
area: performance
state: open
priority: low
closes_when: "An all-flavour reading is within 1.2× of the precomputed-table cost, measured on the same machine, with the table oracle still at zero differences, or the cost is recorded as not worth recovering on a process where PDF readings dominate."
blocked_by: []
opened: 2026-11-03
tags: [pdf, interpolation, simd]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
sources:
  - {id: mp1, resource: "../../sprints/cms-pdf-memory/sessions/M-P1-report.md", title: "cms-pdf-memory M-P1 report, Found"}
---
End to end, CMS's 880-member request on `pp_to_ll` is no slower, because the
readings are about 3 s of a run of about 24 s. Where readings dominate, the
cost shows. Building the rows for every flavour together lets the divisions
vectorise, and rows duplicated at band edges can be skipped. Gate the change
with `pdf::table_oracle`.
