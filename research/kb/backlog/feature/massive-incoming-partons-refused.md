---
type: Backlog Item
title: Processes with massive incoming partons are refused
description: "A proton subprocess whose incoming parton has nonzero mass is refused; 4 CMS cards (a massive b in the proton) stop there."
area: feature
state: open
priority: medium
closes_when: "A proton process with a massive incoming parton integrates with MadGraph's treatment of the incoming mass, and its σ matches MadGraph on one of the survey's cards within the σ-gate tolerance."
blocked_by: []
opened: 2026-10-20
tags: [proton, kinematics, masses, madgraph-parity, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
`derive_flavor_groups` refuses an incoming leg with nonzero mass
(`ProtonError::MassiveInitialState`, `vibegraph-lib/src/proton.rs:768`): the
probe and the beams are on the light cone. In the
[CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md) this is
4 LO cards (first blocker on 2). MadGraph accepts the card; how it handles the
incoming mass (massless kinematics with the mass kept in the matrix element,
or massive momenta) needs reading before implementing.
