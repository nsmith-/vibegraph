---
type: Backlog Item
title: "Parameters defined under if configuration.flag: blocks, or with string-valued conditional values, are refused"
description: "dim6top_LO_UFO defines MW, aEWM1 and aEW under if configuration.mw_ew_input: and uses string-valued conditional values; both are refused at model load."
area: feature
state: open
priority: medium
closes_when: "Parameter definitions inside if configuration.<flag>: blocks and string-valued conditional value expressions are evaluated with configuration.py's defaults (the evaluator ufo-coupling-order-expressions-refused added), with a test, and tZq_EFT_CPV_point_1_13p6's model gets past model load."
blocked_by: []
opened: 2026-11-17
tags: [ufo, refusal, cms-gridpacks, dim6top]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Found by sprint K's K-U1 session
(its report, `sessions/K-U1-report.md` in the sprint folder).
The coupling-order expressions of `dim6top_LO_UFO` now evaluate against
`configuration.py`. Its `parameters.py` still stops the load earlier:
- L3818 defines `MW`, `aEWM1` and `aEW` under `if configuration.mw_ew_input:`;
- L3809 and L3866 write values as `'G' if configuration.norm_chromo_gs else '1.'`.

[`ufo-coupling-order-expressions-refused`](ufo-coupling-order-expressions-refused.md)
cannot close on its own oracle until this one is done.
