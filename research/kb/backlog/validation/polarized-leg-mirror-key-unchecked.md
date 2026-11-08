---
type: Backlog Item
title: "MadGraph's mirror key ignores polarization; a polarized initial leg at proton beams is unchecked"
description: "diagram_generation.py:1838 mirrors on PDG ids only, so u{L} u~ could mirror onto a different state; vibegraph's key includes polarization. Unmeasured; whether vibegraph accepts the card is unknown."
area: validation
state: open
priority: low
closes_when: "A card with a polarized initial-state leg at proton beams is run on both sides and the mirror treatment compared, or vibegraph is shown to refuse such a card, with a test either way."
blocked_by: []
opened: 2026-11-07
tags: [mirror, polarization, proton, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hr1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-R1-report.md", title: "H-R1 report, Found 2"}
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log: H-R1 done, 'MadGraph's mirror key ignores polarization'"}
---
Found by H-R1 while fixing the explicit-ordering mirror (#96). MadGraph's mirror
check (`diagram_generation.py:1838`) compares PDG ids only, so on a line with a
polarized beam leg (`p{L} p > …`) it would mirror `u{L} u~` onto `u~ u{L}`-shaped
candidates, which are a different physical state. vibegraph's mirror key includes
the polarizations, so it does not. Neither side was run, and whether vibegraph
accepts a polarized initial-state leg at proton beams at all was not checked. If
MadGraph is confirmed to mirror wrongly, it belongs in
[madgraph-defects](../../validation/madgraph-defects.md).
