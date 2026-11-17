---
type: Backlog Item
title: "On ZPrimeToTT, vibegraph's seed spread is about twice its quoted error"
description: "Five ZPrimeToTT_M2000 seeds at the default 0.1% target spread 0.18% (χ²/dof 3.47 across seeds) while each run's own χ²/dof is 1.0–1.2; the quoted error may understate the real one."
area: validation
state: open
priority: medium
closes_when: "20 or more seeds of the ZPrimeToTT_M2000GeV_W200GeV card at the default target give the ratio of seed spread to quoted error with its uncertainty; a ratio consistent with 1 closes it, otherwise the cause is named and the error estimate or the target is fixed."
blocked_by: []
opened: 2026-11-17
tags: [validation, vegas, error-estimate, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Found by sprint K's K-X1 session (its report, `sessions/K-X1-report.md` in the
sprint folder). Per-seed σ: 18.946, 18.954, 18.954, 18.909, 18.877 fb, each
quoted ±0.018–0.019 fb. The comparison with MadGraph used the seed spread, so
its pull of +0.50 is unaffected. AGENTS.md: few-seed scatter can mislead in
either direction, so the measurement needs 20+ seeds.
