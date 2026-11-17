---
type: Backlog Item
title: The τ Breit–Wigner map's MLM gain is confounded with allocation
description: "--map-tau breit-wigner lowers pp_to_ll_0j2j_mlm's cost × variance by about 10% (3 seeds), but its survey also gives @2 1.4× more points per iteration; a pinned-allocation run would separate the two."
area: performance
state: open
priority: low
closes_when: "A fixed-budget A/B of --map-tau breit-wigner against the default, with the part allocation pinned equal on both arms, over ≥5 seeds, records whether the map itself lowers @1 and @2's spread."
blocked_by: []
opened: 2026-11-10
tags: [mlm, performance, phase-space-map, ab-test]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: diag, resource: "../../performance/mlm-integration-cost-vs-madevent.md", title: "MLM integration cost against MadEvent"}
---
3 seeds: 5.47e8 against 6.10e8 (sd 0.34e8 and 0.29e8); @2 S 544 against 658, @1 457
against 505, @0 unchanged ([A/B](../../performance/mlm-integration-cost-vs-madevent.md)). Its survey also moved @2's share of points
from 24% to 35%, so the gain cannot yet be credited to the map.
