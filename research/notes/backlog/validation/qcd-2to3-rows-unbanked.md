---
type: Backlog Item
title: p p > j j j and the 2→3 QCD partonic rows have no reference or manifest rows
description: The 2→3 QCD processes became reachable with abedb81 but have no MadGraph reference generation, no manifest rows and no gates.
area: validation
state: open
priority: medium
closes_when: MadGraph references for p p > j j j and the 2→3 QCD partonic processes are banked and their manifest rows carry measured cells, σ read over a ≥5-seed sweep.
blocked_by: []
opened: 2026-08-06
tags: [coverage, qcd, 2to3, heavy-tail, madgraph-regen]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L391-L399", title: "TODO.md entry T027"}
---
Commit `abedb81` made the momentum guard in `run.rs`'s `Op::Add` arm
scale-relative. That made `u u~ > g g g`, `g g > g u u~` and `g u > g g u`
reachable; `g g > g g g` already ran.

None of these processes has any of the following:
- a MadGraph reference script under `validation/madgraph/scripts/`;
- a row in `validation/manifest.toml`;
- a gate.

Generating the references takes hours of MadGraph
([note 36 §1](../../36-banked-open-ends-plan.md)).

Facts for whoever takes this on:
- **Convergence.** All four 2→3 reproducers hit the convergence cap, with
  per-iteration χ²/dof between 2.5 and 8.2. This is the 2→6 heavy-tail
  pathology one multiplicity lower and much cheaper to study. Budget a ≥5-seed
  sweep and never read a single run.
- **Guard tolerance.** `abedb81`'s tests hold the guard's measured residue
  distribution: at most 1.4 ulps over 1088 sums, against a 1024-ulp bound. Use
  that data if the tolerance is ever revisited.
