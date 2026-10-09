---
type: Backlog Item
title: Alternating channel-weight and grid refinement is unstudied
description: Multichannel α is adapted once on a survey and then fixed; whether alternating α and grid training reaches a lower-variance mixture is unknown.
area: feature
state: open
priority: low
closes_when: An offline study on recorded g_j(x), f(x) reports the variance an alternating α/grid scheme reaches and its point cost against a 20+ seed spread on both arrangements, and the result is recorded as adopt or reject.
blocked_by: []
opened: 2026-09-06
tags: [research, multichannel, vegas, variance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L763-L776", title: "TODO.md entry T061"}
---
The Kleiss–Pittau α-adaptation runs on a survey before the per-channel VEGAS
grids train, and α then stays fixed. An expectation-maximisation-style loop
(train grids at fixed α, re-derive α from the trained grids' variance shares,
retrain) might converge to a lower-variance mixture.

Measure offline first from recorded `g_j(x)`, `f(x)` on existing samples:
the variance the alternation would reach, the points it costs, and whether it
oscillates. Read it against the estimator's measured seed spread (20+ seeds
on both arrangements): the α update is itself a survey estimate over a Pareto
weight tail of index ≈ 2.

Pre-registered failure criteria: α cycling rather than converging; a win
inside the seed spread; any channel's reallocation falling below its coverage
floor. Guardrail as everywhere in the multichannel: an α floor, never a
coverage split.
