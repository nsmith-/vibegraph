---
type: Backlog Item
title: "A non-default mxx_only_part_antipart is ignored, though MadEvent uses it to replace class cuts"
description: "classes.rs files mxx_only_part_antipart as IgnoredBenign, but MadGraph puts its keys into pdg_cut, so e.g. {22: True} removes pta/etaa from photons in MadEvent while vibegraph keeps them."
area: feature
state: open
priority: medium
closes_when: "A card whose mxx_only_part_antipart differs from {'default': False} is refused (or its keys folded into the per-PDG cut handling), with a test on a card listing one code."
blocked_by: []
opened: 2026-11-07
tags: [run-card, cuts, madgraph-parity, silent-wrong-answer]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hc1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-C1-report.md", title: "H-C1 report, Found F3"}
---
Found by H-C1. `vibegraph-lib/src/runcard/classes.rs:450` files
`mxx_only_part_antipart` as `IgnoredBenign(B_MXX)` ("qualifies the
`mxx_min_pdg` cut alone"). In MadGraph, `banner.py:4711` puts its keys into
`pdg_cut`, and `setcuts.f:318–334` then overwrites those legs' class cuts with
the per-PDG defaults. So `{'default': False, 22: True}` removes `pta`/`etaa`
from photons in MadEvent while vibegraph keeps them: a silent difference in σ.

The default `{'default': False}`, which 35 banked cards write, is unaffected
([field classification](../../run-card/field-classification.md), "Opaque
defaults"). No survey card is known to set another value (not checked), hence
medium rather than high.
