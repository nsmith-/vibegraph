---
type: Backlog Item
title: Scale, PDF and αs variation weights are not written
description: "CMS stores per-event scale, PDF and αs weights from MadGraph's systematics module on 59 of 65 LO gridpacks; vibegraph writes none."
area: feature
state: open
priority: high
closes_when: "generate writes scale, PDF and αs variation weights in MadGraph's systematics format (initrwgt weight groups and per-event rwgt weights), so downstream CMS tooling reads them unchanged, and the weights for the CMS runcmsgrid.sh arguments match MadGraph's systematics on a banked sample."
blocked_by: []
opened: 2026-10-20
tags: [systematics, lhef, pdf, scales, alpha-s, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
In the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md)
every LO card sets `use_syst = True`, and genproductions' `runcmsgrid.sh` runs
MadGraph's systematics module after generation on 59 of 65 LO (20 of 23 NLO)
gridpacks. Its arguments are always
`--mur=1,2,0.5 --muf=1,2,0.5 --together=muf,mur,dyn --dyn=-1,1,2,3,4
--alps=0.5,1,2`, with PDF lists of about 880, 491+ or 1,275 members. The
other 6 LO cards are Run 2 packs that use SysCalc from the run card.

vibegraph writes no variation weights. Until it does, a card asking for them
should say so: [systematics-request-dropped-silently](../hygiene/systematics-request-dropped-silently.md).
