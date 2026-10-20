---
type: Backlog Item
title: The automatic WEIGHTED search stops below MadGraph's cap
description: "The lowest-WEIGHTED search stops at (n_ext-2)·max hierarchy, where MadGraph stops at (n_ext-2)·max interaction WEIGHTED, so four CMS BSM cards find no diagrams."
area: feature
state: open
priority: high
closes_when: "The search cap is (n_ext-2) times the largest WEIGHTED of any model interaction, as MadGraph's, and a census test finds MadGraph's diagrams on a card whose lowest order lies above the old cap (e.g. a QED+NP vertex)."
blocked_by: []
opened: 2026-10-20
tags: [diagrams, coupling-orders, weighted, madgraph-parity, non-sm-ufo, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
With no explicit orders, `generate_sets` steps WEIGHTED up from
(n_ext − 2) × the smallest hierarchy and stops at
`max_w = (n_ext − 2) * max_hier`
(`vibegraph-lib/src/diagrams/mod.rs:682-686`). MadGraph's cap is
(n_ext − 2) × the largest WEIGHTED of any interaction
(`diagram_generation.py:1998`, `base_objects.py:1481`). A vertex carrying
two orders (QED+NP, QED+LAM112) has a WEIGHTED above any single hierarchy, so
the diagrams it gives are never reached and the card ends with an empty
diagram set rather than a refusal.

Measured in the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md):
cards 060, 067 and 050 (MadGraph steps to WEIGHTED ≤ 6, vibegraph stops at 4)
and 071 (18 against 16). MadGraph's log for 060 steps to ≤ 5 and finds one
diagram per subprocess.

WEIGHTED is computed in three places with different missing-order rules:
[weighted-order-computed-three-ways](../hygiene/weighted-order-computed-three-ways.md).
