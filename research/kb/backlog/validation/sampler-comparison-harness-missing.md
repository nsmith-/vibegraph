---
type: Backlog Item
title: No harness runs the adopt-or-reject comparison between samplers
description: "PR #48's grid-rule table was run by hand; every candidate sampler needs the same seed sweep (σ spread, χ²/dof, ε_unw, Hill index, evals to target, variance×CPU) as one reusable command."
area: validation
state: blocked
priority: medium
closes_when: "One pixi task runs named arms (integrate flag sets) over rows, budgets and seeds, writes per-run JSON, and renders the comparison table; it reproduces PR #48's grid-rule measurements (its default and bin-width-bound K = 4 arms) on pp_to_llj_mlm within their seed spread; the protocol is recorded as a validation concept."
blocked_by: []
opened: 2026-10-17
tags: [sampler, seed-sweep, unweighting, weight-tail, tooling, statistics]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: p1, resource: "https://github.com/nsmith-/vibegraph/pull/48", title: "PR #48: the hand-run grid-rule measurement table"}
---
Every sampler candidate closes on the same question, adopt or reject, and the
record that answers it should be produced the same way each time. PR #48's
grid-rule measurements built that table by hand: per (row, arm, seed) an `integrate` and a `generate`
run, ε_unw read as `σ / Σⱼ w_maxⱼ` and the Hill index off `generate`'s per-channel
debug log line, CPU from the process. Nothing of it is committed.

**Blocked on PR #48 merging** (draft as of 2026-10-17): its `--grid-*` flags
are the arms the reproduction check runs, and they are not on `main`. Build it
once, with arms as `integrate` flag sets, so it serves PR #48's grid rules and
`--sampler` kinds once
[integrate-cannot-select-a-sampler](../feature/integrate-cannot-select-a-sampler.md)
lands. Per arm and row: σ as the unweighted seed mean, the seeds' spread,
quoted error, seeds' χ²/dof about their mean, pull against the MadEvent seed
record; ε_unw at the default truncation over a fixed scan; Hill median and
worst channel; points to reach a `Budget::Target` stop; `quoted²·CPU` and
`spread²·CPU` relative to a baseline arm; and the training evaluations kept
apart from the evaluations after training. `generate` needs to emit ε_unw and
the per-channel Hill indices as machine-readable output instead of a debug line.

Statistics follow the house rules
([seed sweeps](../../validation/seed-sweeps-and-budget-ladders.md)): ≥5 seeds
per arm combined by unweighted mean, reading spread and χ²/dof rather than one
pull; 20+ seeds on the two arms a verdict turns on; a budget rung as a second
axis. Verdict tables are written before the run
([pre-registered verdicts](../../validation/pre-registered-verdicts.md)). The
seed scaffolds hand-copied across the σ tests
([sigma-gate-scaffold-hand-copied](../hygiene/sigma-gate-scaffold-hand-copied.md))
are the code this should share with, not a fourth copy.
