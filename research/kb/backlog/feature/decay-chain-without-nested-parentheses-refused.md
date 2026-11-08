---
type: Backlog Item
title: "A decay chain that decays a daughter without nesting parentheses is refused"
description: "'(R2p23* > t~ nuEM, t~ > w- b~, w- > lepM anuEM)' is refused because the core has no W- to decay; vibegraph says MadGraph would drop that decay. MadGraph's behaviour is unchecked."
area: feature
state: open
priority: medium
closes_when: "MadGraph's reading of this decay syntax is established from its output for the LQLQ card (dropped, or applied to the t~ daughter), vibegraph matches it (with a warning if MadGraph drops), and an LQLQToBLTNu card gets past enumeration."
blocked_by: []
opened: 2026-11-07
tags: [decay-chains, proc-card, madgraph-parity, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
---
CMS produced these gridpacks, so MadGraph accepted the line. If it silently
drops the W decay, the gridpack's σ includes undecayed W's and vibegraph
should reproduce that with a warning. If it decays the W from the top, the
refusal is wrong. Read the gridpack's subprocess list to tell which.

Refused in `main-202a540-200-pc` (2): LQLQToBLTNu_madgraph-M2400_13p6TeV,
LQLQToBLTNu_madgraph-M2600_13p6TeV. Census: [refusals at 202a540](../../validation/cms-gridpack-refusals-202a540.md).
