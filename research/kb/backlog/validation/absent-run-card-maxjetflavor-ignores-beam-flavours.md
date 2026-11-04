---
type: Backlog Item
title: With no run card, maxjetflavor stays 4 in the five-flavour scheme where MadGraph sets 5
description: "MadGraph's default run card sets maxjetflavor (and asrwgtflavor) to the heaviest quark in the beams; vibegraph's absent-run-card default keeps 4, so a 5F process gets b cuts instead of jet cuts on its b quarks."
area: validation
state: open
priority: high
closes_when: "The absent-run-card default sets maxjetflavor and asrwgtflavor to max(4, |pdg| of every beam quark below 7) as banner.py's create_default_for_process does, a test pins 5 for p p > j j under sm-no_b_mass and 4 under sm, and an explicit run-card value still wins."
blocked_by: []
opened: 2026-11-04
tags: [run-card, cuts, five-flavour, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: banner, resource: "../../../refs/mg5amcnlo/madgraph/various/banner.py", title: "MadGraph banner.py, RunCardLO.create_default_for_process (~:4767–4810)"}
  - {id: runcard, resource: "../../../../vibegraph-lib/src/runcard.rs", title: "vibegraph-lib/src/runcard.rs, the maxjetflavor default (~:1206)"}
  - {id: cuts, resource: "../../../../vibegraph-lib/src/cuts.rs", title: "vibegraph-lib/src/cuts.rs, classify (~:1207)"}
---
MadGraph writes each process's default run card through
`RunCardLO.create_default_for_process`. For a hadronic beam it sets
`maxjetflavor = max([4] + [|id| for beam ids with -7 < id < 7])` and the same
value for `asrwgtflavor`. Under a model with a massless b, `p` holds b, so the
default is 5. vibegraph's absent-run-card default is a fixed 4 (`runcard.rs`
~:1206), so a 5F process run without a run card classes its b quarks as b
rather than jets. Their cuts then differ from MadGraph's: `ptj`, `etaj` and
`drjj` against the b-class defaults. Grouping also refuses `p p > j j j` with
`CutIndicatorDiffers` (see
[the refusal](../feature/five-flavour-ppjjj-refused-cut-indicator-differs.md)).

The rule belongs to the default card only: a run card the user supplies keeps
its value. `asrwgtflavor` matters for MLM α_s reweighting.
