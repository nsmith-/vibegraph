---
type: Backlog Item
title: "enumerate's ldjson progress has no total"
description: "enumerate --output-ldjson emits progress with a climbing done count but no total, so a consumer cannot show a bar or ETA for a large multi-line card; what done counts is undocumented."
area: feature
state: open
priority: medium
closes_when: "enumerate's progress carries a total (or fraction) known before the walk, or, if none can be known, documents what done counts and adds the running diagram count and the current process line (k of n) as progress fields; a CLI test pins the fields."
blocked_by: []
opened: 2026-11-17
tags: [cli, ldjson, progress, enumerate, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Requested by simshop, a CMS private-MC web app. Its checker ran `enumerate
proc_card.dat --run-card run_card.dat -j 1 --svg first=50` with a 120 s
timeout under vibegraph 0.2.1. The card was `sm-no_b_mass`, `generate p p >
e+ e-` plus `add process` lines up to `p p > e+ e- j j j j`, and the run hit
the timeout. Its stream carried `progress` objects with `fields.stage =
"enumerate"` and a `done` count climbing into the thousands, but no `total`.
Its `log` objects were per-subprocess lines ("392 diagrams for g u > e+ e- g u
b b~").

In simshop's order of value:
1. A `total`: the number of subprocesses or leg-label assignments the walk
   will visit, if it can be known before the walk starts.
2. Otherwise, document what `done` counts, and add a running diagram count and
   "process line k of n" as progress fields. Line k of n alone already helps.

A related item is [`ldjson-stream-gaps`](ldjson-stream-gaps.md), on progress
throttling and the panic case.

Raised to medium (2026-11-17): simshop ranks it second among its development
blockers, because its checker now streams the ldjson to the page, and large
checks show no ETA.
