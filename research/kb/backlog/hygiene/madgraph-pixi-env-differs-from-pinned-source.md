---
type: Backlog Item
title: "The madgraph pixi env ships MadGraph 3.5.7 while the pinned reference is 3.7.1"
description: "Oracles run the pinned 3.7.1 submodule source with the env's Python; a bare mg5_aMC from the env is 3.5.7, so an agent can generate a reference with the wrong version."
area: hygiene
state: open
priority: low
closes_when: "The madgraph env's mg5_aMC is the pinned 3.7.1, or the kb names the env-python-plus-submodule invocation as the only supported one and a check refuses the env binary."
blocked_by: []
opened: 2026-11-17
tags: [tooling, madgraph, pixi]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Found by sprint K's K-R1, K-P1 and K-O1 sessions. Every sprint-K brief said
3.5.7 for the same reason.
