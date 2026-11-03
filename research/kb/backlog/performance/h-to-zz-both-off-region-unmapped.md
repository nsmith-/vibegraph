---
type: Backlog Item
title: The H → ZZ* peak with both Z pairs off shell has no map
description: "On bbx_to_ccx_emmm_qcd0, 3–7 points per rung on the H peak with both Z pairs at 39–76 GeV keep single-point shares up to 0.35 after the symmetric pair draw; MadEvent gives this region its own channel."
area: performance
state: open
priority: medium
closes_when: "A both-off component (MadEvent's third H → ZZ* sub-channel, or a map that draws neither pair on its pole) is added and a 20-paired-seed bbx ladder at 600k and 1.2M shows no H point with a single-point share above 1e-3 in 613, 614, 598 or 604, with the sd ratio no worse than 0.708, or the record says why not."
blocked_by: []
opened: 2026-10-31
tags: [two-to-six, heavy-tail, resonance, multichannel, madevent-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-31}
sources:
  - {id: f1, resource: "../../sprints/alpha-share-tails/sessions/F1-report.md", title: "alpha-share-tails F1 report, falsifier (a) and Found 1"}
---
F1's symmetric pair draw covers both one-on, one-off halves. What remains in
falsifier (a) is the both-off population: the largest, 0.353 of its
iteration, is seed 165, channel 604, at 1.2M under the α floor. MadEvent
splits each H → ZZ* config into on/off, off/on and both-off channels, the
last with both Z's on 1/s grids
([MadEvent's split](../../sprints/alpha-share-tails/sessions/madevent-hzz-coverage.md)).
