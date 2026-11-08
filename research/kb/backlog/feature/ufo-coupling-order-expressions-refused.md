---
type: Backlog Item
title: "A UFO coupling order written as a Python expression is refused at model load"
description: "dim6top_LO_UFO writes 'QCD': 1 if configuration.norm_chromo_gs else 0 in a coupling's order dict; vibegraph requires literal integers and refuses the model."
area: feature
state: open
priority: medium
closes_when: "Order values that are expressions over the model's configuration module are evaluated at load with that module's defaults (or a stated subset of Python), with a test, and tZq_EFT_CPV_point_1_13p6 gets past model load."
blocked_by: []
opened: 2026-11-07
tags: [ufo, coupling-orders, smeft, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
---
`Coupling parse error: coupling 'GC_541': order must be a dict of order names
to non-negative integers`. The value is
`{'FCNC':1,'QED':1, 'QCD':1 if configuration.norm_chromo_gs else 0}`, read
from the model's own `configuration` module.

The separate case of a negative order (onia_sm's `'QED': -1`, the
GluGluH_HToUpsilon gridpacks) is filed on the Sprint H branch as
`negative-ufo-coupling-orders-refused`.

Refused in `main-202a540-200-pc` (2): tZq_EFT_CPV_point_1_13p6,
ttZ_3l_4l_EFT_CPV_point_0_13p6. Census: [refusals at 202a540](../../validation/cms-gridpack-refusals-202a540.md).
