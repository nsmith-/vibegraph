---
type: Backlog Item
title: Reference σ seed scatter and convergence are untested
description: "The hadronic σ references are one MadEvent run each at one budget, their quoted error trusted as is, though MadEvent seeds have scattered at χ²/dof 3–14 at the 0.1 % level."
area: validation
state: open
priority: medium
closes_when: "Every gated σ reference has independent MadEvent seeds (fresh process directories) and a second budget, the bundle records each row's seed χ²/dof and budget shift, and the σ gates use an error that covers the measured scatter where it exceeds the quoted error."
blocked_by: []
opened: 2026-10-09
tags: [validation, refdata, madgraph, seeds, convergence]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: user, resource: "https://github.com/nsmith-/vibegraph/pull/18#discussion_r4232969692", title: "User review on PR #18: the reference can scatter and be unconverged too"}
---
The [budget-alignment rule](../../validation/budget-alignment-rule.md) sizes
each σ gate so this crate's error is about MadGraph's, and the
[seed sweeps](../../validation/seed-sweeps-and-budget-ladders.md) test this
side for scatter and convergence. The reference side is tested unevenly:

- Generators that source `validation/madgraph/madevent_seeds.sh` (decay widths,
  decay-chain σ and events, on-shell veto, grammar σ) bank one run per seed,
  and their gates read the spread.
- `gen_mlm_references.sh` runs seeds too, but its extra seeds share one
  process directory and inherit its grids, so they are not independent (seed
  χ²/dof 0.3–0.8). Only `FRESH_ROWS` (default `pp_to_ll_0j2j_mlm`) get fresh,
  independent seeds.
- `gen_hadronic_sigma.sh`, which banks the hadronic rows the budget-alignment
  table sizes against, runs each row once: its quoted error enters every pull
  as if honest, and no budget ladder checks its convergence.

That trust is not safe. MadEvent is a VEGAS integrator with the same failure
modes, and a MadGraph-only study for the
[identical-particles decision](../../process/identical-particles-across-decays.md)
measured χ²/dof of 3–14 across MadEvent seeds at the 0.1 % level, full-process
runs included.

Likely shape: independent seeds and a second budget for every gated σ
reference (in the build, or as an audit before each refdata release), the
χ²/dof and shift recorded per row in the bundle, and the σ gates' MadGraph
error inflated where the measured scatter exceeds the quoted error. Rows whose
reference is unconverged at the banked budget are re-banked at a larger one.
