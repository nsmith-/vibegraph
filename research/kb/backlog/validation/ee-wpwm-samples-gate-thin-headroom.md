---
type: Backlog Item
title: ee_to_wpwm's samples gate sits at 1.57× its p floor
description: "KS pt(w+) at seed 0x5a4d0003 reads min p 1.573e-4 against the 1e-4 floor on 3 and 5 seeds (ee_to_wpwm_cw 2.4×), the thinnest samples headroom in the suite; not caused by this sprint."
area: validation
state: open
priority: low
closes_when: "The cell's thin headroom is diagnosed (a real shape difference in pt(w+), the seed, or the gate's statistic) and either fixed or recorded as expected with the measured headroom over 5+ seeds."
blocked_by: []
opened: 2026-10-31
tags: [samples-gate, headroom, ee-to-ww]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-31}
sources:
  - {id: f1, resource: "../../sprints/alpha-share-tails/sessions/F1-report.md", title: "alpha-share-tails F1 report, headroom probe and Found 4"}
---
Seen in `probe_proton_samples_p_floor_headroom` during F1's validate pair,
identical in the base run; not investigated.
