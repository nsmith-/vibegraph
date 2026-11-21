---
type: Backlog Item
title: "MadGraph folds restricted internal parameters that are exactly 0 or 1 into constants; vibegraph does not"
description: "Under a restriction, MadGraph's fix_parameter_values freezes internals that evaluate to 0 or 1; vibegraph keeps them live, so a later param card moving an upstream input could give a different value."
area: validation
state: open
priority: medium
closes_when: "At load, an internal that is exactly 0 or 1 under the restriction while a free external is upstream of it is either folded as MadGraph does or refused by name, with a test that moves the input through a param card."
blocked_by: []
opened: 2026-11-17
tags: [ufo, restriction, madgraph-parity, silent-wrong]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Found by sprint K's K-R1 session
(its report, `sessions/K-R1-report.md` in the sprint folder).
MadGraph's `detect_special_parameters` and `fix_parameter_values`
(`models/import_ufo.py` at 3.7.1) turn restricted *internal* parameters that
come out exactly 0 or 1 into constants. vibegraph does not.

K-R1 counted 37 such internals on `sm`, 66 on the special-values card and 64
on `cG_cHB_massless`. It moved each free external upstream of one, one at a
time, on those three cards: no coupling moved. A param card that moves an input
these internals depend on could still make vibegraph and MadGraph disagree
silently. That is unmeasured.
