---
type: Backlog Item
title: "cuts.rs and maps.rs repeat the weakest-of-filters reduction"
description: "cuts.rs:2247 and maps.rs:255 each reduce a channel's pairing filters to the weakest bound in their own code."
area: hygiene
state: open
priority: low
closes_when: "One helper computes the weakest-of-filters reduction for both sites."
blocked_by: []
opened: 2026-11-24
tags: [multichannel, cuts, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
---
Found by sprint L's hygiene pass L-Z4 on PR 5 (#184) and `ew_recombine.py`
(#170, #196).
