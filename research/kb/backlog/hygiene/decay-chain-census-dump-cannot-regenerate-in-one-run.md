---
type: Backlog Item
title: "dump_decay_chain_census.py cannot regenerate cms_zto2q_zjj_jj in one full run"
description: "MadGraph raises 'name sm already consider as a python library' once the default sm is imported earlier in the same process, so the census row's banked value cannot be reproduced by a full regeneration."
area: hygiene
state: open
priority: low
closes_when: "A full dump_decay_chain_census.py run regenerates every banked case, cms_zto2q_zjj_jj included (each card in its own process, or MadGraph's model-module cache reset between cards), and the regenerated JSON equals the banked one."
blocked_by: []
opened: 2026-11-17
tags: [tooling, madgraph, census, decay-chains]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Found by sprint K's K-D1 session (its report, `sessions/K-D1-report.md` in the
sprint folder), which kept the banked value and added only its three new cases.
