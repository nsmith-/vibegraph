---
type: Backlog Item
title: "MSSM_SLHA2's restricted interaction count (812) is confirmed against MadGraph once but not pinned by a test"
description: "The ported restriction zero rule drops three MSSM_SLHA2 vertices to reach MadGraph's 812; no test pins the count, as smeftsim.rs does for SMEFTsim."
area: validation
state: open
priority: low
closes_when: "A test asserts MSSM_SLHA2's interaction count under its default restriction equals MadGraph's 812, with the MadGraph command recorded beside it."
blocked_by: []
opened: 2026-11-04
tags: [ufo, restrict-card, madgraph-parity, sprint-g]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Checked by the manager in the sprint container (`import model MSSM_SLHA2; display interactions`, 812; #88 comment 6620367621). Log: [v02-refusals](../../sprints/v02-refusals/log.md).
