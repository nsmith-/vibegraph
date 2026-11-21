---
type: Backlog Item
title: "bstar_tW (CERN card 157) refused with zero effective points at survey budget"
description: "Card 157 passes setup on 400c88f9 but integration refuses with \"effective 0.0 points\" at survey budget (1% target, neval 120000). The cause is unexplained."
area: validation
state: open
priority: medium
closes_when: "The cause is named: either a card σ of zero that MadGraph shares, or a phase-space or cut fault that is fixed, or a refusal that names the real reason."
blocked_by: []
opened: 2026-11-21
tags: [cms-gridpacks, integration, refusal]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-21}
sources:
  - {id: cern, resource: "../../sprints/v022-smeft-and-refusals/log.md", title: "Sprint K log, CERN targeted check on 400c88f9"}
---
v0.2.1 failed this card at setup. The release candidate gets past setup, but no
sampled point carries weight. Compare against the gridpack's MadGraph σ and
results.dat: if MadGraph also reads 0, as card 244 did, this is parity.
