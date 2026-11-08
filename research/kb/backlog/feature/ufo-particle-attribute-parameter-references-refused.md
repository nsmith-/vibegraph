---
type: Backlog Item
title: "A UFO particle whose charge refers to a parameter object is refused"
description: "heco_spinhalf_withZ0 sets charge = n.value, a reference to a model parameter, and vibegraph's particle parser requires a number."
area: feature
state: open
priority: medium
closes_when: "Particle attributes that reference a parameter's value are resolved at load (as MadGraph's import evaluates them), with a test, and both HSCP-MCP-PhotonFusion gridpacks get past model load."
blocked_by: []
opened: 2026-11-07
tags: [ufo, model-loading, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
---
`Particle parse error: particle 'heco': charge must be a number`
(`particles.py:397`, `charge = n.value`). The charge is a model parameter so
that one UFO serves every charge point of the scan.

Refused in `main-202a540-200-pc` (2): HSCP-MCP-PhotonFusion_Par-M-600-Q-39,
HSCP-MCP-PhotonFusion_Par-M-1600-Q-33. Census: [refusals at 202a540](../../validation/cms-gridpack-refusals-202a540.md).
