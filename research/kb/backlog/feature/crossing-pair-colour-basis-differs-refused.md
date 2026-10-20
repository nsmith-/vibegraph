---
type: Backlog Item
title: Crossing pairs that share a matrix element but not a colour basis are refused
description: "Subprocesses like u u > g g u u and u~ u~ > g g u~ u~ share |M|² but not a colour basis, and derive_flavor_groups refuses them; 8 HT-binned MLM CMS cards stop there."
area: feature
state: open
priority: medium
closes_when: "A flavour group may hold subprocesses whose colour bases differ by a conjugation or crossing, with each event's colour flow drawn in its own subprocess's basis, and an HT-binned γ/QCD MLM card from the survey passes grouping with the event colour labels checked per subprocess."
blocked_by: []
opened: 2026-10-20
tags: [flavour-groups, colour, crossing, mlm, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
`derive_flavor_groups` refuses two subprocesses that agree on |M|² but whose
colour bases differ (`ProtonError::ColorStructureDiffers`,
`vibegraph-lib/src/proton.rs:573`, raised at `:865`), because an event of
the second is labelled with a colour flow drawn in the first's indexing.

In pass 2 of the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md)
this refused 8 HT-binned γ/QCD MLM cards, e.g. `u u > g g u u` with
`u~ u~ > g g u~ u~`, and `u b > b g u` with `u~ b~ > b~ g u~`. MadGraph
groups such pairs and maps the colour flow between them.
