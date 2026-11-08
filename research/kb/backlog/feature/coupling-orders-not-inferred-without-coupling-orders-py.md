---
type: Backlog Item
title: "A UFO without coupling_orders.py has only QCD and QED, so its own orders are unknown"
description: "Bprime_no_b_mass has no coupling_orders.py and its couplings carry QAD; vibegraph falls back to QCD and QED and refuses the proc card's QAD=2 as not an order of the model."
area: feature
state: open
priority: high
closes_when: "With coupling_orders.py absent, the order set is read from the couplings' order dicts (as MadGraph's import does), with a test on a model lacking the file, and bstar_tW_Right_M3200 gets past enumeration."
blocked_by: []
opened: 2026-11-07
tags: [ufo, coupling-orders, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
---
`failed to enumerate process: coupling order 'QAD' is not an order of this
model (its orders: QCD, QED)`. The model directory has `couplings.py` using
`QAD` but no `coupling_orders.py`. Read how MadGraph's UFO import builds the
order list when the file is missing before choosing the fallback.

Refused in `main-202a540-200-pc` (2): bstar_tW_Right_M3200,
bstar_tW_Right_M3400. Census: [refusals at 202a540](../../validation/cms-gridpack-refusals-202a540.md).
