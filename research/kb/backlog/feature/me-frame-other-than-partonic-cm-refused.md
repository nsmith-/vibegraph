---
type: Backlog Item
title: "A polarised card with a non-default me_frame is refused"
description: "WW_LL polarises W+ with me_frame naming frame_id 24; only the partonic centre-of-mass frame is supported, so frame-dependent polarised |M|² is refused."
area: feature
state: open
priority: medium
closes_when: "The run card's me_frame is honoured for polarised legs (boost to the named frame before evaluation), with a standalone row in a non-default frame, and WW_LL integrates with σ matching MadGraph."
blocked_by: []
opened: 2026-11-07
tags: [polarization, run-card, madgraph-parity, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
---
`u u~ W+ W- polarizes the massive leg 'W+', so its squared amplitude depends on
the frame it is evaluated in, and the run card's me_frame names frame_id 24;
only MadGraph's default, the partonic centre of mass, is supported`.

Polarised diboson and VBS is an active CMS programme, so more cards will use
this. The related decayed-leg polarisation refusal is
[polarized-intermediate-resonances-refused](polarized-intermediate-resonances-refused.md).

Refused in `main-202a540-200-pc` (2 gridpacks, both labelled WW_LL).
Census: [refusals at 202a540](../../validation/cms-gridpack-refusals-202a540.md).
