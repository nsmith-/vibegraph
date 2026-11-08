---
type: Backlog Item
title: "Subprocesses with different outgoing masses are refused instead of split"
description: "ZPrimeToTT groups u u~ > ta+ vt b … with u u~ > e+ ve b …; the grouping refuses ('split them by outgoing masses first') where it could split the parts itself."
area: feature
state: open
priority: medium
closes_when: "Flavour grouping splits subprocesses by outgoing masses automatically (as it splits incoming masses), with a test, and ZPrimeToTT_M2000GeV_W200GeV integrates with σ matching MadGraph by the multi-seed criterion."
blocked_by: []
opened: 2026-11-07
tags: [flavour-groups, phase-space, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
---
`subprocesses u u~ > ta+ vt b d u~ b~ and u u~ > e+ ve b ta- vt~ b~ have
different outgoing masses, so no single phase-space map covers the sum`. The
massive tau is the difference. The incoming-mass split already exists; this is
its outgoing counterpart.

Refused in `main-202a540-200-pc` (2): ZPrimeToTT_M2000GeV_W200GeV,
ZPrimeToTT_M6000GeV_W1800GeV. Census: [refusals at 202a540](../../validation/cms-gridpack-refusals-202a540.md).
