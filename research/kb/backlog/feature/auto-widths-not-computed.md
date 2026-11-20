---
type: Backlog Item
title: "DECAY Auto is refused: vibegraph has no width calculator"
description: "A param card with DECAY <pdg> Auto is refused by the binder; MadGraph computes such widths at launch with MadWidth, so simshop must make users type every width."
area: feature
state: open
priority: medium
closes_when: "DECAY <pdg> Auto binds to a width computed at tree level from the model's two-body decays, as MadWidth does by default, the computed widths are reported and banked with the run, and the widths for the SM top, W, Z and H and one BSM particle agree with MadGraph's auto widths at the model's own precision; a decay mode beyond two-body is reported as not included."
blocked_by: []
opened: 2026-11-20
tags: [param-card, widths, madgraph-parity, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-20}
---
The binder refuses `DECAY <pdg> Auto` (`ufo/bind.rs`, `CardError::AutoWidth`):
"computing Auto widths is not supported yet, so give the width as a number".
MadGraph computes those widths at launch with MadWidth, whose default is the
tree-level two-body partial widths.

simshop's param-card editor needs `Auto` as a choice; until this item closes
it requires typed widths. The user asked for this item in v0.2.3 and accepts
two-body tree-level widths to start.

What it needs:
- each particle's two-body decay channels from the model's vertices, with
  on-shell kinematics (closed channels skipped);
- the partial widths from the existing amplitude evaluation, summed over final
  helicities and colours and averaged over the parent's;
- the computed widths reported (per channel and total) and recorded in the
  integrate artifact with the card, so a run stays reproducible;
- MadWidth's own options (its precision threshold, three-body corrections)
  either matched or refused by name.
