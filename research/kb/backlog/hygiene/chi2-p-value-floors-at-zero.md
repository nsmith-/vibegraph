---
type: Backlog Item
title: "χ² p-values below about 1e-16 read as exactly 0"
description: "puruspe::gammq 0.4.4 returns 0 once x > a + 20(√a + 1); stats.rs's χ² test and seed-record print such tails as 0 rather than as a bound."
area: hygiene
state: open
priority: low
closes_when: "A printed χ² p-value of 0 is either replaced by a non-truncating upper incomplete gamma (pinned by a known-answer test such as Q(2, 65) = 66e⁻⁶⁵) or printed as a bound everywhere it is shown."
blocked_by: []
opened: 2026-10-16
tags: [statistics, dependencies, reporting]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
F3 in `sprints/sigma-reference-scatter` found that `puruspe::gammq` 0.4.4
truncates its series (`gamma.rs:260`) and returns exactly 0 in the deep tail;
across 1–199 dof the true tail there is at most 8.5e-17. `seed-record` now
prints `< 1e-16`. `vibegraph-lib/src/stats.rs:274` still reports such a
p-value as 0. No verdict changes, since every threshold in use is far above
1e-16; the cost is a report that states a bound as a value.
