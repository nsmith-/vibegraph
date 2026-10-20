---
type: Backlog Item
title: pp_to_llj_fixed clears LLJ_MAX_REL by only 1.8×
description: The gate quintet reads +0.284% against LLJ_MAX_REL 0.005 (1.8×); eighty seeds read +0.23% (2.2×), so the margin is a real offset plus a high quintet.
area: validation
state: open
priority: medium
closes_when: The row's +0.2% offset is diagnosed (see llj-family-reads-high-vs-seed-records) or the gate quintet clears ≥2× under a recorded decision to accept the thinner margin.
blocked_by: [llj-family-reads-high-vs-seed-records]
opened: 2026-10-20
tags: [sigma-gate, headroom, llj, seed-sweep]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
On tree `340683c` the `pp_to_llj_fixed` gate's five seeds read +0.284%
against `LLJ_MAX_REL` 0.005. Eighty seeds at the gate budget read +0.231%
(2.2×), and the 600k rung reads +0.22%, so the budget is not what is thin.
The user kept every gate budget on 2026-10-20, so the remaining routes are
diagnosing the offset the ℓℓj family shares or accepting the margin.

Detail: [V1 report, Found 1](../../sprints/sigma-calibration-rerecord/sessions/V1-report.md); [the 2026-10 census](../../validation/seed-headroom-census-2026-10.md).
